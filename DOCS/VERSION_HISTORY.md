# vlt-syslogd Version History

## v0.3.0 (2026-06-27)
クロスプラットフォーム対応を中心とした機能拡張版。v0.2.0 のパース基盤の上に、macOS / Linux サポートと運用機能を追加した最初の正式公開リリース。

### [Added]
- **クロスプラットフォーム対応**: macOS / Linux でのビルドと動作に対応。バインドアドレスを設定可能に。
- **設定ファイル対応**: Server 版で設定ファイルによる構成をサポート。
- **非同期ロギング**: Server 版のログ出力を非同期化。Windows では `%ProgramData%` 配下への業務向けロギングに対応。
- **パニックハンドリング**: 想定外の障害時にプロセスが安全に振る舞うよう改善。

### [Notes]
- v0.2.0（パースロジックの大幅改善、Tolerant Parsing）の内容はすべて含む。
- 本バージョンが、クリーンな公開リポジトリでの最初のタグ付きリリース。

## v0.2.0 (2026-02-07) - "The Real World Provider"
Macの `logger`, `nc` や Windowsの PowerShell (.NET), MegaLog 等の主要ツールから送信される Syslog メッセージのバイナリ解析結果に基づいた、パースロジックの大幅な改善。大半の送信ツールが RFC 5424（特にBOM要件）に厳密に従っていない実態を確認したため、現実のデータ構造に即した「寛容なパース」へと最適化しました。

### [Added / Improved]
- **Tolerant Parsingの実装**: RFC 5424（新規格）において、BOMがないUTF-8パケットを「BOM Trap」として認識し、自動的に救済するロジックを導入。
- **三段構えのデコードフロー**: 
  1. BOM (Byte Order Mark) 判定
  2. Structured Data (SD) の `charset` パラメータ解析
  3. 統計的推定 (`chardetng`) によるエンコーディング推測
- **マルチプラットフォーム検証**: Mac (`logger`, `nc`) および Windows (PowerShell, .NET) からの送信テストを完了し、文字化けゼロを達成。
- **技術ドキュメントの整備**: `DOCS/syslog_verification_report.md` を追加。実地検証のエビデンスを記録。

### [Fixed]
- Windows等のレガシーなソースから送られるShift_JISメッセージが、UTF-8として誤判定される問題を修正。

---

## v0.1.0 (2026-02-01) - "The First Signal"
プロジェクトの初期リリース。ポータブル版の基本機能実装。

### [Initial Features]
- Rustによる高性能な非同期UDP Syslogサーバーエンジンの構築。
- GUI（ポータブル版）によるリアルタイムログモニタリング。
- RFC 5424 / RFC 3164 の基本的なパースへの対応。
- 実用的で軽量な実行バイナリ（ポータブル版）の提供。
