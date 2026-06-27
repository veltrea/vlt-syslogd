# vlt-syslog-srv (Server Engine)

Windows サーバーで 24 時間安定稼働することを目的とした、サービス版 Syslog サーバーエンジン。

## 機能
- **Windows Service 対応**: バックグラウンドでの常時稼働。
- **高信頼パース**: `v0.2.0` 世代の「寛容なパースロジック」を搭載。
- **ファイル出力**: 構造化されたログを専門のロガー（flexi_logger）経由で永続化。

## インストール方法 (管理者権限が必要)

現時点では `sc.exe` を使用して手動で登録します。

### 1. サービスの登録
```cmd
sc create vlt-syslog-srv binPath= "C:\path\to\vlt-syslog-srv.exe" start= auto
```

### 2. サービスの開始
```cmd
sc start vlt-syslog-srv
```

### 3. 設定とログ
ログはデフォルトで実行ファイルと同じディレクトリの `logs/` に生成されます。

## 開発用コマンド
コンソール上でデバッグ実行する場合は以下のコマンドを使用します。
```powershell
./vlt-syslog-srv.exe run
```
