#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod parser;

use eframe::egui;
use parser::SyslogMessage;
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
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
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
        }
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
                ui.label(egui::RichText::new("Hybrid Encoding | High Reliability").weak());

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
                    egui::Grid::new("log_grid_v4")
                        .striped(true)
                        .num_columns(5)
                        .spacing([15.0, 8.0])
                        .show(ui, |ui| {
                            ui.strong("Time");
                            ui.strong("Tag");
                            ui.strong("Severity");
                            ui.strong("Enc");
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
                                ui.label(egui::RichText::new(&log.encoding).weak());

                                let message_label =
                                    ui.label(egui::RichText::new(&log.content).color(color));

                                // コンテキストメニュー（右クリック）
                                message_label.context_menu(|ui| {
                                    if ui.button("Copy Message").clicked() {
                                        ui.output_mut(|o| o.copied_text = log.content.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("Copy as Hex").clicked() {
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

#[tokio::main]
async fn main() -> eframe::Result<()> {
    // 多重起動防止
    let instance = single_instance::SingleInstance::new("vlt_syslogd_singleton_lock").unwrap();
    if !instance.is_single() {
        return Ok(());
    }

    let (tx, rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        let addr = "0.0.0.0:514";
        let socket = match UdpSocket::bind(addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to bind UDP socket on {}: {}", addr, e);
                return;
            }
        };

        // デバッグ用生データ保存ファイルの準備
        let _ = std::fs::create_dir_all("logs");
        let mut debug_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/debug_raw.log")
            .ok();

        let mut buf = [0u8; 8192];
        loop {
            if let Ok((size, src)) = socket.recv_from(&mut buf).await {
                let raw_msg = &buf[..size];

                // 生データのHEXダンプを保存
                if let Some(ref mut file) = debug_file {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    let line = format!(
                        "[{}] [src:{}] raw:{}\n",
                        timestamp,
                        src,
                        hex::encode(raw_msg)
                    );
                    let _ = file.write_all(line.as_bytes());
                    let _ = file.flush();
                }

                let parsed = parser::parse_syslog(raw_msg);
                let _ = tx.send(parsed).await;
            }
        }
    });

    let icon_data = include_bytes!("../icons/vlt_syslogd_icon.png");
    let image = image::load_from_memory(icon_data).expect("Failed to load icon");
    let image = image.to_rgba8();
    let (width, height) = image.dimensions();
    let icon = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_title("vlt-syslogd")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "vlt-syslogd",
        native_options,
        Box::new(|cc| Box::new(SyslogApp::new(cc, rx))),
    )
}
