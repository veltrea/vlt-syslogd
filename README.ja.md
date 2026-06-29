# vlt-syslogd-server (Workspace)

2バイト文字(CJK)環境での高い視認性と実用性を追求した、Windows / macOS 対応 Syslog ソリューション。

## プロジェクトの構成と目的

本プロジェクトでは、用途に合わせて 2 つの形態を提供するためにディレクトリを分離・整理しました。

### 1. [Portable](./Portable) (Released v0.3.0)
- **ステータス**: リリース済み。健康と気力が続く限り、保守を継続。
- **コンセプト**: 「USBメモリに忍ばせておく、エンジニアの万能ナイフ」。
- **特徴**: GUI とサーバーエンジンを一体化した単一実行ファイル。インストール不要、インターネット不要で、現場のデバッグや一時的なログ収集に最高の機動力を提供します。
- **想い**: 日々忙しく働くエンジニアとして、「こういうのがあれば便利だな」と思って自作しました。同じような仕事をしている皆さんにも、ぜひ気軽に使ってもらいたいと考えています。

### 2. [Server](./Server) (Under Development)
- **ステータス**: 次期目標として開発中。
- **コンセプト**: 「Windows サーバーでの本格安定運用」。
- **特徴**: Windows サービスとしてバックグラウンドで動作する「エンジン」と、必要時のみ接続する「監視用フロントエンド」を分離。
- **目的**: 常時稼働が求められる本番運用環境において、堅牢なログ収集基盤を提供します。

---

## 開発の背景

長年愛用してきた既存の Windows 用軽量 Syslog サーバーが抱えていた「日本語表示の限界」を克服するため、Rust 言語でゼロから構築しました。

詳細は各ディレクトリの README を参照してください。

---

## ダウンロード（macOS）

`./Portable/build-macos.sh` を実行すると、ad-hoc 署名済みの macOS バンドルが `dist/` に作られます。GUI には**データの置き場所だけが違う**2形態があり、使い方に合う方を選びます:

| 形態 | データ保存先 | 向いている用途 | 成果物 |
|---|---|---|---|
| **App** | `~/Library/Application Support/vlt-syslogd/` | `/Applications` に入れて日常的に使う | `vlt-syslogd-macos-app-v<ver>.zip` |
| **Portable** | `.app` 自身の隣 | USB に入れて持ち歩く・システムを汚さない | `vlt-syslogd-macos-portable-v<ver>.zip` |

どちらも管理者権限なしでどこからでも起動できます。macOS（Mojave 以降）では、標準の syslog ポート `514` を `0.0.0.0` で待ち受けるのに root は**不要**なので、ダブルクリックでそのまま使えます。（`127.0.0.1:514` のように*特定*インターフェースに bind する場合は root が必要 — 下の待ち受けポートの項を参照。）

ダウンロードしたアプリは初回起動時に Gatekeeper の quarantine が付きます。特に **Portable** 版はデータを `.app` の隣に置くため、quarantine 中は App Translocation でそれが壊れます。次のコマンドで一度だけ quarantine を外してください:

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

または初回だけアプリを右クリック →「開く」。ポートが既に使われている場合や、起動時から常駐させたい場合は Server 版を使ってください。

## macOS でのビルドと実行

GUI 版（`Portable`）は Windows / macOS で動作します。日本語フォントはプラットフォームごとに自動判定され、macOS ではヒラギノ角ゴシックを読み込みます。そのため、追加設定なしで2バイト文字が正しく表示されます。

### 前提条件

- Rust 1.85 以降（各クレートは `edition = "2024"` を使用）。

### ビルド

```bash
# GUI（単一バイナリ）。既定は App 版、feature を付けると Portable 版。
cd Portable
cargo build --release                       # App 版（データは ~/Library/Application Support）
cargo build --release --features portable   # Portable 版（データはバイナリの隣）
```

App 版・Portable 版の両方を、配布できる ad-hoc 署名済み macOS `.app` + zip として `dist/` に作るには:

```bash
./Portable/build-macos.sh
```

### 待ち受けポート

514 番は標準の syslog ポートです。macOS（Mojave 以降）では `0.0.0.0` への bind に root は**不要**で、root が要るのは `127.0.0.1` のような*特定*インターフェースへの bind だけです。Linux では 1024 未満に root または `CAP_NET_BIND_SERVICE` が必要です。bind の成否（成功・失敗）はログ表示の最初の行に表示され、失敗時は環境設定ウィンドウ（設定 →「環境設定…」）で別のポートを指定できます。

```bash
# Option A: listen on the standard port 514 with root
sudo ./target/release/vlt-syslogd-portable

# Option B: listen on a non-privileged port without root
VLT_SYSLOGD_BIND=0.0.0.0:5514 ./target/release/vlt-syslogd-portable
```

`VLT_SYSLOGD_BIND` は GUI 版の待ち受けアドレスを上書きします。

### サーバーエンジン（macOS ではコンソール常駐デーモン）

`Server` クレートは Windows ではサービスとして動作します。macOS では、同じエンジンがフォアグラウンドのコンソール常駐デーモンとしてビルド・動作します（`launchd` での管理を想定）。待ち受けアドレスは `config.toml` から読み込まれ、初回起動時に OS 標準のデータ領域へ生成されます（macOS: `/usr/local/var/vlt-syslogd`、Linux: `/var/lib/vlt-syslogd`、Windows: `C:\ProgramData\vlt-syslogd`）。場所を変えたいときは環境変数 `VLT_SYSLOGD_DATA_DIR` で上書きします。

```bash
cd Server
cargo build --release
./target/release/vlt-syslogd-srv run
```
