#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;
mod platform;
mod settings;

#[cfg(target_os = "macos")]
mod macos_menu;

use eframe::egui;
use parser::{Severity, SyslogMessage};
use std::io::Write;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

/// socket マネージャ → GUI に送る bind 状態。GUI 側の表示状態としても使う。
#[derive(Clone)]
enum BindState {
    /// bind 試行中(初期状態 / 再試行直後)。
    Connecting,
    /// 待ち受け成功(成功アドレスはログに別途出すのでここでは持たない)。
    Bound,
    /// 待ち受け失敗。GUI でポート入力を促す。
    Failed { addr: String, error: String },
}

/// 編集メニュー(コピー/カット/ペースト等)の操作種別。
/// macOS のネイティブメニューと、Windows/Linux のアプリ内メニューの両方から使う。
/// 実体は「フォーカス中の egui テキスト編集へ該当イベントを注入する」ことで実現する。
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
/// egui のテキスト編集は全選択=⌘/Ctrl+A、取り消し=⌘/Ctrl+Z、やり直し=⌘/Ctrl+Shift+Z を
/// キーイベントとして解釈するので、メニューからの操作もこれを注入して再現する。
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

struct SyslogApp {
    logs: Vec<SyslogMessage>,
    receiver: mpsc::Receiver<SyslogMessage>,
    auto_scroll: bool,
    filter: String,
    log_file: Option<std::fs::File>,
    // bind 状態と、GUI からポートを選び直すための経路
    bind_state: BindState,
    bind_status_rx: mpsc::Receiver<BindState>,
    bind_tx: mpsc::Sender<u16>,
    port_input: String,
    port_error: Option<String>,
    // 環境設定ウィンドウ
    show_preferences: bool,
    pref_port: String,
    pref_log_dir: String,
    pref_error: Option<String>,
    pref_saved: bool,
    effective_log_dir: std::path::PathBuf,
    // メニュー(ネイティブ/アプリ内とも)から積まれた、次の描画で egui 入力へ注入する編集イベント
    pending_events: Vec<egui::Event>,
    // 直近でフォーカスされていたテキスト欄の id。
    // メニューバーをクリックするとウィンドウがキーを失い egui がフォーカスを落とすため、
    // 編集メニュー操作の注入直前にここへフォーカスを戻して、対象欄に確実に効かせる。
    last_text_focus: Option<egui::Id>,
    // About ダイアログの表示状態(Windows/Linux 用。macOS は OS 標準の About パネルを使う)
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    show_about: bool,
}

impl SyslogApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        receiver: mpsc::Receiver<SyslogMessage>,
        bind_status_rx: mpsc::Receiver<BindState>,
        bind_tx: mpsc::Sender<u16>,
    ) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // ユーザー設定(ポート / ログ保存先)を読み込む
        let cfg = settings::load();

        // ログファイルの準備(設定の上書きを考慮した実効ログディレクトリへ)
        let log_dir = settings::effective_log_dir(&cfg);
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join(format!(
            "syslog_{}.log",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ));
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        let mut fonts = egui::FontDefinitions::default();
        load_cjk_font(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        // CJK フォント(ヒラギノ等)は行高が高く、egui 既定サイズのままだと文字も、
        // ウィンドウのタイトルバー(高さ = タイトル文字の高さ + 枠の上下余白)も大きく見える。
        // 全体のテキストサイズを少し下げ、フローティングウィンドウの枠余白を薄くして、
        // 間延びしない見た目にする。
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
            // フローティングウィンドウ(環境設定 / About)のタイトルバー描画について。
            // egui 0.27 の Window はタイトル文字を「背景バーの中央」ではなく、
            // 枠線(border_padding)と x-height 補正(-1.5px)の分だけ上にずらして描く。
            // CJK タイトル(行高 ≈16.5px)+ 薄いバーだと、この固定オフセットが目立ち、
            // タイトルが上に寄って見える(中央より約 1.75px 上)。
            // egui 内部の描画位置は直接いじれないため、window_margin の上を下より厚く
            // して背景バー中央を引き下げ、テキスト中央と一致させる(実測でズレ ≈ +0.25px)。
            // 左右 8 / 上 6 / 下 2。これでバーは macOS 標準(約28pt)並みの薄さを保ちつつ
            // タイトルが垂直中央に揃う。
            style.spacing.window_margin = egui::Margin {
                left: 8.0,
                right: 8.0,
                top: 6.0,
                bottom: 2.0,
            };
            cc.egui_ctx.set_style(style);
        }

        // macOS では画面最上部のネイティブメニューバーを構築する。
        // NSApplication は eframe(winit)が初期化済みなので、この時点で安全に呼べる。
        #[cfg(target_os = "macos")]
        macos_menu::install();

        Self {
            logs: Vec::new(),
            receiver,
            auto_scroll: true,
            filter: String::new(),
            log_file,
            bind_state: BindState::Connecting,
            bind_status_rx,
            bind_tx,
            // bind 失敗時(使用中・特権など)の再試行先として、非特権ポートの定番をプリフィル
            port_input: "5514".to_string(),
            port_error: None,
            show_preferences: false,
            pref_port: cfg.bind_port.to_string(),
            pref_log_dir: log_dir.display().to_string(),
            pref_error: None,
            pref_saved: false,
            effective_log_dir: log_dir,
            pending_events: Vec::new(),
            last_text_focus: None,
            show_about: false,
        }
    }

    /// 編集メニューの操作を「次フレームで egui 入力へ注入するイベント」として積む。
    /// 注入は [`eframe::App::update`] の冒頭で行い、同フレーム内のフォーカス中ウィジェットに効かせる。
    fn queue_edit(&mut self, action: EditAction) {
        let ev = match action {
            EditAction::Copy => egui::Event::Copy,
            EditAction::Cut => egui::Event::Cut,
            EditAction::Paste => {
                // ペーストだけは OS クリップボードの中身が必要。読めなければ何もしない。
                match arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                    Ok(text) => egui::Event::Paste(text),
                    Err(_) => return,
                }
            }
            EditAction::SelectAll => key_event(egui::Key::A, false),
            EditAction::Undo => key_event(egui::Key::Z, false),
            EditAction::Redo => key_event(egui::Key::Z, true),
        };
        self.pending_events.push(ev);
    }

    /// Windows / Linux 向けのアプリ内メニューバー(ファイル / 編集 / ヘルプ)。
    /// macOS では画面最上部のネイティブメニュー([`macos_menu`])を使うので描画しない。
    #[cfg(not(target_os = "macos"))]
    fn show_in_app_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("ファイル(File)", |ui| {
                    if ui.button("環境設定…").clicked() {
                        self.show_preferences = true;
                        self.pref_error = None;
                        self.pref_saved = false;
                        ui.close_menu();
                    }
                    if ui.button("ログフォルダを開く").clicked() {
                        let _ = platform::open_in_file_manager(&self.effective_log_dir);
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
                    if ui.button("vlt-syslogd について").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// About(バージョン情報)ウィンドウ。Windows / Linux 用。
    /// macOS は OS 標準の About パネルを使うため描画しない。
    #[cfg(not(target_os = "macos"))]
    fn show_about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        let mut keep_open = true;
        egui::Window::new(egui::RichText::new("vlt-syslogd について").size(11.0).strong())
            .collapsible(false)
            .resizable(false)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                ui.heading("vlt-syslogd");
                ui.label(format!("バージョン {}", env!("CARGO_PKG_VERSION")));
                ui.label("シンプルな syslog ビューア");
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

    /// 環境設定の内容を適用する: config.toml に保存 → ログファイル再オープン → 再 bind。
    fn apply_preferences(&mut self) {
        let port = match self.pref_port.trim().parse::<u16>() {
            Ok(p) if p > 0 => p,
            _ => {
                self.pref_error = Some("ポートは 1〜65535 の数字で指定してください".to_string());
                self.pref_saved = false;
                return;
            }
        };

        let cfg = settings::Settings {
            bind_port: port,
            log_dir: self.pref_log_dir.trim().to_string(),
        };
        if let Err(e) = settings::save(&cfg) {
            self.pref_error = Some(format!("設定の保存に失敗しました: {}", e));
            self.pref_saved = false;
            return;
        }
        self.pref_error = None;
        self.pref_saved = true;

        // ログ保存先を新しいディレクトリへ切り替え(GUI 側のログファイルを開き直す)
        let new_log_dir = settings::effective_log_dir(&cfg);
        let _ = std::fs::create_dir_all(&new_log_dir);
        let log_path = new_log_dir.join(format!(
            "syslog_{}.log",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ));
        self.log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        self.pref_log_dir = new_log_dir.display().to_string();
        self.effective_log_dir = new_log_dir;

        // ポート変更と生ログ保存先を反映するため再 bind を要求する。
        // (socket マネージャは select! でこの要求を受け、現在のソケットを張り直す)
        let _ = self.bind_tx.try_send(port);
        self.bind_state = BindState::Connecting;
    }

    /// 環境設定ウィンドウを描画する。
    fn show_preferences_window(&mut self, ctx: &egui::Context) {
        if !self.show_preferences {
            return;
        }
        let mut keep_open = true;
        // egui の Window タイトルは既定で Heading(大きめ)になる。RichText でサイズを明示すると
        // タイトルだけ上書きできる(into_text_and_format で size が font_id を上書きするため)。
        // 本文と同じ 12px・太字にして、設定ダイアログらしい控えめなタイトルにする。
        egui::Window::new(egui::RichText::new("環境設定").size(11.0).strong())
            .collapsible(false)
            .resizable(false)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                egui::Grid::new("prefs_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("待ち受けポート:");
                        ui.add(egui::TextEdit::singleline(&mut self.pref_port).desired_width(90.0));
                        ui.end_row();

                        ui.label("ログ保存先:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pref_log_dir)
                                    .desired_width(300.0),
                            );
                            if ui.button("参照…").clicked()
                                && let Some(dir) = rfd::FileDialog::new().pick_folder()
                            {
                                self.pref_log_dir = dir.display().to_string();
                            }
                        });
                        ui.end_row();
                    });

                ui.add_space(6.0);
                if let Some(err) = &self.pref_error {
                    ui.colored_label(egui::Color32::from_rgb(240, 90, 90), err);
                } else if self.pref_saved {
                    ui.colored_label(egui::Color32::from_rgb(120, 200, 120), "保存しました");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("適用").clicked() {
                        self.apply_preferences();
                    }
                    if ui.button("閉じる").clicked() {
                        self.show_preferences = false;
                    }
                });
            });
        if !keep_open {
            self.show_preferences = false;
        }
    }

    /// bind 状態に応じた上部バナーを描画する。
    /// 成功時は何も出さない。失敗時はポート入力欄を出して再 bind を促す。
    fn show_bind_banner(&mut self, ctx: &egui::Context) {
        match self.bind_state.clone() {
            // 待ち受け中はバナーを出さない(本来のログ画面を邪魔しない)
            BindState::Bound => {}
            BindState::Connecting => {
                egui::TopBottomPanel::top("bind_banner").show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.label("⏳ 待ち受けを開始しています…");
                    ui.add_space(4.0);
                });
            }
            BindState::Failed { addr, error } => {
                egui::TopBottomPanel::top("bind_banner").show(ctx, |ui| {
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "⚠ {} で待ち受けを開始できませんでした",
                            addr
                        ))
                        .color(egui::Color32::from_rgb(240, 90, 90))
                        .strong(),
                    );
                    ui.label(egui::RichText::new(format!("  ({})", error)).weak());
                    ui.add_space(2.0);
                    ui.label(
                        "考えられる原因: 同じポートを別プロセスが使用中 / Linux で特権ポート(1024未満)に \
                         root 権限がない / 特定アドレスへの特権ポート bind に権限がない、など。\
                         (macOS では 0.0.0.0 への特権ポート bind は root 不要です。)\
                         下の欄で別のポートを指定して再試行できますが、送信側(機器・ルータ等)も\
                         そのポートに向ける必要があります。常駐で確実に動かしたい場合は Server 版もご利用ください。",
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("待ち受けポート:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.port_input).desired_width(80.0),
                        );
                        if ui.button("待ち受け開始").clicked() {
                            match self.port_input.trim().parse::<u16>() {
                                Ok(port) if port > 0 => {
                                    self.port_error = None;
                                    let _ = self.bind_tx.try_send(port);
                                    self.bind_state = BindState::Connecting;
                                }
                                _ => {
                                    self.port_error =
                                        Some("1〜65535 の数字を入力してください".to_string());
                                }
                            }
                        }
                    });
                    if let Some(err) = &self.port_error {
                        ui.label(
                            egui::RichText::new(err).color(egui::Color32::from_rgb(240, 90, 90)),
                        );
                    }
                    ui.add_space(6.0);
                });
            }
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

        // socket マネージャからの bind 状態を反映
        while let Ok(state) = self.bind_status_rx.try_recv() {
            self.bind_state = state;
        }

        // 今フレームでフォーカスされているテキスト欄を覚えておく(失われていなければ更新)。
        // メニュー操作でフォーカスが外れた後に、この id へ戻すために使う。
        if let Some(id) = ctx.memory(|m| m.focused()) {
            self.last_text_focus = Some(id);
        }

        // macOS のネイティブメニューから来た要求を回収して反映する。
        #[cfg(target_os = "macos")]
        {
            for req in macos_menu::drain_requests() {
                match req {
                    macos_menu::MenuRequest::Preferences => {
                        self.show_preferences = true;
                        self.pref_error = None;
                        self.pref_saved = false;
                    }
                    macos_menu::MenuRequest::OpenLogs => {
                        let _ = platform::open_in_file_manager(&self.effective_log_dir);
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

        // メニュー(ネイティブ/アプリ内)から積まれた編集イベントを、今フレームの入力へ注入する。
        // 描画(=テキスト編集ウィジェットが入力を読む)より前に積むことで同フレームで効く。
        if !self.pending_events.is_empty() {
            // メニューバー操作でフォーカスが外れている場合があるので、直前の欄へ戻してから注入する。
            // (ショートカット経由ではフォーカスは保持されており、この再要求は実質ノーオペ)
            if let Some(id) = self.last_text_focus {
                ctx.memory_mut(|m| m.request_focus(id));
            }
            let evs = std::mem::take(&mut self.pending_events);
            ctx.input_mut(|i| i.events.extend(evs));
        }

        // メニューバー: macOS はネイティブ上部バー、Windows/Linux はアプリ内バー。
        #[cfg(not(target_os = "macos"))]
        {
            self.show_in_app_menu_bar(ctx);
            self.show_about_window(ctx);
        }

        self.show_bind_banner(ctx);
        self.show_preferences_window(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🚀 vlt-syslogd");

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
            // CJK フォント（メイリオ等）は ASCII フォントと縦メトリクスが異なり、
            // egui が確保する行高に対してグリフが上に詰まって「上揃え」に見える。
            // y_offset_factor でグリフをフォントサイズの数 % だけ下げ、行の中央に寄せる。
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

/// GUI に表示する内部ステータス（起動・bind 失敗など）用のメッセージを生成する。
fn system_message(content: String, severity: Severity) -> SyslogMessage {
    SyslogMessage {
        severity,
        timestamp: chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
        hostname: None,
        tag: Some("vlt-syslogd".to_string()),
        content,
        raw: String::new(),
        encoding: "system".to_string(),
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    // 多重起動防止。
    // ロックファイルは必ず「書き込み可能な絶対パス」(OS の一時ディレクトリ)に作る。
    // single-instance は名前を相対パスとして cwd に作ろうとするが、Finder からの
    // ダブルクリック(LaunchServices)起動では cwd が読み取り専用の "/" になるため、
    // 相対パスだとロック作成に失敗してアプリが起動できない。
    // また、ロック生成に失敗しても多重起動防止はあくまで補助機能なので、
    // パニックさせず素通りして本体は起動させる。
    let lock_path = std::env::temp_dir().join("vlt_syslogd_singleton.lock");
    let instance = single_instance::SingleInstance::new(&lock_path.to_string_lossy());
    if let Ok(ref inst) = instance
        && !inst.is_single()
    {
        // 既に別インスタンスが起動済み。二重に立ち上げない。
        return Ok(());
    }

    let (tx, rx) = mpsc::channel(1000);
    // GUI ↔ socket マネージャ: 状態通知(マネージャ→GUI)とポート指定(GUI→マネージャ)
    let (status_tx, status_rx) = mpsc::channel::<BindState>(8);
    let (bind_tx, bind_rx) = mpsc::channel::<u16>(8);

    tokio::spawn(run_socket_manager(tx, bind_rx, status_tx));

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_title("vlt-syslogd");

    // アイコンは読み込めなくても致命的ではない（macOS の Dock は .app の icns を使う）
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
        "vlt-syslogd",
        native_options,
        Box::new(|cc| Box::new(SyslogApp::new(cc, rx, status_rx, bind_tx))),
    )
}

/// UDP の待ち受けを管理する。bind に失敗したら GUI からの新ポートを待って再試行する。
///
/// - 最初の試行先は環境変数 `VLT_SYSLOGD_BIND`(無ければ標準 514)。
/// - 成功したら状態を Bound にし、受信ループへ移る(以後は戻らない)。
/// - 失敗したら状態を Failed にし、GUI が選んだポートを受け取って再 bind する。
async fn run_socket_manager(
    tx: mpsc::Sender<SyslogMessage>,
    mut bind_rx: mpsc::Receiver<u16>,
    status_tx: mpsc::Sender<BindState>,
) {
    // 初期ポートは 環境変数 VLT_SYSLOGD_BIND > 設定ファイルの bind_port(既定 514)
    let mut addr = std::env::var("VLT_SYSLOGD_BIND")
        .unwrap_or_else(|_| format!("0.0.0.0:{}", settings::load().bind_port));

    loop {
        match UdpSocket::bind(&addr).await {
            Ok(socket) => {
                let _ = status_tx.send(BindState::Bound).await;
                let _ = tx
                    .send(system_message(
                        format!("Listening on {} (UDP)", addr),
                        Severity::Notice,
                    ))
                    .await;
                // 受信ループを回しつつ、環境設定からの再 bind 要求も待つ。
                // 要求が来たら recv_loop は drop され(ソケットを閉じ)、新アドレスで張り直す。
                tokio::select! {
                    _ = recv_loop(socket, &tx) => return,
                    maybe = bind_rx.recv() => match maybe {
                        Some(port) => {
                            addr = format!("0.0.0.0:{}", port);
                            continue;
                        }
                        None => return,
                    },
                }
            }
            Err(e) => {
                // bind 失敗の主因は OS で異なる:
                //   - Windows : ポート使用中 or ファイアウォール/権限
                //   - macOS   : Mojave 以降 0.0.0.0 への特権ポート bind は root 不要。
                //               失敗するのは「使用中」か「特定アドレスへの特権ポート bind」
                //   - Linux   : 1024 未満は root / CAP_NET_BIND_SERVICE が必要
                let hint = if cfg!(target_os = "windows") {
                    "別プロセスが使用中か、ファイアウォール/権限の問題です"
                } else if cfg!(target_os = "macos") {
                    "ポートが使用中の可能性があります(macOS は 0.0.0.0 への特権ポート bind に root 不要)。GUI で別ポートを指定できます"
                } else {
                    "ポート使用中、または特権ポート(1024未満)に root/CAP_NET_BIND_SERVICE が必要です。GUI で別ポート指定か sudo 起動を"
                };
                let _ = status_tx
                    .send(BindState::Failed {
                        addr: addr.clone(),
                        error: e.to_string(),
                    })
                    .await;
                let _ = tx
                    .send(system_message(
                        format!("Failed to bind {}: {} — {}", addr, e, hint),
                        Severity::Error,
                    ))
                    .await;
                eprintln!("Failed to bind UDP socket on {}: {} — {}", addr, e, hint);

                // GUI がポートを指定してくるまで待つ。GUI が閉じてチャネルが切れたら終了。
                match bind_rx.recv().await {
                    Some(port) => addr = format!("0.0.0.0:{}", port),
                    None => return,
                }
            }
        }
    }
}

/// 待ち受け成功後の受信ループ。受け取ったパケットを生ログに残し、パースして GUI へ送る。
async fn recv_loop(socket: UdpSocket, tx: &mpsc::Sender<SyslogMessage>) {
    // デバッグ用生データ保存ファイルの準備(保存先は設定の実効ログディレクトリに従う)
    let log_dir = settings::effective_log_dir(&settings::load());
    let _ = std::fs::create_dir_all(&log_dir);
    let mut debug_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("debug_raw.log"))
        .ok();

    let mut buf = [0u8; 8192];
    loop {
        if let Ok((size, src)) = socket.recv_from(&mut buf).await {
            let raw_msg = &buf[..size];

            // 生データのHEXダンプを保存
            if let Some(ref mut file) = debug_file {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let line = format!("[{}] [src:{}] raw:{}\n", timestamp, src, hex::encode(raw_msg));
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }

            let parsed = parser::parse_syslog(raw_msg);
            let _ = tx.send(parsed).await;
        }
    }
}
