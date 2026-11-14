# Veltrea Syslog Server — Server エンジン

*パッケージ名: `vlt-syslogd-srv` · v0.4.0*

画面を持たない常駐エンジンです。Windows サービス、launchd、systemd として動き、UDP で syslog を受けてファイルへ書きます。表示は [Console](../Console)（`vlt-syslogd-console`）が TCP で接続して行います。全体の説明は [リポジトリの README](../README.ja.md) にあります。

パースは Portable と同じで、1 通ごとに符号化を決めます。ログの書き出しには `flexi_logger` を使います。

## 既定のポート

| ポート | 既定の bind | 用途 |
|---|---|---|
| 514/udp | `0.0.0.0` | syslog の受信 |
| 5141/tcp | `127.0.0.1` | Console への配信 |
| 5142/tcp | `127.0.0.1` | 制御（`get_config` / `set_config`） |

データと `config.toml` の既定位置:

- Windows: `C:\ProgramData\vlt-syslogd`
- macOS: `/usr/local/var/vlt-syslogd`
- Linux: `/var/lib/vlt-syslogd`

`VLT_SYSLOGD_DATA_DIR` があればそちらを使います。初回起動時に `config.toml` が無ければ作られます。

## インストール

OS ごとの手順はインストールガイドにまとまっています。

- [Windows](../DOCS/INSTALL.windows.ja.md)
- [macOS](../DOCS/INSTALL.macos.ja.md)
- [Linux](../DOCS/INSTALL.linux.ja.md)

v0.4.0 のリリースにあるビルド済みバイナリは、Windows（`vlt-syslogd-srv-windows-v0.4.0.zip`。中に `install-windows.ps1`）と macOS（`vlt-syslogd-srv-macos-v0.4.0.tar.gz`。中に `install-macos.sh`）です。Linux 用のビルド済み Server はこのリリースにありません。ソースからビルドして `install-linux.sh` を使います。

サービスに登録せず、フォアグラウンドで動かすときは `run` を付けます。引数なしのとき、Windows はサービスディスパッチャを待ち、macOS と Linux はそのままフォアグラウンドで受けます。

```powershell
./vlt-syslogd-srv.exe run
```

```bash
./vlt-syslogd-srv run
```

514 番は、Linux では root または `CAP_NET_BIND_SERVICE` が要ります。macOS では Server を root の LaunchDaemon として入れるので bind できます。Windows で 514 を bind すること自体に管理者権限は要りません。要るのはサービスの登録です。リモートから届かないときは、ファイアウォールが UDP 514 を通しているかを先に見てください。

## ライセンス

[MIT](../LICENSE)。Copyright (c) 2026 veltrea。
