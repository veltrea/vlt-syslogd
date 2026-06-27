# vlt-syslogd Version History

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
