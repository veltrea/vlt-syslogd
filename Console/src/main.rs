// vlt-syslogd Console（GUI フロントエンド）のエントリポイント。
//
// このバイナリは「常駐サービス(Server 版)に TCP で接続して受信ログを表示する」ビューア。
// 受信は net::run_client が担い、mpsc で SyslogMessage を GUI へ流す。GUI は毎フレーム
// mpsc を drain して自前バッファに溜め、egui で描画するだけ(ネットワークは触らない)。
//
// Portable 版が「自分で UDP を待ち受ける」のに対し、Console は「サービスの配信ポート
// (既定 127.0.0.1:5141)へ接続する」点だけが構造的に違う。GUI 骨格は Portable と共通。

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod control;
mod net;
mod parser;
mod platform;
mod service;
mod settings;

#[cfg(target_os = "macos")]
mod macos_menu;

use eframe::egui;
use net::ConnState;
use parser::SyslogMessage;
use service::ServiceStatus;
use settings::Settings;
use tokio::sync::mpsc;

// ---- 定数 ----
const MAX_LOG_ENTRIES: usize = 10_000;
const WINDOW_TITLE: &str = "vlt-syslogd Console";

/// 編集メニュー(コピー/カット/ペースト等)の操作種別。
/// macOS のネイティブメニューと、Windows/Linux のアプリ内メニューの両方から使う。
#[derive(Clone, Copy)]
enum EditAction {
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

/// 修飾キー付きのキー押下イベントを作る(command は macOS=⌘ / その他=Ctrl 相当)。
fn key_event(key: egui::Key, shift: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            shift,
            command: true,
            ..Default::default()
        },
    }
}

struct ConsoleApp {
    // 受信ログのバッファ。
    logs: Vec<SyslogMessage>,
    auto_scroll: bool,
    filter: String,

    // net::run_client との配線。
    msg_rx: mpsc::Receiver<SyslogMessage>,
    state_rx: mpsc::Receiver<ConnState>,
    addr_tx: mpsc::Sender<String>,
    conn_state: ConnState,

    // サービス状態(別スレッドのポーラから受け取る)。
    svc_rx: std::sync::mpsc::Receiver<ServiceStatus>,
    service_status: ServiceStatus,
    svc_action_msg: Option<String>,

    settings: Settings,

    // 切断バナーの接続先入力。
    addr_input: String,
    addr_error: Option<String>,

    // 環境設定ウィンドウ。
    show_preferences: bool,
    pref_server_addr: String,
    pref_control_addr: String,
    pref_error: Option<String>,
    pref_saved: bool,

    // サーバ側 syslog 設定(制御ポート経由で取得・変更)。
    srv_cfg_status: Option<(bool, String)>, // (成功か, メッセージ)
    edit_bind_addr: String,
    edit_stream_addr: String,
    edit_log_level: String,
    edit_max_size_mb: String,
    edit_keep_files: String,
    srv_cfg_loaded: bool,

    // メニューから積まれた、次の描画で egui 入力へ注入する編集イベント。
    pending_events: Vec<egui::Event>,
    last_text_focus: Option<egui::Id>,

    // About ダイアログ(Windows/Linux 用。macOS は OS 標準パネル)。
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    show_about: bool,
}

impl ConsoleApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        settings: Settings,
        msg_rx: mpsc::Receiver<SyslogMessage>,
        state_rx: mpsc::Receiver<ConnState>,
        addr_tx: mpsc::Sender<String>,
        svc_rx: std::sync::mpsc::Receiver<ServiceStatus>,
    ) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut fonts = egui::FontDefinitions::default();
        load_cjk_font(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        // CJK フォントは行高が高いので、全体のテキストサイズを少し下げ、
        // フローティングウィンドウの枠余白を薄くして間延びを抑える(Portable と同方針)。
        {
            use egui::{FontFamily, FontId, TextStyle};
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles = [
                (TextStyle::Heading, FontId::new(15.0, FontFamily::Proportional)),
                (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
                (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
                (TextStyle::Small, FontId::new(9.0, FontFamily::Proportional)),
            ]
            .into();
            style.spacing.window_margin = egui::Margin {
                left: 8.0,
                right: 8.0,
                top: 6.0,
                bottom: 2.0,
            };
            cc.egui_ctx.set_style(style);
        }

        #[cfg(target_os = "macos")]
        macos_menu::install();

        let addr_input = settings.server_addr.clone();
        let pref_server_addr = settings.server_addr.clone();
        let pref_control_addr = settings.control_addr.clone();

        Self {
            logs: Vec::new(),
            auto_scroll: true,
            filter: String::new(),
            msg_rx,
            state_rx,
            addr_tx,
            conn_state: ConnState::Connecting {
                addr: settings.server_addr.clone(),
            },
            svc_rx,
            service_status: ServiceStatus::Unknown("確認中…".to_string()),
            svc_action_msg: None,
            settings,
            addr_input,
            addr_error: None,
            show_preferences: false,
            pref_server_addr,
            pref_control_addr,
            pref_error: None,
            pref_saved: false,
            srv_cfg_status: None,
            edit_bind_addr: String::new(),
            edit_stream_addr: String::new(),
            edit_log_level: String::new(),
            edit_max_size_mb: String::new(),
            edit_keep_files: String::new(),
            srv_cfg_loaded: false,
            pending_events: Vec::new(),
            last_text_focus: None,
            show_about: false,
        }
    }

    /// 編集メニューの操作を「次フレームで egui 入力へ注入するイベント」として積む。
    fn queue_edit(&mut self, action: EditAction) {
        let ev = match action {
            EditAction::Copy => egui::Event::Copy,
            EditAction::Cut => egui::Event::Cut,
            EditAction::Paste => match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                Ok(text) => egui::Event::Paste(text),
                Err(_) => return,
            },
            EditAction::SelectAll => key_event(egui::Key::A, false),
            EditAction::Undo => key_event(egui::Key::Z, false),
            EditAction::Redo => key_event(egui::Key::Z, true),
        };
        self.pending_events.push(ev);
    }

    /// 接続先を変更して net::run_client に再接続を要求する。設定にも保存する。
    fn change_server_addr(&mut self, new_addr: String) {
        self.settings.server_addr = new_addr.clone();
        let _ = settings::save(&self.settings);
        let _ = self.addr_tx.try_send(new_addr.clone());
        self.conn_state = ConnState::Connecting { addr: new_addr };
    }

    /// Windows / Linux 向けのアプリ内メニューバー。macOS はネイティブメニューを使う。
    #[cfg(not(target_os = "macos"))]
    fn show_in_app_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル(File)", |ui| {
                    if ui.button("環境設定…").clicked() {
                        self.open_preferences();
                        ui.close_menu();
                    }
                    if ui.button("設定フォルダを開く").clicked() {
                        let _ = platform::open_in_file_manager(&platform::data_dir());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("終了").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("編集(Edit)", |ui| {
                    if ui.button("取り消す").clicked() {
                        self.queue_edit(EditAction::Undo);
                        ui.close_menu();
                    }
                    if ui.button("やり直す").clicked() {
                        self.queue_edit(EditAction::Redo);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("カット").clicked() {
                        self.queue_edit(EditAction::Cut);
                        ui.close_menu();
                    }
                    if ui.button("コピー").clicked() {
                        self.queue_edit(EditAction::Copy);
                        ui.close_menu();
                    }
                    if ui.button("ペースト").clicked() {
                        self.queue_edit(EditAction::Paste);
                        ui.close_menu();
                    }
                    if ui.button("すべてを選択").clicked() {
                        self.queue_edit(EditAction::SelectAll);
                        ui.close_menu();
                    }
                });

                ui.menu_button("ヘルプ(Help)", |ui| {
                    if ui.button("vlt-syslogd Console について").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// About(バージョン情報)ウィンドウ。Windows / Linux 用。
    #[cfg(not(target_os = "macos"))]
    fn show_about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        let mut keep_open = true;
        egui::Window::new(egui::RichText::new("vlt-syslogd Console について").size(11.0).strong())
            .collapsible(false)
            .resizable(false)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                ui.heading("vlt-syslogd Console");
                ui.label(format!("バージョン {}", env!("CARGO_PKG_VERSION")));
                ui.label("常駐サービスに接続して syslog を表示するビューア");
                ui.label("ライセンス: MIT");
                ui.add_space(8.0);
                if ui.button("閉じる").clicked() {
                    self.show_about = false;
                }
            });
        if !keep_open {
            self.show_about = false;
        }
    }

    fn open_preferences(&mut self) {
        self.show_preferences = true;
        self.pref_server_addr = self.settings.server_addr.clone();
        self.pref_control_addr = self.settings.control_addr.clone();
        self.pref_error = None;
        self.pref_saved = false;
    }

    /// 接続設定(server_addr / control_addr)を保存して再接続する。
    fn apply_connection_prefs(&mut self) {
        let server_addr = self.pref_server_addr.trim().to_string();
        let control_addr = self.pref_control_addr.trim().to_string();
        if server_addr.is_empty() || control_addr.is_empty() {
            self.pref_error = Some("アドレスを入力してください".to_string());
            self.pref_saved = false;
            return;
        }
        self.settings.control_addr = control_addr;
        if let Err(e) = settings::save(&self.settings) {
            self.pref_error = Some(format!("設定の保存に失敗しました: {e}"));
            self.pref_saved = false;
            return;
        }
        self.pref_error = None;
        self.pref_saved = true;
        // server_addr の保存と再接続はまとめて change_server_addr に任せる。
        self.addr_input = server_addr.clone();
        self.change_server_addr(server_addr);
    }

    /// 制御ポートからサーバの現在設定を取得して編集欄に反映する。
    fn fetch_server_config(&mut self) {
        match control::get_config(&self.settings.control_addr) {
            Ok(cfg) => {
                self.edit_bind_addr = cfg.server.bind_addr;
                self.edit_stream_addr = cfg.server.stream_addr;
                self.edit_log_level = cfg.logging.level;
                self.edit_max_size_mb = cfg.logging.max_size_mb.to_string();
                self.edit_keep_files = cfg.logging.keep_files.to_string();
                self.srv_cfg_loaded = true;
                self.srv_cfg_status = Some((true, "現在の設定を取得しました".to_string()));
            }
            Err(e) => {
                self.srv_cfg_status = Some((false, format!("取得に失敗: {e}")));
            }
        }
    }

    /// 編集欄の内容をサーバへ適用する。restart_required ならサービスを再起動する。
    fn apply_server_config(&mut self) {
        let max_size_mb = match self.edit_max_size_mb.trim().parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                self.srv_cfg_status =
                    Some((false, "ログ最大サイズ(MB)は数字で指定してください".to_string()));
                return;
            }
        };
        let keep_files = match self.edit_keep_files.trim().parse::<usize>() {
            Ok(v) => v,
            Err(_) => {
                self.srv_cfg_status =
                    Some((false, "保持ファイル数は数字で指定してください".to_string()));
                return;
            }
        };
        let cfg = control::ServerConfigDto {
            server: control::ServerSection {
                bind_addr: self.edit_bind_addr.trim().to_string(),
                stream_addr: self.edit_stream_addr.trim().to_string(),
            },
            logging: control::LoggingSection {
                level: self.edit_log_level.trim().to_string(),
                max_size_mb,
                keep_files,
            },
        };
        match control::set_config(&self.settings.control_addr, &cfg) {
            Ok(restart_required) => {
                if restart_required {
                    match service::restart() {
                        Ok(()) => {
                            self.srv_cfg_status = Some((
                                true,
                                "設定を保存し、サービスを再起動しました".to_string(),
                            ));
                        }
                        Err(e) => {
                            self.srv_cfg_status = Some((
                                false,
                                format!(
                                    "設定は保存しましたが、再起動に失敗しました: {e}（手動で再起動してください）"
                                ),
                            ));
                        }
                    }
                } else {
                    self.srv_cfg_status = Some((true, "設定を保存しました".to_string()));
                }
            }
            Err(e) => {
                self.srv_cfg_status = Some((false, format!("適用に失敗: {e}")));
            }
        }
    }

    /// 環境設定ウィンドウ。接続設定 + サーバ(syslog)設定の 2 セクション。
    fn show_preferences_window(&mut self, ctx: &egui::Context) {
        if !self.show_preferences {
            return;
        }
        let mut keep_open = true;
        egui::Window::new(egui::RichText::new("環境設定").size(11.0).strong())
            .collapsible(false)
            .resizable(false)
            .default_width(440.0)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                // --- 接続設定 ---
                ui.strong("接続設定");
                egui::Grid::new("conn_prefs_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("配信アドレス (host:port):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.pref_server_addr)
                                .desired_width(220.0),
                        );
                        ui.end_row();

                        ui.label("制御アドレス (host:port):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.pref_control_addr)
                                .desired_width(220.0),
                        );
                        ui.end_row();
                    });
                ui.add_space(4.0);
                if let Some(err) = &self.pref_error {
                    ui.colored_label(egui::Color32::from_rgb(240, 90, 90), err);
                } else if self.pref_saved {
                    ui.colored_label(egui::Color32::from_rgb(120, 200, 120), "保存しました");
                }
                ui.horizontal(|ui| {
                    if ui.button("保存して再接続").clicked() {
                        self.apply_connection_prefs();
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // --- サーバ(syslog)設定 ---
                ui.horizontal(|ui| {
                    ui.strong("サーバ設定 (syslog)");
                    if ui.button("現在値を取得").clicked() {
                        self.fetch_server_config();
                    }
                });
                ui.label(
                    egui::RichText::new(
                        "サービスの設定を変更します。適用するとサービスを再起動します。",
                    )
                    .weak(),
                );
                ui.add_space(4.0);

                ui.add_enabled_ui(self.srv_cfg_loaded, |ui| {
                    egui::Grid::new("srv_cfg_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("受信アドレス (bind_addr):");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_bind_addr)
                                    .desired_width(220.0),
                            );
                            ui.end_row();

                            ui.label("配信アドレス (stream_addr):");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_stream_addr)
                                    .desired_width(220.0),
                            );
                            ui.end_row();

                            ui.label("ログレベル:");
                            egui::ComboBox::from_id_source("log_level_combo")
                                .selected_text(&self.edit_log_level)
                                .show_ui(ui, |ui| {
                                    for lv in ["error", "warn", "info", "debug", "trace"] {
                                        ui.selectable_value(
                                            &mut self.edit_log_level,
                                            lv.to_string(),
                                            lv,
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("ログ最大サイズ (MB):");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_max_size_mb)
                                    .desired_width(90.0),
                            );
                            ui.end_row();

                            ui.label("保持ファイル数:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_keep_files)
                                    .desired_width(90.0),
                            );
                            ui.end_row();
                        });

                    ui.add_space(4.0);
                    if ui.button("サーバへ適用(再起動)").clicked() {
                        self.apply_server_config();
                    }
                });

                if let Some((ok, msg)) = &self.srv_cfg_status {
                    let color = if *ok {
                        egui::Color32::from_rgb(120, 200, 120)
                    } else {
                        egui::Color32::from_rgb(240, 90, 90)
                    };
                    ui.colored_label(color, msg);
                }

                ui.add_space(6.0);
                ui.separator();
                if ui.button("閉じる").clicked() {
                    self.show_preferences = false;
                }
            });
        if !keep_open {
            self.show_preferences = false;
        }
    }

    /// 接続状態に応じた上部バナー。接続中は出さない。接続試行中/切断時に表示する。
    fn show_conn_banner(&mut self, ctx: &egui::Context) {
        match self.conn_state.clone() {
            ConnState::Connected { .. } => {}
            ConnState::Connecting { addr } => {
                egui::TopBottomPanel::top("conn_banner").show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.label(format!("⏳ {addr} に接続しています…"));
                    ui.add_space(4.0);
                });
            }
            ConnState::Disconnected { addr, error } => {
                egui::TopBottomPanel::top("conn_banner").show(ctx, |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!("⚠ {addr} に接続できません"))
                            .color(egui::Color32::from_rgb(240, 90, 90))
                            .strong(),
                    );
                    ui.label(egui::RichText::new(format!("  ({error})")).weak());
                    ui.label(
                        egui::RichText::new(
                            "サービスが起動しているか、配信アドレスが正しいか確認してください。\
                             自動で再接続を試み続けます。",
                        )
                        .weak(),
                    );
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label("配信アドレス:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.addr_input).desired_width(180.0),
                        );
                        if ui.button("再接続").clicked() {
                            let addr = self.addr_input.trim().to_string();
                            if addr.is_empty() {
                                self.addr_error = Some("アドレスを入力してください".to_string());
                            } else {
                                self.addr_error = None;
                                self.change_server_addr(addr);
                            }
                        }
                    });
                    if let Some(err) = &self.addr_error {
                        ui.colored_label(egui::Color32::from_rgb(240, 90, 90), err);
                    }
                    ui.add_space(6.0);
                });
            }
        }
    }
}

impl eframe::App for ConsoleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 受信ログの取り込み。
        while let Ok(log) = self.msg_rx.try_recv() {
            self.logs.push(log);
            if self.logs.len() > MAX_LOG_ENTRIES {
                self.logs.remove(0);
            }
        }
        // 接続状態の更新。
        while let Ok(state) = self.state_rx.try_recv() {
            self.conn_state = state;
        }
        // サービス状態の更新(別スレッドのポーラから)。
        while let Ok(status) = self.svc_rx.try_recv() {
            self.service_status = status;
        }

        // 今フレームでフォーカスされているテキスト欄を覚えておく。
        if let Some(id) = ctx.memory(|m| m.focused()) {
            self.last_text_focus = Some(id);
        }

        // macOS のネイティブメニューから来た要求を反映する。
        #[cfg(target_os = "macos")]
        {
            for req in macos_menu::drain_requests() {
                match req {
                    macos_menu::MenuRequest::Preferences => self.open_preferences(),
                    macos_menu::MenuRequest::OpenLogs => {
                        let _ = platform::open_in_file_manager(&platform::data_dir());
                    }
                    macos_menu::MenuRequest::Copy => self.queue_edit(EditAction::Copy),
                    macos_menu::MenuRequest::Cut => self.queue_edit(EditAction::Cut),
                    macos_menu::MenuRequest::Paste => self.queue_edit(EditAction::Paste),
                    macos_menu::MenuRequest::SelectAll => self.queue_edit(EditAction::SelectAll),
                    macos_menu::MenuRequest::Undo => self.queue_edit(EditAction::Undo),
                    macos_menu::MenuRequest::Redo => self.queue_edit(EditAction::Redo),
                }
            }
        }

        // メニューから積まれた編集イベントを今フレームの入力へ注入する。
        if !self.pending_events.is_empty() {
            if let Some(id) = self.last_text_focus {
                ctx.memory_mut(|m| m.request_focus(id));
            }
            let evs = std::mem::take(&mut self.pending_events);
            ctx.input_mut(|i| i.events.extend(evs));
        }

        // メニューバー: macOS はネイティブ、Windows/Linux はアプリ内。
        #[cfg(not(target_os = "macos"))]
        {
            self.show_in_app_menu_bar(ctx);
            self.show_about_window(ctx);
        }

        self.show_conn_banner(ctx);
        self.show_preferences_window(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 vlt-syslogd Console");

                // 接続状態インジケータ。
                match &self.conn_state {
                    ConnState::Connected { addr } => {
                        ui.colored_label(
                            egui::Color32::from_rgb(120, 200, 120),
                            format!("● 受信中 ({addr})"),
                        );
                    }
                    ConnState::Connecting { .. } => {
                        ui.colored_label(egui::Color32::from_rgb(220, 200, 100), "◌ 接続中");
                    }
                    ConnState::Disconnected { .. } => {
                        ui.colored_label(egui::Color32::from_rgb(240, 90, 90), "○ 切断");
                    }
                }
                ui.separator();
                ui.label(format!("サービス: {}", self.service_status.label()));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                    if ui.button("🗑 Clear").clicked() {
                        self.logs.clear();
                    }
                    if ui.button("⚙ 設定").clicked() {
                        self.open_preferences();
                    }
                });
            });

            // サービス操作 + 直近の操作結果。
            ui.horizontal(|ui| {
                ui.label("サービス操作:");
                if ui.button("開始").clicked() {
                    self.svc_action_msg = Some(match service::start() {
                        Ok(()) => "開始を要求しました".to_string(),
                        Err(e) => format!("開始に失敗: {e}"),
                    });
                }
                if ui.button("停止").clicked() {
                    self.svc_action_msg = Some(match service::stop() {
                        Ok(()) => "停止を要求しました".to_string(),
                        Err(e) => format!("停止に失敗: {e}"),
                    });
                }
                if ui.button("再起動").clicked() {
                    self.svc_action_msg = Some(match service::restart() {
                        Ok(()) => "再起動を要求しました".to_string(),
                        Err(e) => format!("再起動に失敗: {e}"),
                    });
                }
                if let Some(msg) = &self.svc_action_msg {
                    ui.label(egui::RichText::new(msg).weak());
                }
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
                        || l.hostname
                            .as_ref()
                            .map_or(false, |h| h.to_lowercase().contains(&filter))
                })
                .collect();

            let row_height = 24.0;
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.auto_scroll)
                .show_rows(ui, row_height, filtered_logs.len(), |ui, row_range| {
                    egui::Grid::new("log_grid_console_v1")
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

        // 受信を画面に反映するため定期的に再描画する。
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

/// OS ごとに日本語(CJK)対応フォントを探して egui に登録する(Portable と同じ実装)。
fn load_cjk_font(fonts: &mut egui::FontDefinitions) -> bool {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\Fonts\\meiryo.ttc",
            "C:\\Windows\\Fonts\\YuGothM.ttc",
            "C:\\Windows\\Fonts\\msgothic.ttc",
            "C:\\Windows\\Fonts\\msyh.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Hiragino Sans GB.ttc",
        ]
    } else {
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
                egui::FontData::from_owned(font_data).tweak(egui::FontTweak {
                    y_offset_factor: 0.08,
                    ..Default::default()
                }),
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

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let settings = settings::load();

    // net::run_client 用チャネル(tokio mpsc)。
    let (msg_tx, msg_rx) = mpsc::channel::<SyslogMessage>(1024);
    let (state_tx, state_rx) = mpsc::channel::<ConnState>(16);
    let (addr_tx, addr_rx) = mpsc::channel::<String>(16);
    tokio::spawn(net::run_client(
        settings.server_addr.clone(),
        addr_rx,
        msg_tx,
        state_tx,
    ));

    // サービス状態ポーラ。status() はサブプロセス起動でブロッキングなので UI と分離する。
    let (svc_tx, svc_rx) = std::sync::mpsc::channel::<ServiceStatus>();
    std::thread::spawn(move || loop {
        if svc_tx.send(service::status()).is_err() {
            break; // GUI 終了。
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_title(WINDOW_TITLE);

    let icon_data = include_bytes!("../../icons/vlt_syslogd_icon.png");
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
        WINDOW_TITLE,
        native_options,
        Box::new(move |cc| {
            Box::new(ConsoleApp::new(
                cc, settings, msg_rx, state_rx, addr_tx, svc_rx,
            ))
        }),
    )
}
