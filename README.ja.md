# Veltrea Syslog Server

[English](README.md) · 日本語 · [한국어](README.ko.md) · [简体中文](README.zh.md) · [繁體中文](README.tw.md) · [Tiếng Việt](README.vi.md)

**受信した 1 通ごとに Shift_JIS と UTF-8 を判定して復号します。同じポートに符号化が混ざっていても、文字化けしません。**

**[解説サイト](https://veltrea.github.io/vlt-syslogd/)** · [English page](https://veltrea.github.io/vlt-syslogd/index.en.html) · **[リリース v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)**

![Enc 列に UTF-8 と Shift_JIS と Shift_JIS (MSG-SD/BOM-Missing) が並んだ Portable のログ画面](DOCS/assets/portable-main.png)

*サイトと同じ画面です。v0.4.0 の Windows 版。Enc 列がその行の判定で、`UTF-8`、`Shift_JIS`、RFC 5424 の構造化データが文字集合を宣言したときの `Shift_JIS (MSG-SD/BOM-Missing)` が出ます。行を右クリックして **Copy as Hex** を選ぶと、元のバイト列を16進でコピーできます。*

最新は **v0.4.0**。Windows、macOS、Linux (x86_64) 向けです。どれにするか迷ったら **Portable** から始めてください。

画面の文言は日本語です。Windows と Linux のメニューは「ファイル(File)」「編集(Edit)」「ヘルプ(Help)」と併記しています。

## 三つの形

syslog を読む考え方は同じで、動かし方だけが違います。

| | Portable | Server | Console |
|---|---|---|---|
| バイナリ | `vlt-syslogd-portable` | `vlt-syslogd-srv` | `vlt-syslogd-console` |
| 役割 | 自分で UDP を開く GUI。インストーラもサービス登録も要りません。 | 画面のないエンジン。Windows サービス、launchd の LaunchDaemon、systemd のいずれか。起動と同時に受けてファイルへ書きます。 | 動いている Server に TCP で繋がる GUI。閉じても受信は止まりません。サービスの開始・停止・再起動もこの画面から。 |
| ポート | UDP。既定は `0.0.0.0:514` | UDP `514`（syslog）。TCP `5141`（Console への配信）と TCP `5142`（制御）は既定で `127.0.0.1` | Server の配信先と制御先へ接続 |
| v0.4.0 の配布物 | Windows、macOS、Linux x86_64 | Windows と macOS。Linux はソースからビルド（`Server/install-linux.sh` はリポジトリにある。このリリースに Linux 用 Server バイナリはない） | Windows と macOS。Linux はソースからビルド |

**App** と **Portable** は同じ GUI です。データの置き場所だけが違います。

| 形態 | データディレクトリ |
|---|---|
| **App** | Windows は `%APPDATA%\vlt-syslogd`、macOS は `~/Library/Application Support/vlt-syslogd`、Linux は `$XDG_DATA_HOME/vlt-syslogd` または `~/.local/share/vlt-syslogd` |
| **Portable** | 実行ファイルの隣。macOS では `.app` の隣（USB に入れて持ち歩く。システムの中には書かない） |

どちらの形態でも `VLT_SYSLOGD_DATA_DIR` があればそちらを使います。Server のデータの既定はシステム領域です。`C:\ProgramData\vlt-syslogd`、`/usr/local/var/vlt-syslogd`、`/var/lib/vlt-syslogd`。

環境設定（**ファイル → 環境設定…**）にあるのは二つだけです。待ち受けポートとログ保存先。**適用** で保存します。

## v0.4.0 を入手して、1 通送る

次のファイルは [v0.4.0 のリリース資産](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0) です。試しの送信先は、ログの 1 行目に出たポートにしてください。`Listening on 0.0.0.0:514 (UDP)` と出ていれば、そのポートは開いています。

常駐させる、別の機器から受ける、という先は [インストールガイド](DOCS/INSTALL.ja.md) にあります。

### Windows

| 形 | ファイル |
|---|---|
| Portable | [vlt-syslogd-portable-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-portable-windows-v0.4.0.zip) → `vlt-syslogd-portable.exe` |
| App | [vlt-syslogd-app-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-app-windows-v0.4.0.zip) → `vlt-syslogd-windows-x86_64-app-v0.4.0.exe` |
| Server | [vlt-syslogd-srv-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-windows-v0.4.0.zip) → `vlt-syslogd-srv.exe` と `install-windows.ps1` |
| Console | [vlt-syslogd-console-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-windows-v0.4.0.zip) → `vlt-syslogd-console.exe` |

展開して Portable の exe をダブルクリックします。SmartScreen が出たら **詳細情報** → **実行**。UDP 514 の bind に管理者権限は要りません。

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::UTF8.GetBytes('<132>auth: 認証に失敗しました')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

行が増えれば届いています。この本文は BOM のない UTF-8 で、このプログラムが想定している普通の形です。

### macOS

| 形 | ファイル |
|---|---|
| Portable | [vlt-syslogd-macos-portable-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.dmg) または [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.zip) → `vlt-syslogd-portable.app` |
| App | [vlt-syslogd-macos-app-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.dmg) または [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.zip) → `vlt-syslogd.app` |
| Server | [vlt-syslogd-srv-macos-v0.4.0.tar.gz](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-macos-v0.4.0.tar.gz) → `vlt-syslogd-srv` と `install-macos.sh` |
| Console | [vlt-syslogd-console-macos-app-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-macos-app-v0.4.0.zip) → `vlt-syslogd-console.app` |

ビルドは ad-hoc 署名で、公証はしていません。ダウンロードしたアプリには Gatekeeper の隔離属性が付きます。一度外すか、右クリックして **開く** を選んでください。

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

Portable はデータを `.app` の隣に置きます。隔離属性が残っていると App Translocation でその場所が見えなくなります。日本語フォントは macOS に入っているヒラギノを使います。

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

macOS の `logger` はローカルのシステムログへ書きます。ホストと UDP ポートは指定できません。こちらでは `nc` を使ってください。514 が開いたと思い込む前に [ポート 514](#ポート-514) を見てください。

### Linux (x86_64)

このリリースに入っているのは GUI だけです。Server と Console のビルド済みバイナリはありません。必要ならソースからビルドします（[BUILD.ja.md](DOCS/BUILD.ja.md)、[INSTALL.linux.ja.md](DOCS/INSTALL.linux.ja.md)）。

| 形 | ファイル |
|---|---|
| Portable | [vlt-syslogd-linux-x86_64-portable-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-portable-v0.4.0) |
| App | [vlt-syslogd-linux-x86_64-app-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-app-v0.4.0) |

動的リンクの x86_64 ELF です。ウィンドウを開くにはディスプレイ（X11 または Wayland）が要ります。v0.4.0 のリリースノートは OpenGL も要件に挙げています。

```bash
chmod +x vlt-syslogd-linux-x86_64-portable-v0.4.0
./vlt-syslogd-linux-x86_64-portable-v0.4.0
```

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

util-linux の `logger` は UDP で送れます。macOS の `logger` にはこの指定はありません。Linux での形です。

```bash
logger -n 127.0.0.1 -P 514 -d 'hello from logger'
```

日本語を出すには、GUI が探す CJK フォントが要ります。Noto Sans CJK（`/usr/share/fonts` 以下の `NotoSansCJK-Regular.ttc`）か、IPA ゴシック（`/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf`）です。無いと □ になります。Debian / Ubuntu の `fonts-noto-cjk` は、そのうち先頭のパスへ Noto を置きます。

## 先に知っておくこと

### ポート 514

ログの 1 行目が bind の結果です。失敗の行が出ているあいだ、514 へ送っても画面には出ません。

- **Windows。** UDP 514 の bind に管理者は要りません。インストール時に管理者を求めるのは、サービスを登録するときだけです。
- **Linux。** 1024 未満は root か `CAP_NET_BIND_SERVICE` が要ります。Portable のバイナリなら次です。

  ```bash
  sudo setcap 'cap_net_bind_service=+ep' ./vlt-syslogd-linux-x86_64-portable-v0.4.0
  ```

  `Server/install-linux.sh` は root で動く systemd ユニットを入れます。ユニットファイルには `AmbientCapabilities=CAP_NET_BIND_SERVICE` を使うコメントアウトされた例もあります。
- **macOS。** root でない状態で `0.0.0.0:514` を bind すると `EACCES` になった、という記録が [DOCS/portable_macos_port_behavior.md](DOCS/portable_macos_port_behavior.md) にあります。アプリ内のヒントは、macOS のワイルドカード bind には root が要らない、とまだ表示します。信じるのはログの 1 行目です。

bind に失敗すると、バナーには **5514** が入ります。**待ち受け開始** はそのセッションだけそのポートで受け、保存はしません。残すには **環境設定** で **適用** するか、`VLT_SYSLOGD_BIND=0.0.0.0:5514` を付けて起動します。試しの送信は、ログの行が示しているポートへ向けてください。macOS で 514 そのものを掴むには、管理者権限で Portable を起動するか、root の LaunchDaemon として入る Server（`com.veltrea.vlt-syslogd-srv`）を使います。

### macOS の Gatekeeper

これらのビルドの後ろに Apple Developer アカウントはありません。初回は、右クリック → **開く**、または上の `com.apple.quarantine` の除去が済むまで止まります。

### サービスが動いていても、届かないことがある

別の機器から届くのは、`bind_addr` が `0.0.0.0` または LAN のアドレスで、かつファイアウォールが UDP 514 を通しているときです。配信（`5141`）と制御（`5142`）は、変えない限りループバックのままです。この二つをマシンの外へ出すと、Server の設定を書き換えられる `set_config` も出ます。別のマシンの Console を使うときは、インストールガイドが SSH トンネルを勧めています。

```powershell
# Windows。管理者の PowerShell
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

```bash
# Linux
sudo ufw allow 514/udp
# firewalld:
sudo firewall-cmd --permanent --add-port=514/udp && sudo firewall-cmd --reload
```

macOS のアプリケーションファイアウォールはアプリ単位です。尋ねられたら `vlt-syslogd-srv` の着信を許可します。

## 背景

日本語がきちんと出ない、軽い Windows 用 syslog サーバーを長く使っていたので、Rust で一から書きました。1 通ごとに復号します。BOM、RFC 5424 の構造化データにある `charset`、あるいはバイト列そのもの（`chardetng` による Shift_JIS の推定を含む）です。Enc 列はその判定です。保守は、健康と気力が続くあいだ続けます。作者は veltrea です。

USB メモリに入れておく単一ファイル、という Portable の使い方は今もそのままです。

## ドキュメント

| | |
|---|---|
| インストール | [INSTALL.ja.md](DOCS/INSTALL.ja.md) — [Windows](DOCS/INSTALL.windows.ja.md) · [macOS](DOCS/INSTALL.macos.ja.md) · [Linux](DOCS/INSTALL.linux.ja.md) |
| ポートとサービス名 | [REFERENCE.ja.md](DOCS/REFERENCE.ja.md) |
| ビルド | [BUILD.ja.md](DOCS/BUILD.ja.md) — `cargo build --release --workspace`（edition 2024） |
| 符号化のメモ | [syslog_verification_report.ja.md](DOCS/syslog_verification_report.ja.md) |

## ライセンス

[MIT](LICENSE)。Copyright (c) 2026 veltrea。
