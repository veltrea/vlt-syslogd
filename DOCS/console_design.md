# vlt-syslogd Console 完成設計書

- 作成日: 2026-06-29
- 対象: `Console`（GUI フロントエンド）クレートを「ビルド可能・実用」にするための実装設計
- 前提レポート: [console_status_report.md](console_status_report.md)（現状把握。本書はその「不足物」を実装可能なレベルに具体化したもの）

---

## 0. 検証済みの前提（設計の土台）

すべて実ファイルを行単位で確認済み。

- **既存部品（実装済み・流用する）**: `Console/src/` の `parser.rs`(型) / `settings.rs`(server_addr 永続化) / `platform.rs`(保存先) / `net.rs`(TCP クライアント `run_client` + テスト) / `service.rs`(OS サービス監視) / `macos_menu.rs`(ネイティブメニュー)。
- **欠落（本書で設計する）**: `Console/Cargo.toml`、`Console/src/main.rs`、ルート workspace への登録、（設定変更機能）Server 制御ポート。
- **流用元 `Portable/src/main.rs`（検証済みの実構成）**: `#[tokio::main] async fn main` → `tokio::spawn(run_socket_manager(tx, bind_rx, status_tx))` → `recv_loop(UdpSocket)` が受信して **`mpsc::Sender<SyslogMessage>`** で GUI へ流す。GUI(`SyslogApp`)は毎フレーム `mpsc` を drain して自前バッファに溜め、`egui` で描画。`SharedState` は**使っていない**（mpsc 方式）。
- **重要な一致点**: `Console/src/net.rs::run_client(initial_addr, addr_rx, msg_tx, state_tx)` の **チャネル型は `mpsc::Sender<SyslogMessage>` / `mpsc::Sender<ConnState>` / `mpsc::Receiver<String>`**。これは Portable の受信→GUI インターフェースと同型。
  → **Console は「Portable の `recv_loop`(UDP 自前待ち受け) を `net.rs::run_client`(TCP で Server へ接続) に差し替えるだけ」で GUI を流用できる。**
- **シリアライズ互換（確認済み）**: `Server/src/parser.rs::SyslogMessage` と `Console/src/parser.rs::SyslogMessage` はフィールド（`severity,timestamp,hostname:Option,tag:Option,content,raw,encoding`）・`Severity`(0..7) とも完全一致。JSON Lines はそのままデシリアライズ可能。

### データフロー（完成形）
```
[syslog機器] --UDP:514--> [Server run_syslog_server] --parse/JSON--> broadcast
                                                                        |
                                                  [Server run_stream_server TCP:5141]
                                                                        | JSON Lines(一方向)
                                                                        v
                          [Console net.rs::run_client] --mpsc<SyslogMessage>--> [ConsoleApp(egui)]
                                        ^                                              |
                                        | mpsc<String>(接続先変更)   表示/フィルタ/接続状態バナー
                                        +------------------(Preferences で server_addr 変更)-------+

(Phase2 追加)
  [ConsoleApp 設定画面] --control.rs 同期TCP--> [Server run_control_server TCP:5142]
        get_config / set_config(JSONL 1行)        config.toml 読み書き → service::restart() で反映
```

---

## 1. 全体方針とフェーズ分け

- **Phase 1（最小実用＝「TCPでログ表示」）**: `D-1` Cargo.toml + `D-2` main.rs + `D-3` workspace 登録。これだけで「Server に接続して syslog を画面表示・フィルタ・接続状態表示・サービス状態表示」が動く。
- **Phase 2（「syslog 設定変更」）**: `D-4` Server 制御ポート + Console 側 control.rs + 設定画面拡張。元要件の「syslog の設定もできる」を満たす。設計判断が要るため独立フェーズにする。

---

## 2. D-1: `Console/Cargo.toml`（新規作成）

`Portable/Cargo.toml` をベースに、Console で実際に使う依存だけにする。

```toml
[package]
name = "vlt-syslogd-console"
version = "0.3.0"
edition = "2024"
license = "MIT"
authors = ["veltrea <veltrea@outlook.com>"]

[dependencies]
eframe = "0.27"
egui = "0.27"
tokio = { version = "1", features = ["full"] }   # net.rs::run_client が使用
serde = { version = "1.0", features = ["derive"] } # parser/settings
serde_json = "1.0"                                # net.rs のデシリアライズ
toml = "0.8"                                      # settings の保存
arboard = "3"                                     # 編集メニューの「ペースト」用クリップボード

# macOS ネイティブメニュー(macos_menu.rs)
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

[build-dependencies]
winres = "0.1"   # build.rs(Windows アイコン)

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(feature, values("cargo-clippy"))'] }
```

判断（要確認、§9）:
- `chrono` … Console は受信メッセージの `timestamp: String` をそのまま表示するだけなら**不要**。Portable は持つが Console には入れない方針で良いか。
- `rfd` … フォルダ選択ダイアログ。Console は `platform::open_in_file_manager`（`Command` 直叩き）でフォルダを開くので Phase1 では**不要**。
- `single-instance` … 二重起動防止。Console は複数起動を許してもよい性質（読むだけ）。**任意**。Portable に合わせるなら追加。

---

## 3. D-2: `Console/src/main.rs`（新規作成・本体）

`Portable/src/main.rs` の `SyslogApp` を `ConsoleApp` に作り替える。**受信が UDP 自前待ち受け→TCP クライアント接続に変わる以外、GUI 骨格はほぼそのまま流用**。

### 3.1 モジュール宣言
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod parser; mod settings; mod platform; mod net; mod service; mod macos_menu;
use eframe::egui;
use tokio::sync::mpsc;
use parser::{SyslogMessage, Severity};
use net::ConnState;
use service::ServiceStatus;
```

### 3.2 `ConsoleApp` 構造体
```rust
const MAX_LOG_ENTRIES: usize = 10_000;
const WINDOW_TITLE: &str = "vlt-syslogd Console";

struct ConsoleApp {
    // 受信(net.rs から)
    msg_rx: mpsc::Receiver<SyslogMessage>,
    state_rx: mpsc::Receiver<ConnState>,
    addr_tx: mpsc::Sender<String>,      // 接続先変更を run_client へ通知
    // サービス状態(別スレッドのポーラから)
    svc_rx: std::sync::mpsc::Receiver<ServiceStatus>,
    // 表示状態
    messages: std::collections::VecDeque<SyslogMessage>,
    conn_state: ConnState,
    service_status: ServiceStatus,
    settings: settings::Settings,
    filter: String,
    auto_scroll: bool,
    show_preferences: bool,
    show_about: bool,
}
```

### 3.3 `update()` の流れ（毎フレーム）
1. **受信の取り込み**: `while let Ok(m) = self.msg_rx.try_recv() { push して MAX 超過分を pop_front }`。
2. **接続状態の更新**: `while let Ok(s) = self.state_rx.try_recv() { self.conn_state = s }`。
3. **サービス状態の更新**: `while let Ok(s) = self.svc_rx.try_recv() { self.service_status = s }`。
4. **macOS メニュー要求**: `#[cfg(target_os="macos")] for req in macos_menu::drain_requests() { ... }`
   - `MenuRequest::Preferences => show_preferences = true`
   - `MenuRequest::OpenLogs => { let _ = platform::open_in_file_manager(&platform::data_dir()); }`
   - `Copy/Cut/Paste/SelectAll/Undo/Redo` … Portable の `queue_edit`/編集処理を流用。
5. **TopPanel**（上部バー）:
   - タイトル「vlt-syslogd Console」
   - 接続先表示: `format!("Server: {}", self.settings.server_addr)`
   - 接続状態バナー（`ConnState` で色分け）:
     - `Connected` → 緑「● 受信中」
     - `Connecting` → 黄「◌ 接続中…」
     - `Disconnected{error}` → 赤「○ 切断 ({error})」
   - サービス状態: `ui.label(self.service_status.label())`（🟢稼働中/⚪停止中/❌未インストール/❓不明）+（任意で「開始/停止/再起動」ボタン → `service::start()/stop()/restart()`）
   - Filter 欄 / Auto-scroll チェック / Clear ボタン（`self.messages.clear()`） / Preferences ボタン
6. **CentralPanel**（ログ本体）: Portable と同じ。`filter` で `content/tag/hostname` を小文字一致フィルタ → `ScrollArea::vertical().stick_to_bottom(auto_scroll)` → 各行 `severity.color()` で着色して `timestamp / [severity] / tag: / content` を表示。
7. **Preferences ウィンドウ**（`show_preferences`）:
   - 「接続先サーバ (host:port)」= `self.settings.server_addr` をテキスト編集。
   - 「保存して再接続」ボタン → `settings::save(&self.settings)` → `self.addr_tx.try_send(self.settings.server_addr.clone())`（run_client が `addr_rx` で受けて張り直す）。
   - （Phase2）この下に「サーバ設定（syslog）」セクションを追加（§5）。
8. **About ウィンドウ**（`show_about`）: 名称・著作表記（© 2026 veltrea）。
9. `ctx.request_repaint_after(Duration::from_millis(200));`（受信を拾うため定期再描画）。

### 3.4 `main()`（エントリポイント）
```rust
#[tokio::main]
async fn main() -> eframe::Result<()> {
    let settings = settings::load();

    // run_client 用チャネル(tokio mpsc)
    let (msg_tx, msg_rx)     = mpsc::channel::<SyslogMessage>(1024);
    let (state_tx, state_rx) = mpsc::channel::<ConnState>(16);
    let (addr_tx, addr_rx)   = mpsc::channel::<String>(16);
    tokio::spawn(net::run_client(settings.server_addr.clone(), addr_rx, msg_tx, state_tx));

    // サービス状態ポーラ(別スレッド。status() はサブプロセス起動でブロッキングなので UI と分離)
    let (svc_tx, svc_rx) = std::sync::mpsc::channel::<ServiceStatus>();
    std::thread::spawn(move || loop {
        let _ = svc_tx.send(service::status());
        std::thread::sleep(std::time::Duration::from_secs(3));
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        WINDOW_TITLE, options,
        Box::new(move |cc| {
            setup_japanese_fonts(&cc.egui_ctx);          // Portable の load_cjk_font を流用
            #[cfg(target_os = "macos")] macos_menu::install();
            Box::new(ConsoleApp::new(msg_rx, state_rx, addr_tx, svc_rx, settings))
        }),
    )
}
```
- 日本語フォント: Portable `main.rs` の `load_cjk_font` をそのまま移植（OS のフォントファイルを読み egui に登録。追加クレート不要）。
- 編集メニュー（コピー/ペースト等）: Portable の `key_event`/`queue_edit` 周りを移植（`arboard` でクリップボード）。

---

## 4. D-3: ルート `Cargo.toml` への登録

```toml
[workspace]
resolver = "2"
members = ["Portable", "Server", "Console"]
```
- これで `cargo build -p vlt-syslogd-console` が有効になる。

---

## 5. D-4: 設定変更機能（Phase 2）

「コンソールから syslog の設定（受信アドレス等）を変える」。**現状は Console 側にも Server 側にも経路が無い**ので両方を新設する。

### 5.1 Server 側：制御ポートを追加
- **設定**: `Server/src/config.rs` の `ServerConfig` に追加（既存 config.toml 互換のため `serde default`）:
  ```rust
  #[serde(default = "default_control_addr")]
  pub control_addr: String,   // 既定 127.0.0.1:5142
  ```
- **リスナー**: `Server/src/main.rs` に `run_control_server(addr)` を新設し、`run_syslog_server` から `tokio::spawn` で起動（`run_stream_server` と同じ並べ方）。
- **方針**: `stream_addr` と同じく **ループバック限定・認証なし**（外部公開しない前提を踏襲）。bind を 0.0.0.0 にする運用は想定しない。
- **プロトコル（行区切り JSON / JSONL）** — 1 行リクエスト → 1 行レスポンス。**Content-Length は付けない**:
  ```
  → {"cmd":"get_config"}\n
  ← {"ok":true,"config":{"bind_addr":"0.0.0.0:514","stream_addr":"127.0.0.1:5141","logging":{"level":"info","max_size_mb":10,"keep_files":7}}}\n

  → {"cmd":"set_config","config":{...同形...}}\n
  ← {"ok":true,"restart_required":true}\n        // 失敗時 {"ok":false,"error":"..."}
  ```
- **実装が単純**: `get_config` は `config::load_config()` で config.toml を読んで返すだけ。`set_config` は受信値を検証して `toml::to_string_pretty` で config.toml に**書き込むだけ**。動作中のサーバとの共有メモリは不要。
- **反映方式（推奨）**: `set_config` は**永続化のみ**を行い `restart_required:true` を返す。実際の反映（UDP ソケットの再 bind 等）は **サービス再起動**で行う。Console は `service::restart()` を持っているのでこれを呼ぶ。
  - 理由: 動作中プロセス内でのホットリロード（bind の張り直し・ログローテーション設定の再適用）は状態管理が複雑でリスクが高い。再起動方式なら既存資産（service.rs）だけで完結し、確実。
  - ホットリロードは将来拡張（§9 の判断事項）。

### 5.2 Console 側
- **新規モジュール `Console/src/control.rs`**: 同期 round-trip（`std::net::TcpStream` で 1 行送って 1 行受ける）。net.rs（非同期ストリーム受信）とは責務が違うので分ける。
  ```rust
  pub fn get_config(control_addr: &str) -> std::io::Result<ServerConfigDto>;
  pub fn set_config(control_addr: &str, cfg: &ServerConfigDto) -> std::io::Result<SetResult>;
  ```
  - `ServerConfigDto` は Server の `config` 形に対応する serde 構造体。
- **設定の追加**: `Console/src/settings.rs::Settings` に `control_addr: String`（既定 `127.0.0.1:5142`）を追加（`#[serde(default)]` で既存 config.toml 互換）。
- **Preferences ウィンドウ拡張**: 「サーバ設定 (syslog)」セクション。
  - 開いたとき `control::get_config` で現在値を取得し編集欄へ。
  - 編集対象（最小）: `bind_addr`（受信アドレス）, `logging.level`。
  - 「サーバへ適用」ボタン → `control::set_config` → `restart_required` なら確認の上 `service::restart()`。
  - 取得/適用の失敗は赤字で表示（サービス停止中・制御ポート未対応など）。

---

## 6. ファイル別 作業チェックリスト

| # | ファイル | 操作 | フェーズ | 内容 |
|---|---|---|---|---|
| 1 | `Console/Cargo.toml` | 新規 | P1 | §2 |
| 2 | `Console/src/main.rs` | 新規 | P1 | §3（ConsoleApp / update / main / フォント / 編集メニュー） |
| 3 | ルート `Cargo.toml` | 編集 | P1 | members に `"Console"` 追加 |
| 4 | `Console/src/settings.rs` | 編集 | P2 | `control_addr` 追加 |
| 5 | `Console/src/control.rs` | 新規 | P2 | 制御 round-trip クライアント |
| 6 | `Server/src/config.rs` | 編集 | P2 | `control_addr` + `default_control_addr()` |
| 7 | `Server/src/main.rs` | 編集 | P2 | `run_control_server` 追加 + spawn |

既存の `net.rs / parser.rs / platform.rs / service.rs / macos_menu.rs / build.rs` は**変更不要**（そのまま `mod` 宣言して使う）。

---

## 7. 検証計画

- **P1-1 ビルド**: `cargo build -p vlt-syslogd-console`（macOS）。`cargo test -p vlt-syslogd-console`（net.rs の既存テスト2本が緑）。
- **P1-2 結線テスト（GUI 抜き）**: Server を起動 → `nc 127.0.0.1 5141` で JSON Lines が流れることを確認。次に Console を起動し、Server に `logger`/UDP で送ったログが画面に出ることを確認。
- **P1-3 接続状態**: Server 停止中に Console 起動 → 赤「切断」。Server 起動 → 緑「受信中」に自動遷移（run_client の再接続）。
- **P1-4 サービス状態**: `service::status()` が macOS/Windows/Linux で正しいラベルを返すか（リモート VM で確認可）。
- **P2-1 制御**: Console から `get_config` 表示 → `bind_addr` 変更 → `set_config` → `service::restart()` → 新ポートで受信できることを確認。

GUI の見た目確認はビルド成功では代替できない（実起動が要る）。リモート/ローカルで実起動して目視するか、できない場合はその旨を明示する。

---

## 8. リスク・注意

- **eframe/egui 0.27 と edition 2024 の組合せ**: Portable が同構成でビルドできている実績があるので踏襲。バージョンは Portable と完全一致させる（差異でビルドが割れないように）。
- **`#[tokio::main]` と `eframe::run_native` の同居**: Portable が実証済みのパターン。multi-thread ランタイム上で `run_client` をワーカースレッドで回し、メインスレッドは eframe がブロックする。
- **編集メニュー（macOS）の移植**: `macos_menu.rs` の `MenuRequest` は `Copy/Cut/Paste/SelectAll/Undo/Redo` を含む。Portable の `queue_edit`/`key_event` 実装をそのまま持ってくる必要がある（独自実装し直さない）。
- **設定の後方互換**: `Settings`/`ServerConfig` への追加フィールドは必ず `#[serde(default)]` 系で補い、既存 config.toml を壊さない（Server は既に `stream_addr` でこの作法を採っている）。

---

## 9. 設計判断（2026-06-29 決定済み・実装反映済み）

1. **設定反映方式**: 「config.toml 書き換え + `service::restart()`」を採用（決定）。`set_config` は永続化のみで `restart_required:true` を返し、Console が `service::restart()` を呼ぶ。
2. **制御ポート**: `127.0.0.1:5142`（loopback・認証なし）を採用（決定）。
3. **設定変更の対象範囲**: 全項目（`bind_addr`/`stream_addr`/`logging.level`/`max_size_mb`/`keep_files`）を採用（決定）。
4. **Console の依存**: `chrono`/`rfd`/`single-instance` は不採用（未使用のため省略）。`image`(アイコン)は採用。
5. **Console の単一起動制限**: 設けない（複数起動を許可）。
6. **「コンソール機能」の解釈**: ログ表示 GUI + 接続/サービス/設定操作。コマンド入力欄は要件外。

### 実装ステータス（Phase 1 + Phase 2 とも完了）

| 成果物 | 状態 |
|---|---|
| `Console/Cargo.toml` | ✅ 作成 |
| `Console/src/main.rs`（ConsoleApp/接続バナー/サービス操作/設定画面） | ✅ 作成 |
| `Console/src/control.rs`（制御 round-trip クライアント） | ✅ 作成 |
| `Console/src/settings.rs`（`control_addr` 追加） | ✅ 編集 |
| ルート `Cargo.toml`（members に Console） | ✅ 編集 |
| `Server/src/config.rs`（`control_addr` + `save_config`） | ✅ 編集 |
| `Server/src/main.rs`（`run_control_server` + `handle_control`） | ✅ 編集 |

**検証結果（2026-06-29）**:
- `cargo check --workspace` 警告ゼロ / `cargo test -p vlt-syslogd-console` 2/2 合格。
- 実起動結線テスト（Server を高ポートで起動し独立クライアントで検証）: `get_config` が現在設定を返す / `set_config` が config.toml へ永続化し `restart_required:true` を返す / 異常系（unknown cmd・不正JSON）が `{"ok":false,...}` を返す / UDP→TCP(5141) のストリーム配信が JSON Lines で流れ日本語 UTF-8 が正しく復元される、をすべて確認。
- GUI の目視確認のみ未実施（表示環境が必要なため）。ロジック・通信・ビルドは検証済み。
