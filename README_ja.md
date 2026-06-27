# vlt-syslog-server (Workspace)

2バイト文字(CJK)環境での高い視認性と実用性を追求した、Windows / macOS / Linux 対応 Syslog ソリューション。

## プロジェクトの構成と目的

本プロジェクトでは、用途に合わせて 2 つの形態を提供するためにディレクトリを分離・整理しました。

### 1. [Portable](./Portable) (Released v0.2.0)
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

## macOS / Linux でのビルドと実行

GUI 版（ルートクレートと `Portable`）は Windows / macOS / Linux で動作します。日本語フォントはプラットフォームごとに自動判定され、macOS ではヒラギノ角ゴシック、Linux では Noto Sans CJK / IPA フォントを読み込みます。そのため、追加設定なしで2バイト文字が正しく表示されます。

### 前提条件

- Rust 1.85 以降（各クレートは `edition = "2024"` を使用）。

### ビルド

```bash
# Portable (GUI + server engine in a single binary — recommended)
cd Portable
cargo build --release        # binary: target/release/vlt-syslog-portable

# Root crate (the basic, UTF-8 only GUI)
cargo build --release        # from the repository root; binary: target/release/vlt-syslogd
```

### 待ち受けポート（514 は macOS / Linux で root 権限が必要）

514 番は標準の syslog ポートですが、macOS と Linux では 1024 未満は特権ポートのため root 権限が必要です。bind の成否（成功・失敗）はログ表示の最初の行に表示されます。

```bash
# Option A: listen on the standard port 514 with root
sudo ./target/release/vlt-syslog-portable

# Option B: listen on a non-privileged port without root
VLT_SYSLOGD_BIND=0.0.0.0:5514 ./target/release/vlt-syslog-portable
```

`VLT_SYSLOGD_BIND` は両方の GUI 版で待ち受けアドレスを上書きします。

### サーバーエンジン（macOS / Linux ではコンソール常駐デーモン）

`Server` クレートは Windows ではサービスとして動作します。macOS と Linux では、同じエンジンがフォアグラウンドのコンソール常駐デーモンとしてビルド・動作します（`launchd` や `systemd` での管理を想定）。待ち受けアドレスは `config.toml`（初回起動時にカレントディレクトリへ生成）から読み込まれます。

```bash
cd Server
cargo build --release
./target/release/vlt-syslog-srv run
```
