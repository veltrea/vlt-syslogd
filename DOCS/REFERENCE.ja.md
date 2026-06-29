# vlt-syslogd — 構成リファレンス

English version: [REFERENCE.md](REFERENCE.md)

インストールには不要です。仕組みを知りたいとき・設定を自分で変えるときの参照用です。インストール手順は OS 別ページへ: [macOS](INSTALL.macos.ja.md)・[Linux](INSTALL.linux.ja.md)・[Windows](INSTALL.windows.ja.md)。

---

## 3つの部品

| 呼び名 | 実行ファイル名 | 役割 | 動作形態 |
|---|---|---|---|
| **Server** | `vlt-syslogd-srv`（Windows は `.exe`） | 画面を持たない syslog エンジン。UDP で syslog を受信し Console へ配信する。 | 常駐サービス(launchd / systemd / Windows サービス) |
| **Console** | `vlt-syslogd-console`（Windows は `.exe`） | Server に TCP 接続して表示・操作する GUI ビューア。 | デスクトップアプリ |
| **Portable** | `vlt-syslogd-portable`（Windows は `.exe`） | 自分で UDP syslog を待ち受ける単体 GUI(サービス不要)。 | デスクトップアプリ |

---

## Server が使うポート

- **514/udp** — syslog 受信(標準ポート。macOS / Linux では bind に管理者/root 権限が要る。下記参照)
- **5141/tcp** — Console への配信(JSON Lines、一方向)
- **5142/tcp** — 制御チャネル(`get_config` / `set_config`)

公開範囲の既定値と変更時の注意は、各 OS ページの「ネットワーク構成」節を参照してください。

### なぜ 514 番の待ち受けに管理者権限が要るのか（macOS / Linux）

0〜1023 番のポートは **特権ポート（well-known ports）** と呼ばれ、Unix 系 OS では root（管理者）権限を持つプロセスでないと bind（待ち受け開始）できません。一般ユーザーのプログラムが、syslog・SSH・HTTP といった標準サービスのポートを勝手に乗っ取って成りすますのを防ぐための、古くからある仕組みです。syslog の標準ポートである 514 もこの範囲に入ります。

**macOS も BSD 由来のため、この制限がそのまま当てはまります**（「Mac だから不要」ということはありません）。各 OS での扱いは次のとおり:

- **macOS** — Server は launchd の **LaunchDaemon（root 実行）**として登録されるので 514 を bind できます。インストール時に管理者認証を求められるのはこのためです。Portable を一般ユーザーで起動した場合は、既定の 514 を bind できず失敗することがあります。その場合は 1024 番以上のポートに変える（例: `bind_addr = "0.0.0.0:5514"`、送信側もそのポートに合わせる）か、管理者権限で起動してください。
- **Linux** — systemd ユニットは既定で root 実行のため bind できます。root を避けたい場合は `CAP_NET_BIND_SERVICE` を付与する方法があります（[INSTALL.linux.ja.md](INSTALL.linux.ja.md) §8）。
- **Windows** — この「特権ポート」の概念がなく、**514 の待ち受け自体に管理者権限は不要**です（インストーラがサービス登録時に管理者権限を使うのは、これとは別の理由です）。

---

## サービス識別子

同梱のインストーラと Console は最初からこの名前で一致しているので、**通常は意識する必要はありません**。インストーラを自分で改造してサービス名を変えた場合だけ、Console 側も同じ名前に揃えてください。

| OS | 識別子 |
|---|---|
| macOS（launchd ラベル） | `com.veltrea.vlt-syslogd-srv` |
| Linux（systemd ユニット） | `vlt-syslogd-srv.service` |
| Windows（サービス名） | `vlt-syslogd-srv` |
