# vlt-syslogd

**"Syslog messages should be UTF-8. Future-proof, lightweight, and international."**

## 哲学とスローガン
- **UTF-8 Only**: 「Syslog メッセージは UTF-8 で送るべきである」という強い信念のもと、レガシーな文字コード（SJIS/EUC等）のサポートをあえて排除します。
- **No Legacy Baggage**: 過去のしがらみを捨て去ることで、実装のシンプルさと高い保守性、そして圧倒的な開発スピードを実現します。
- **International by Default**: 最初から UTF-8 に一本化することで、追加コストなしに多言語・国際対応が完了した状態を提供します。

## 概要
`vlt-syslogd` は、Windows 環境でも日本語や多言語が正しく表示・保存できる、Rust 製のモダンな Syslog サーバーです。軽量かつ高速で、無駄な変換処理による文字化けのリスクをゼロにします。

## 主な機能 (Outline)
1. **高性能 UDP リスナー**:
   - `tokio` による非同期 I/O を採用。
   - 大量のログを受信しても UI がフリーズしない設計。
2. **リアルタイム・日本語表示 UI**:
   - `egui` によるモダンでレスポンシブなインターフェース。
   - UTF-8 日本語メッセージが「そのまま」化けずに表示される。
3. **構造化ロギング**:
   - 受信した RFC 5424/3164 形式のログをパースし、見やすいテーブル形式で表示。
4. **自動ログ保存**:
   - 受信したログを自動的にファイルへ書き出し。
   - 日付やサイズによるローテーション機能（予定）。

## 技術スタック
- **Language**: Rust
- **Async Runtime**: Tokio
- **GUI Framework**: eframe / egui
- **DateTime**: Chrono

## 開発ロードマップ (Status)
- [x] Phase 1: UDP リスナーと基本 UI の結合 (Completed)
- [x] Phase 2: Syslog プロトコルのパースロジック実装 (Completed)
- [x] Phase 3: ログフィルタリングと検索機能 (Completed)
- [x] Phase 4: ファイル保存・永続化 (Completed)

## 使い方 (Usage)
1. アプリケーションを起動すると、自動的に `0.0.0.0:514` (UDP) で待機を開始します。
2. ネットワーク機器やサーバーからこの PC の IP アドレス宛に Syslog を送信してください。
3. 受信したログは画面にリアルタイム表示され、`logs/` ディレクトリ配下に自動保存されます。
   - ログファイル形式: `logs/syslog_YYYYMMDD_HHMMSS.log`

## ビルド方法 (Build)

### 前提条件
- Rust (Cargo)
- Python 3 (アイコン埋め込み用)

### 手順
1. リリースビルドの作成:
   ```powershell
   cargo build --release
   ```

2. アイコンの埋め込み (Windows用):
   ビルド後、以下のスクリプトを実行して `.exe` にアイコンを適用します。
   ```powershell
   python embed_icon.py
   ```
   > **Note**: `target/release/vlt-syslogd.exe` が更新されます。

