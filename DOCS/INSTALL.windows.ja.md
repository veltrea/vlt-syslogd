# vlt-syslogd — インストールガイド（Windows）

English: [INSTALL.windows.md](INSTALL.windows.md) ／ 他の OS: [macOS](INSTALL.macos.ja.md)・[Linux](INSTALL.linux.ja.md) ／ [入口に戻る](INSTALL.ja.md)

このページは **Windows 専用**です。macOS / Linux の方は上のリンクから。

vlt-syslogd は syslog を受信して表示するソフトです。使い方が3通りあるので、**まず自分がどれをやりたいかを選んでください**。

---

## まず — あなたはどれを入れる？

| やりたいこと | 入れるもの | 進む順番 |
|---|---|---|
| **とにかく試したい / 1台で完結させたい** | Portable だけ（インストール不要） | §1 → §2 |
| **常時 syslog を受け続けたい（サーバ用途）** | Server（常駐）＋ Console（見る画面） | §1 → §3 → §4 → §5 → §6 |
| **すでに動いている Server を、別の画面から見たい** | Console だけ | §1 → §5 |

迷ったら、いちばん上の **Portable** から始めてください。サービス登録も管理者権限も要らず、起動すればすぐ動きます。

> 3つの違い（Server / Console / Portable とは何か）は[構成リファレンス](REFERENCE.ja.md)に。インストールするだけなら読まなくて構いません。

---

## 1. 実行ファイルを用意する

GitHub Releases から **Windows 用**のファイル（`.exe`）をダウンロードします。自分のコースで使うものだけで構いません。

- `vlt-syslogd-srv.exe`（Server）
- `vlt-syslogd-console.exe`（Console）
- `vlt-syslogd-portable.exe`（Portable）

SmartScreen の警告が出たら「詳細情報」→「実行」を選んでください。

> **ソースから自分でビルドしたい場合**は [BUILD.ja.md](BUILD.ja.md) を参照。ビルド後の手順は配布バイナリと同じです。

---

## 2. Portable を起動して試す（インストール不要）

Portable は単体で UDP 514 を待ち受ける GUI です。サービス登録は不要。**ダウンロードした `vlt-syslogd-portable.exe` をダブルクリックで開く**だけです。

ウィンドウが開いたら、PowerShell からテスト送信して、表に行が増えれば成功です。

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::ASCII.GetBytes('<34>Oct 11 22:14:15 myhost myapp: hello')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

これで「とにかく試したい」コースは完了です。常駐させたくなったら §3 へ。

---

## 3. Server をインストール（Windows サービス）

Server は画面を持たない常駐プログラムで、Windows サービスとして登録します。インストールスクリプトは `Server/` フォルダにあり、「実行ファイル配置 → データフォルダ作成 → サービス登録 → 起動」までを自動でやります。管理者権限が要るのは**インストール時の1回だけ**で、以後は自動で動き、OS 起動時にも立ち上がります。

**管理者として実行**した PowerShell から:

```powershell
cd Server
powershell -ExecutionPolicy Bypass -File .\install-windows.ps1
# または: ... -File .\install-windows.ps1 -BinPath C:\path\to\vlt-syslogd-srv.exe
```

配置先:

- 実行ファイル: `C:\Program Files\vlt-syslogd\vlt-syslogd-srv.exe`
- データ/ログ: `C:\ProgramData\vlt-syslogd\`
- サービス: `vlt-syslogd-srv`(スタートアップ種別: 自動)
- 状態確認: `sc.exe query vlt-syslogd-srv`

> PowerShell スクリプトは CP932 文字化け回避のため**英語（ASCII のみ）**で書いてあります。日本語の説明はこのマニュアルにあります。

---

## 4. 設定ファイル（必要に応じて）

Server は**初回起動時**にデータフォルダ `C:\ProgramData\vlt-syslogd\` へ `config.toml` を自動生成します。既定のままで動くので、**ポートやネットワークを変えたいときだけ**読んでください。

```toml
[server]
bind_addr    = "0.0.0.0:514"        # syslog 受信(UDP)
stream_addr  = "127.0.0.1:5141"     # Console への配信(TCP)
control_addr = "127.0.0.1:5142"     # 制御チャネル(TCP)

[logging]
level        = "info"
max_size_mb  = 10
keep_files   = 7
```

変更したら、サービスを再起動するか、Console の「**サーバへ適用(再起動)**」を使ってください。テスト用に `VLT_SYSLOGD_DATA_DIR` 環境変数でデータフォルダを上書きできます。

### ネットワーク構成 — どこから届くか

3 つのポートは**意図的に**公開範囲を変えてあります。

| ポート | 既定 bind | 到達できる範囲 | 変えるには |
|---|---|---|---|
| 514/udp(受信) | `0.0.0.0` | 任意のホスト(リモート機器が syslog を送れる) | `0.0.0.0` のまま、または LAN IP に固定 |
| 5141/tcp(配信) | `127.0.0.1` | **同一ホストのみ** | リモート Console 用に到達可能アドレスへ(下の注意) |
| 5142/tcp(制御) | `127.0.0.1` | **同一ホストのみ** | リモート Console 用に到達可能アドレスへ(下の注意) |

- **リモート機器から受信**できるのは、`bind_addr` が `0.0.0.0`(または LAN IP)**かつ**ファイアウォールが UDP 514 を許可している場合のみ。`bind_addr = "127.0.0.1:514"` にするとローカルホストからしか送れません。
- **Console は既定で Server と同一マシンで動かす必要があります**。`stream_addr` / `control_addr` がループバック限定だからです。別マシンの Console を使うには、これらを到達可能アドレスに変更する必要がありますが、その場合**制御チャネルが露出**します(`set_config` で Server 設定を書き換え可能)。SSH トンネル経由を推奨します。

### ファイアウォール — syslog ポートを開ける

サービスが起動していることと、ポートが到達可能であることは別問題です。サービスが「稼働中」なのにリモート機器のログが出ない場合、たいていは Windows ファイアウォールが原因です。**管理者 PowerShell** で UDP 514 を開けてください(リモート Console 用に意図的に公開する場合のみ 5141/5142 も)。

```powershell
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

---

## 5. Console を起動

Console は Server に接続して表示・操作する GUI です。サービスにする必要はありません。**ダウンロードした `vlt-syslogd-console.exe` をダブルクリックで開きます。**

初回起動後、**⚙ 設定**を開いて、接続先が Server と一致しているか確認します。

- 配信アドレス → Server の `stream_addr`(既定 `127.0.0.1:5141`)
- 制御アドレス → Server の `control_addr`(既定 `127.0.0.1:5142`)

Console には Server を**開始 / 停止 / 適用-再起動**するボタンと、**サービス状態**の表示があります。これらは Server が同梱のインストーラでサービス登録されているときに動作します。想定挙動:

- 開始/停止/再起動は **UAC（ユーザーアカウント制御）**で昇格します。承認すると続行します。
- **サービス未インストール時**: 操作は即座に分かりやすいメッセージで失敗します。**設定の保存自体は成功**し、再起動だけがスキップされます。

---

## 6. 動作確認

1. サービスが**稼働している**か確認:
   `sc.exe query vlt-syslogd-srv`（`RUNNING` を確認）
2. Server が**待ち受けている**か確認:
   `Get-NetUDPEndpoint -LocalPort 514` と `Get-NetTCPConnection -State Listen -LocalPort 5141,5142`
3. **ローカル**でテスト送信し、Console に出るか確認:
   ```powershell
   $u = New-Object System.Net.Sockets.UdpClient
   $b = [Text.Encoding]::ASCII.GetBytes('<34>Oct 11 22:14:15 myhost myapp: hello')
   $u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
   ```
   Console の**状態インジケータが緑(🟢 /「● 受信中」)**になり、表に行が追加されれば OK。
4. **リモート到達性**(リモート機器から送る場合のみ): ネットワーク上の*別ホスト*から、UDP 514 が開いていてメッセージが届くか確認。届かない場合は `bind_addr`(`127.0.0.1` ではなく `0.0.0.0`/LAN IP か)とファイアウォール(§4)を再確認。「サービス稼働中」は「ポート到達可能」を**保証しません**。

---

## 7. アンインストール

**管理者 PowerShell** から:

```powershell
cd Server; powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1
```

サービスを停止・解除し、配置した実行ファイルを削除します。**設定とログはデータフォルダに残します**。完全に消したい場合は `C:\ProgramData\vlt-syslogd\` を手動で削除してください。

---

## 8. トラブルシュート

| 症状 | 原因・対処 |
|---|---|
| 日本語コンソール出力が文字化け | コンソールが CP932 のため。インストーラとサービスは ASCII のみで影響なし。 |
| Console が常に「○ 切断」 | Server 未起動、または配信アドレス不一致。Server の稼働と、Console の配信アドレス = Server の `stream_addr` を確認。 |
| 設定の「現在値を取得」が失敗 | 制御アドレス不一致、または制御ポート未対応の古い Server。`control_addr` を確認し Server を再インストール。 |
| インストール後も Console のサービス状態が「未インストール」 | サービス名を独自に変更したときに起きる。Console とインストーラが同じサービス名を指す必要がある(既定では一致済み)。識別子は[構成リファレンス](REFERENCE.ja.md)参照。 |
| サービスは稼働中なのにリモート機器のログが来ない | 「稼働中」≠「到達可能」。(1) `bind_addr` が `127.0.0.1` でなく `0.0.0.0`/LAN IP か、(2) Windows ファイアウォールが UDP 514 を許可しているか(§4)、(3) 送信側が Server の実 IP を向いているか、を確認。 |
| リモート Console が接続できない | `stream_addr`/`control_addr` は既定でループバック限定。到達可能アドレスへ変更しファイアウォールを開ける。制御ポートは Server 設定を書き換え可能なので信頼ホストに限定するか SSH トンネル経由を推奨。 |
| インストールスクリプトが実行できない | 管理者として PowerShell を開き、`-ExecutionPolicy Bypass` を付けて実行しているか確認。 |

---

部品の構成・ポート・サービス識別子の一覧は[構成リファレンス（REFERENCE.ja.md）](REFERENCE.ja.md)を参照してください。
