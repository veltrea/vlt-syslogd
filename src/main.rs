#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;

use eframe::egui;
use parser::{Severity, SyslogMessage};
use std::io::Write;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

struct SyslogApp {
    logs: Vec<SyslogMessage>,
    receiver: mpsc::Receiver<SyslogMessage>,
    auto_scroll: bool,
    filter: String,
    log_file: Option<std::fs::File>,
}

impl SyslogApp {
    fn new(cc: &eframe::CreationContext<'_>, receiver: mpsc::Receiver<SyslogMessage>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // ログファイルの準備
        let _ = std::fs::create_dir_all("logs");
        let log_path = format!(
            "logs/syslog_{}.log",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .ok();

        let mut fonts = egui::FontDefinitions::default();
        load_cjk_font(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        Self {
            logs: Vec::new(),
            receiver,
            auto_scroll: true,
            filter: String::new(),
            log_file,
        }
    }
}

impl eframe::App for SyslogApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(log) = self.receiver.try_recv() {
            if let Some(ref mut file) = self.log_file {
                let log_line = format!(
                    "[{}] [{:?}] [{}] {}\n",
                    log.timestamp,
                    log.severity,
                    log.tag.as_deref().unwrap_or("-"),
                    log.content
                );
                let _ = file.write_all(log_line.as_bytes());
            }

            self.logs.push(log);
            if self.logs.len() > 5000 {
                self.logs.remove(0);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 vlt-syslogd");
                ui.label(egui::RichText::new("UTF-8 Only | Pure Performance").weak());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                    if ui.button("🗑 Clear").clicked() {
                        self.logs.clear();
                    }
                });
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.filter);
                if ui.button("x").clicked() {
                    self.filter.clear();
                }
            });

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            let filter = self.filter.to_lowercase();
            let filtered_logs: Vec<_> = self
                .logs
                .iter()
                .filter(|l| {
                    filter.is_empty()
                        || l.content.to_lowercase().contains(&filter)
                        || l.tag
                            .as_ref()
                            .map_or(false, |t| t.to_lowercase().contains(&filter))
                })
                .collect();

            let row_height = 24.0;
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.auto_scroll)
                .show_rows(ui, row_height, filtered_logs.len(), |ui, row_range| {
                    egui::Grid::new("log_grid_v3")
                        .striped(true)
                        .num_columns(4)
                        .spacing([15.0, 8.0])
                        .show(ui, |ui| {
                            ui.strong("Time");
                            ui.strong("Tag");
                            ui.strong("Severity");
                            ui.strong("Message");
                            ui.end_row();

                            for i in row_range {
                                let log = filtered_logs[i];
                                let (r, g, b) = log.severity.color();
                                let color = egui::Color32::from_rgb(r, g, b);

                                ui.label(&log.timestamp);
                                ui.label(log.tag.as_deref().unwrap_or("-"));
                                ui.label(
                                    egui::RichText::new(format!("{:?}", log.severity))
                                        .color(color)
                                        .strong(),
                                );

                                let message_label =
                                    ui.label(egui::RichText::new(&log.content).color(color));

                                // コンテキストメニュー（右クリック）
                                message_label.context_menu(|ui| {
                                    if ui.button("Copy Message").clicked() {
                                        ui.output_mut(|o| o.copied_text = log.content.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("Copy Raw Packet").clicked() {
                                        ui.output_mut(|o| o.copied_text = log.raw.clone());
                                        ui.close_menu();
                                    }
                                });
                                ui.end_row();
                            }
                        });
                });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

/// OS ごとに日本語（CJK）対応フォントを探して egui に登録する。
/// Windows / macOS / Linux の代表的なフォントパスを順に試し、
/// 最初に見つかったものを Proportional / Monospace の両方に差し込む。
fn load_cjk_font(fonts: &mut egui::FontDefinitions) -> bool {
    // OS 別の候補パス（上にあるものほど優先）
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\meiryo.ttc",   // メイリオ（日本語）
            "C:\\Windows\\Fonts\\YuGothM.ttc",  // 游ゴシック Medium
            "C:\\Windows\\Fonts\\msgothic.ttc", // MS ゴシック
            "C:\\Windows\\Fonts\\msyh.ttc",     // Microsoft YaHei（従来の既定）
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", // Hiragino Kaku Gothic
            "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Hiragino Sans GB.ttc",
        ]
    } else {
        // Linux / その他（Noto CJK・IPA フォントを想定）
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf",
        ]
    };

    for path in candidates {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "japanese_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "japanese_font".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("japanese_font".to_owned());
            return true;
        }
    }
    eprintln!("Warning: no CJK font found; Japanese text may render as tofu (□).");
    false
}

/// GUI に表示する内部ステータス（起動・bind 失敗など）用のメッセージを生成する。
fn system_message(content: String, severity: Severity) -> SyslogMessage {
    SyslogMessage {
        severity,
        timestamp: chrono::Local::now().format("%b %d %H:%M:%S").to_string(),
        hostname: None,
        tag: Some("vlt-syslogd".to_string()),
        content,
        raw: String::new(),
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    // 多重起動防止
    let instance = single_instance::SingleInstance::new("vlt_syslogd_singleton_lock").unwrap();
    if !instance.is_single() {
        // すでに起動している場合は終了（GUIアプリなので静かに終了するかメッセージを出すのが望ましいが、要件は「防止」）
        return Ok(());
    }

    let (tx, rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        // バインド先は環境変数 VLT_SYSLOGD_BIND で上書き可能（既定は標準 syslog ポート 514）
        let addr = std::env::var("VLT_SYSLOGD_BIND").unwrap_or_else(|_| "0.0.0.0:514".to_string());
        let socket = match UdpSocket::bind(&addr).await {
            Ok(s) => {
                let _ = tx
                    .send(system_message(
                        format!("Listening on {} (UDP)", addr),
                        Severity::Notice,
                    ))
                    .await;
                s
            }
            Err(e) => {
                // macOS / Linux では 1024 未満は特権ポートのため root 権限が必要
                let hint = if cfg!(target_os = "windows") {
                    "別プロセスが使用中か、管理者権限が必要です"
                } else {
                    "514 番は特権ポートです。sudo で起動するか、環境変数 VLT_SYSLOGD_BIND=0.0.0.0:5514 を指定してください"
                };
                let _ = tx
                    .send(system_message(
                        format!("Failed to bind {}: {} — {}", addr, e, hint),
                        Severity::Error,
                    ))
                    .await;
                eprintln!("Failed to bind UDP socket on {}: {} — {}", addr, e, hint);
                return;
            }
        };

        let mut buf = [0u8; 8192];
        loop {
            if let Ok((size, _src)) = socket.recv_from(&mut buf).await {
                let raw_msg = &buf[..size];
                let text = String::from_utf8_lossy(raw_msg);
                let parsed = parser::parse_syslog(&text);
                let _ = tx.send(parsed).await;
            }
        }
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_title("vlt-syslogd");

    // アイコンは読み込めなくても致命的ではない（macOS の Dock は .app の icns を使う）
    let icon_data = include_bytes!("../icons/vlt_syslogd_icon.png");
    if let Ok(image) = image::load_from_memory(icon_data) {
        let image = image.to_rgba8();
        let (width, height) = image.dimensions();
        viewport = viewport.with_icon(egui::IconData {
            rgba: image.into_raw(),
            width,
            height,
        });
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "vlt-syslogd",
        native_options,
        Box::new(|cc| Box::new(SyslogApp::new(cc, rx))),
    )
}
