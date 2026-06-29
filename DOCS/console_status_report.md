# vlt-syslogd Console（GUI フロントエンド）実装状況レポート

- 作成日: 2026-06-29
- 対象リポジトリ: `/Volumes/2TB_USB/dev/vlt-syslogd-private`
- 目的: 「Console（フロントエンド）が Server（常駐サービス）と TCP で通信し、syslog データを画面表示・コンソール機能・syslog 設定変更を行う」アプリの完成度を、**第三者が再検証できる形**で記録する。
- 検証方法: 各ソースを Read で直接読み、行番号を明示。構造は `ls` / `test -f` / `wc -l` / `grep -n` で確認。本レポートの末尾に再検証コマンドを添付する。

---

## 0. 重要な自己訂正（このレポートの信頼性に関わるので最初に書く）

本調査の前半、私（Claude）は「サブエージェントの調査報告はハルシネーションを含む」と断定したが、**この断定自体が誤りだった**。

- 私は当初、`Console/src/net.rs` の中身を「`ConsoleClient` / `get_config_raw` / `set_config_raw` / `round_trip` / `run_tcp` / `run_udp` / 自前で syslog を待ち受ける受信器」と報告した。
- しかし**実ファイルを行単位で読み直すと、これらの識別子は net.rs に一つも存在しない**。実際の net.rs は `run_client` / `ConnState` / `SyslogMessage` を使い、Server の配信ポートへ接続する TCP **クライアント**である（後述、行番号付き）。
- `service.rs` についても同様で、私は当初「`SharedState` / `LogEntry` / リングバッファ / `ConsoleConfig`」と報告したが、**実ファイルにこれらは存在せず**、実体は OS 別のサービス状態監視（`ServiceStatus` / `status` / `start` / `stop` / `restart`）だった。
- つまり「3つの設計が分裂して混在している」という私の結論は、**私の誤読（存在しないコードを根拠にした誤り）**であって、コードベースの実態ではない。実態は一貫した単一設計である。

ユーザーの指摘どおり、ターミナル表示上の行の重複（echo）も観測されたが、それとは別に、私の最初の `Read`/`grep` の解釈が実ファイルと食い違っていた。原因を完全には特定できないが、**正典は「現在の実ファイルの内容」**であり、本レポートは実ファイルの再読のみを根拠とする。サブエージェントの報告（net.rs=`run_client`/5141/JSON Lines/テスト2本、service.rs=OS サービス管理 等）は**実ファイルと一致しており正確**だった。

---

## 1. ワークスペース構成

`Cargo.toml`（リポジトリルート, 全12行）:

```
9   [workspace]
10  resolver = "2"
11  members = ["Portable", "Server"]
```

- メンバーは **`Portable` と `Server` の2つのみ**。
- **`Console` はワークスペースに未登録**（→ 現状 `cargo build` の対象に含まれない）。

---

## 2. Console ディレクトリの構造

`ls` / `test -f` / `wc -l` による確認結果:

| パス | 状態 |
|---|---|
| `Console/Cargo.toml` | **MISSING（存在しない）** |
| `Console/src/main.rs` | **MISSING（存在しない）** |
| `Console/build.rs` | 13 行 |
| `Console/src/macos_menu.rs` | 261 行 |
| `Console/src/net.rs` | 188 行 |
| `Console/src/parser.rs` | 42 行 |
| `Console/src/platform.rs` | 73 行 |
| `Console/src/service.rs` | 234 行 |
| `Console/src/settings.rs` | 42 行 |

→ **クレートとして未成立**。`Cargo.toml`（パッケージ定義・依存関係）と、GUI エントリポイントである `main.rs` の両方が欠落している。部品モジュールのみが置かれた状態。

---

## 3. Console 各ファイルの実体（行番号付き）

### 3.1 `Console/src/parser.rs`（42行）— 受信メッセージ型
- `pub enum Severity`（行 9–18）: `Emergency=0 … Debug=7`。
- `Severity::color()`（行 22–30）: 重大度別の表示色 RGB。
- `pub struct SyslogMessage`（行 33–42）: `severity, timestamp, hostname: Option<String>, tag: Option<String>, content, raw, encoding`。
- 冒頭コメント（行 1–6）: 「このクレートは syslog をパースしない。Server が JSON Lines で送る `SyslogMessage` をデシリアライズするだけ。送信側 `Server/src/parser.rs` とフィールド/enum を一致させること」。

### 3.2 `Console/src/settings.rs`（42行）— ローカル設定の永続化
- `pub struct Settings { pub server_addr: String }`（行 9–15）。
- `Default`（行 17–23）: `server_addr = "127.0.0.1:5141"`（= Server の既定 `stream_addr` と同じ）。
- `pub fn load()`（行 26–32） / `pub fn save()`（行 35–42）: config.toml の読み書き。
- **保持するのは「接続先サービスの TCP 配信アドレス」のみ**（受信ログ本体は Server 側の責務、とコメント明記）。

### 3.3 `Console/src/platform.rs`（73行）— OS 別データ保存先
- `const APP = "vlt-syslogd-console"`（行 14）。
- `pub fn data_dir()`（行 17–24）: 環境変数 `VLT_SYSLOGD_CONSOLE_DATA_DIR` 優先、なければ OS 標準領域。
- `pub fn config_path()`（行 27–29）: `<data_dir>/config.toml`。
- `app_data_dir()`（行 35–53）: Windows `%APPDATA%`、macOS `~/Library/Application Support`、Linux `$XDG_DATA_HOME` or `~/.local/share`。
- `pub fn open_in_file_manager()`（行 59–73）: Finder/エクスプローラ/`xdg-open` 連携。

### 3.4 `Console/src/net.rs`（188行）— Server への TCP クライアント
- 冒頭コメント（行 1–10）: 「常駐サービスの配信ポート（既定 127.0.0.1:5141）へ接続し、JSON Lines を `SyslogMessage` にデシリアライズして GUI へ渡す。切断・失敗時は自動再接続。GUI とは 3 本のチャネル（msg_tx / state_tx / addr_rx）でやり取り」。
- `pub enum ConnState`（行 20–27）: `Connecting / Connected / Disconnected{error}`。GUI のバナー表示用。
- `const RECONNECT_DELAY = 2s`（行 30）。
- `pub async fn run_client(initial_addr, addr_rx, msg_tx, state_tx)`（行 33–94）: 常駐ループ。`TcpStream::connect`→`BufReader::lines()` で1行ずつ受信→`serde_json::from_str::<SyslogMessage>` 成功時のみ `msg_tx` へ送信（行 59–61）。`tokio::select!` で受信と接続先変更要求を同時に待つ。EOF/エラーで再接続（行 65）。
- テスト2本（`#[cfg(test)]`, 行 96–188）:
  - `connects_and_decodes_json_lines`（行 105–161）: ダミーサーバを立て、日本語含む JSON 2行を `SyslogMessage` に復元できることを検証。
  - `reports_disconnected_when_no_server`（行 164–187）: 接続先不在で `Disconnected` を通知することを検証。
- **存在しないもの（前回の私の誤報の打ち消し）**: `ConsoleClient`、`get_config_raw`、`set_config_raw`、`round_trip`、`run_tcp`、`run_udp`、`spawn_network_thread`。net.rs は **自前で待ち受けず、接続するだけ**。設定 get/set のコードも net.rs には無い。

### 3.5 `Console/src/service.rs`（234行）— OS 別サービス状態の監視/制御
- `pub enum ServiceStatus`（行 19–28）: `Running / Stopped / NotInstalled / Unknown(String)`。`label()`（行 32–39）で GUI 用ラベル。
- サービス識別子の定数（行 42–49）: Windows `vlt-syslogd-srv`、macOS plist `/Library/LaunchDaemons/com.veltrea.vlt-syslogd-srv.plist`・label `com.veltrea.vlt-syslogd-srv`、Linux unit `vlt-syslogd-srv.service`。
- Windows（行 52–103）: `status()` は `sc query`、`start/stop` は PowerShell `Start-Process -Verb RunAs`（UAC 昇格）。
- macOS（行 106–158）: `status()` は plist 存在 + `launchctl list`、`start/stop` は osascript「管理者として実行」で `launchctl load/unload`。
- Linux（行 161–225）: `status()` は `systemctl is-active`（+ `is-enabled` 補足）、`start/stop` は `pkexec`/`sudo` 経由 `systemctl`。
- 共通 `pub fn restart()`（行 231–234）: stop→start。

### 3.6 `Console/src/macos_menu.rs`（261行）— macOS ネイティブメニュー
- `pub enum MenuRequest`（行 39）、`pub fn drain_requests()`（行 66）、`pub fn install()`（行 183）。
- AppKit（cocoa/objc）でネイティブメニューバーを構築し、押下イベントをキュー経由で egui ループへ橋渡しする設計。

### 3.7 `Console/build.rs`（13行）
- Windows ビルド時のみ winres でアイコン/バージョン情報を埋め込む。ProductName=`vlt-syslogd-console`。

---

## 4. Server 側の通信モデル（`Server/src/main.rs`, `Server/src/config.rs`）

### 4.1 `Server/src/main.rs` の関数一覧（`grep -n` で確認）
```
38   fn main()
103  fn syslog_service_main(...)
115  fn setup_panic_hook()
130  fn init_logger(...)
160  fn run_service(...)
215  async fn run_syslog_server(...)
264  async fn run_stream_server(...)
```

### 4.2 `run_syslog_server`（行 215–257）— syslog 受信本体
- `UdpSocket::bind(config.server.bind_addr)`（行 216–217、既定 `0.0.0.0:514`）で UDP 受信。
- `broadcast::channel::<String>(1024)` を作り（行 224）、配信タスク `run_stream_server` を spawn（行 231–235）。
- 受信ループ（行 239–256）: `parser::parse_syslog` でパース→ログ記録→`serde_json::to_string` した JSON を `stream_tx.send`（行 253–254）。購読者がいなければ捨てる。

### 4.3 `run_stream_server`（行 264–301）— GUI 向け TCP 配信（一方向）
- `TcpListener::bind(stream_addr)`（行 268、既定 `127.0.0.1:5141`）。
- 接続ごとに `stream_tx.subscribe()` し（行 274）、broadcast の JSON 行を `socket.write_all(line)+"\n"` で送るだけ（行 283–284）。
- Lagged はスキップ継続（行 290–292）、Closed で終了（行 295）。
- コメント（行 263）: 「外部公開しない前提なので認証は持たない」。

### 4.4 `Server/src/config.rs`（76行）— 設定スキーマ
- `struct ServerConfig { bind_addr, stream_addr }`（行 13–20、stream_addr 既定 `127.0.0.1:5141`）。
- `struct LoggingConfig { level, max_size_mb, keep_files }`（行 27–32）。
- `Default`（行 34–48）: `bind_addr="0.0.0.0:514"`, logging=info/10MB/7。
- `load_config()`（行 50–66）: 無ければ既定を書き出して返す。**実行時に外部から設定を書き換える経路は無い**。

### 4.5 制御ポートの有無（重要）
- Server の関数は上記7個のみで、**`get_config`/`set_config` 等「Console から設定を変更する要求を受け付ける TCP/制御エンドポイントは存在しない」**（`grep -n "get_config|set_config"` → 0件）。
- すなわち Server→Console は **JSON Lines の一方向配信のみ**。Console→Server の制御（設定変更）チャネルは未実装。

---

## 5. アーキテクチャの実態（確定）

```
[ syslog 送信機器 ] --UDP:514--> [ Server: run_syslog_server ]
                                        | parse → JSON
                                        v  broadcast
                                 [ Server: run_stream_server  TCP:5141 ]
                                        | JSON Lines（一方向）
                                        v
                                 [ Console: net.rs run_client ] → GUI 表示（未実装）
```

- **一貫した単一設計**である（前回私が主張した「3分裂」は誤り）。Console は Server の 5141 ストリームに接続する TCP クライアント。
- 受信したログを画面に出す GUI 本体（eframe/egui の App、`main.rs`）が**未実装**。
- 設定変更（コンソールから syslog 設定を変える）は、**Console 側にも Server 側にも経路が無い**＝この機能は現状ゼロ。

参考: `Portable` クレートは eframe/egui の GUI ビューア（`Portable/Cargo.toml` に eframe="0.27"/egui="0.27"/arboard/rfd 等、`Portable/src/main.rs` 約11.9KB 存在）。Console の GUI はこれを土台に流用できる位置づけ。

---

## 6. 完成度評価

| 層 | 状態 | 根拠 |
|---|---|---|
| 受信メッセージ型 `parser.rs` | ✅ 実装済 | SyslogMessage/Severity（Server と一致前提） |
| ローカル設定 `settings.rs` | ✅ 実装済 | server_addr の load/save |
| OS データ保存先 `platform.rs` | ✅ 実装済 | data_dir/config_path/file manager |
| TCP クライアント `net.rs` | ✅ 実装済（テスト付） | run_client + 単体テスト2本 |
| サービス監視 `service.rs` | ✅ 実装済 | 3 OS の status/start/stop/restart |
| macOS メニュー `macos_menu.rs` | ✅ 実装済 | MenuRequest/drain_requests/install |
| **GUI 本体 `main.rs`** | ❌ 未実装 | ファイル自体が無い |
| **`Console/Cargo.toml`** | ❌ 未実装 | ファイル自体が無い |
| **workspace 登録** | ❌ 未実装 | members に Console 無し |
| Server→Console ログ配信 | ✅ 実装済 | run_stream_server（5141, JSON Lines） |
| **Console→Server 設定変更** | ❌ 未実装 | 両側に制御経路が無い |

---

## 7. 完成までに足りないもの

1. **`Console/Cargo.toml` 作成**（`Portable/Cargo.toml` がほぼテンプレートになる: eframe/egui/tokio/serde/serde_json/toml/chrono/arboard/rfd、macOS は cocoa/objc、build に winres）。
2. **`Console/src/main.rs`（GUI 本体）実装** — ここが最大の欠落。`Portable/src/main.rs` の egui ビューア（ログテーブル/フィルタ/自動スクロール/設定画面）を土台に、`net.rs::run_client` を spawn して `msg_tx`→GUI へ流し、`ConnState` をバナー表示、`service.rs::status()` をポーリング表示する形にまとめる。
3. **ルート `Cargo.toml` の members に `"Console"` を追加**。
4. （任意・別フェーズ）**syslog 設定変更機能** — 現状ゼロ。Server に制御用 TCP エンドポイント（例 set_config）を新設し、Console 側に送信処理を追加して対にする必要がある。設計判断（ポート/プロトコル/認証/再起動方式）が要る。

**最短で「動く」状態**: 1→2→3 で、Server の 5141 に接続してログを画面表示する最小版が完成する（設定変更=4 は後回し可）。

---

## 8. 第三者レビュア向け・再検証コマンド

```bash
cd /Volumes/2TB_USB/dev/vlt-syslogd-private

# workspace メンバー
sed -n '9,11p' Cargo.toml

# Console に Cargo.toml / main.rs が無いことの確認
test -f Console/Cargo.toml && echo HAS_CARGO || echo NO_CARGO
test -f Console/src/main.rs && echo HAS_MAIN || echo NO_MAIN
wc -l Console/src/*.rs Console/build.rs

# net.rs が run_client クライアントであること / 旧主張の識別子が無いこと
grep -nE "run_client|ConnState|SyslogMessage" Console/src/net.rs
grep -nE "ConsoleClient|get_config_raw|set_config_raw|run_tcp|run_udp|round_trip" Console/src/net.rs   # 0件のはず

# service.rs が OS サービス管理であること
grep -nE "ServiceStatus|fn status|fn start|fn stop|fn restart" Console/src/service.rs

# Server の関数一覧と制御ポートの不在
grep -nE "^(async )?fn |get_config|set_config" Server/src/main.rs   # get_config/set_config は 0件のはず
sed -n '264,301p' Server/src/main.rs   # run_stream_server（5141, 一方向 JSON Lines）
sed -n '13,25p' Server/src/config.rs   # bind_addr / stream_addr
```
