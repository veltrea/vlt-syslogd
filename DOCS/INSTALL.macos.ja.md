# vlt-syslogd — インストールガイド（macOS）

English: [INSTALL.macos.md](INSTALL.macos.md) ／ 他の OS: [Linux](INSTALL.linux.ja.md)・[Windows](INSTALL.windows.ja.md) ／ [入口に戻る](INSTALL.ja.md)

このページは **macOS 専用**です。Linux / Windows の方は上のリンクから自分の OS のページへ。

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

GitHub Releases から **macOS 用**のファイルをダウンロードします。自分のコースで使うものだけで構いません。

- `vlt-syslogd-srv`（Server）
- `vlt-syslogd-console`（Console）
- `vlt-syslogd-portable`（Portable）

macOS の配布物は ad-hoc 署名のため、初回起動で「壊れている」等と出ることがあります。その場合は §8 のトラブル項目（quarantine）を参照してください。

> **ソースから自分でビルドしたい場合**は [BUILD.ja.md](BUILD.ja.md) を参照。ビルド後の手順は配布バイナリと同じです。

---

## 2. Portable を起動して試す（インストール不要）

Portable は単体で UDP 514 を待ち受ける GUI です。サービス登録は不要。**ダウンロードした `vlt-syslogd-portable` をそのまま開く**だけです。

ウィンドウが開いたら、ターミナルからテスト送信して、表に行が増えれば成功です。

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

これで「とにかく試したい」コースは完了です。常駐させたくなったら §3 へ。

---

## 3. Server をインストール（launchd 常駐サービス）

Server は画面を持たない常駐プログラムで、launchd のサービスとして登録します。インストールスクリプトは `Server/` フォルダにあり、「実行ファイル配置 → データフォルダ作成 → サービス登録 → 起動」までを自動でやります。管理者を聞かれるのは**インストール時の1回だけ**で、以後は自動で動き、Mac の起動時にも立ち上がります。

```bash
cd Server
sudo ./install-macos.sh            # または: sudo ./install-macos.sh /path/to/vlt-syslogd-srv
```

スクリプトは実行ファイルを次の順で自動検出します: 引数で渡したパス → スクリプトと同じフォルダのコピー → `../target/release/` → `./target/release/`。

配置先:

- 実行ファイル: `/usr/local/bin/vlt-syslogd-srv`
- データ/ログ: `/usr/local/var/vlt-syslogd/`
- LaunchDaemon: `/Library/LaunchDaemons/com.veltrea.vlt-syslogd-srv.plist`
- 状態確認: `sudo launchctl print system/com.veltrea.vlt-syslogd-srv`

---

## 4. 設定ファイル（必要に応じて）

Server は**初回起動時**にデータフォルダ `/usr/local/var/vlt-syslogd/` へ `config.toml` を自動生成します。既定のままで動くので、**ポートやネットワークを変えたいときだけ**読んでください。

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

### ファイアウォール（macOS）

macOS のアプリケーションファイアウォールはポートではなく**アプリ単位**でフィルタします。有効な場合は、プロンプトで `vlt-syslogd-srv` の着信接続を許可するか、システム設定 → ネットワーク → ファイアウォール → オプションで追加してください。「サービス稼働中」でもファイアウォールで弾かれていればリモートのログは届きません。

---

## 5. Console を起動

Console は Server に接続して表示・操作する GUI です。サービスにする必要はありません。**ダウンロードした `vlt-syslogd-console` をそのまま開きます。**

初回起動後、**⚙ 設定**を開いて、接続先が Server と一致しているか確認します。

- 配信アドレス → Server の `stream_addr`(既定 `127.0.0.1:5141`)
- 制御アドレス → Server の `control_addr`(既定 `127.0.0.1:5142`)

Console には Server を**開始 / 停止 / 適用-再起動**するボタンと、**サービス状態**の表示があります。これらは Server が同梱のインストーラでサービス登録されているときに動作します。想定挙動:

- これらの操作は `launchctl` を管理者として実行するため、システムの**パスワード / Touch ID ダイアログ**が出ます。認証すると続行します。
- **サービス未インストール時**: 操作は即座に分かりやすいメッセージで失敗し、**認証は出ません**。**設定の保存自体は成功**し、再起動だけがスキップされます。

配布する場合は `.app` にまとめて ad-hoc 署名してください（プロジェクトの署名手順参照）。ローカル利用なら実行ファイルを直接開くだけで十分です。

---

## 6. 動作確認

1. サービスが**登録・ロードされている**か確認(プロセスの有無ではなく):
   `sudo launchctl print system/com.veltrea.vlt-syslogd-srv`（`state = running` が出る）
2. Server が**待ち受けている**か確認:
   `lsof -nP -iUDP:514` と `lsof -nP -iTCP -sTCP:LISTEN | grep -E "5141|5142"`
3. **ローカル**でテスト送信し、Console に出るか確認:
   ```bash
   printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
   ```
   Console の**状態インジケータが緑(🟢 /「● 受信中」)**になり、表に行が追加されれば OK。
4. **リモート到達性**(リモート機器から送る場合のみ): ネットワーク上の*別ホスト*から、UDP 514 が開いていてメッセージが届くか確認:
   ```bash
   # 別マシンから — SERVER_IP は置き換える
   printf '<34>Oct 11 22:14:15 dev1 app: remote-test' | nc -u -w1 SERVER_IP 514
   ```
   届かない場合は `bind_addr`(`127.0.0.1` ではなく `0.0.0.0`/LAN IP か)とファイアウォール(§4)を再確認。「サービス稼働中」は「ポート到達可能」を**保証しません**。

---

## 7. アンインストール

```bash
cd Server && sudo ./uninstall-macos.sh
```

サービスを停止・解除し、配置した実行ファイルを削除します。**設定とログはデータフォルダに残します**。完全に消したい場合は `/usr/local/var/vlt-syslogd/` を手動で削除してください。

---

## 8. トラブルシュート

| 症状 | 原因・対処 |
|---|---|
| アプリが開けない(「壊れている」/ quarantine) | ad-hoc 署名のダウンロードには quarantine 属性が付く。右クリック→開く、または `xattr -dr com.apple.quarantine <app>`。 |
| Console が常に「○ 切断」 | Server 未起動、または配信アドレス不一致。Server の稼働と、Console の配信アドレス = Server の `stream_addr` を確認。 |
| 設定の「現在値を取得」が失敗 | 制御アドレス不一致、または制御ポート未対応の古い Server。`control_addr` を確認し Server を再インストール。 |
| インストール後も Console のサービス状態が「未インストール」 | launchd ラベルを独自に変更したときに起きる。Console とインストーラが同じラベルを指す必要がある(既定では一致済み)。識別子は[構成リファレンス](REFERENCE.ja.md)参照。 |
| サービスは稼働中なのにリモート機器のログが来ない | 「稼働中」≠「到達可能」。(1) `bind_addr` が `127.0.0.1` でなく `0.0.0.0`/LAN IP か、(2) ファイアウォールが `vlt-syslogd-srv` の着信を許可しているか(§4)、(3) 送信側が Server の実 IP を向いているか、を確認。別ホストから `nc -u SERVER_IP 514` で検証。 |
| リモート Console が接続できない | `stream_addr`/`control_addr` は既定でループバック限定。到達可能アドレスへ変更する。制御ポートは Server 設定を書き換え可能なので SSH トンネル経由を推奨。 |

---

部品の構成・ポート・サービス識別子の一覧は[構成リファレンス（REFERENCE.ja.md）](REFERENCE.ja.md)を参照してください。
