# vlt-syslogd 版本歷史

## v0.2.0 (2026-02-07) - "The Real World Provider"
基於對 Mac 的 `logger`、`nc` 以及 Windows 的 PowerShell (.NET)、MegaLog 等主流工具發送的 Syslog 訊息進行的二進位分析，大幅改進了解析邏輯。由於確認了大多數發送工具並未嚴格遵守 RFC 5424（特別是 BOM 要求），我們已針對實際數據結構優化了「寬容解析 (Tolerant Parsing)」。

### [新增 / 改進]
- **實現寬容解析**: 在 RFC 5424（新標準）中，引入了將無 BOM 的 UTF-8 封包識別為「BOM 封包陷阱」並自動還原的邏輯。
- **三層解碼流程**: 
  1. BOM (位元組順序標記) 判定
  2. Structured Data (SD) 的 `charset` 參數分析
  3. 通過統計推定 (`chardetng`) 推測編碼
- **多平台驗證**: 完成了從 Mac (`logger`, `nc`) 和 Windows (PowerShell, .NET) 的發送測試，實現零亂碼。
- **完善技術文件**: 添加了 `DOCS/syslog_verification_report.md`。記錄了實地驗證證據。

### [修復]
- 修復了從 Windows 等舊版源發送的 Shift_JIS 訊息被誤判為 UTF-8 的問題。

---

## v0.1.0 (2026-02-01) - "The First Signal"
專案初始發佈。可攜版的基礎功能實現。

### [初始特性]
- 使用 Rust 建構高性能非同步 UDP Syslog 伺服器引擎。
- 通過 GUI（可攜版）進行即時日誌監控。
- 支援 RFC 5424 / RFC 3164 的基本解析。
- 提供實用且輕量級的執行二進位檔案（可攜版）。
