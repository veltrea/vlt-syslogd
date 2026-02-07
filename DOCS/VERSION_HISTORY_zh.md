# vlt-syslogd 版本历史

## v0.2.0 (2026-02-07) - "The Real World Provider"
基于对 Mac 的 `logger`、`nc` 以及 Windows 的 PowerShell (.NET)、MegaLog 等主流工具发送的 Syslog 消息进行的二进制分析，大幅改进了解析逻辑。由于确认了大多数发送工具并未严格遵守 RFC 5424（特别是 BOM 要求），我们已针对实际数据结构优化了“宽容解析 (Tolerant Parsing)”。

### [新增 / 改进]
- **实现宽容解析**: 在 RFC 5424（新标准）中，引入了将无 BOM 的 UTF-8 数据包识别为“BOM 陷阱”并自动还原的逻辑。
- **三层解码流程**: 
  1. BOM (字节顺序标记) 判定
  2. Structured Data (SD) 的 `charset` 参数分析
  3. 通过统计推定 (`chardetng`) 推测编码
- **多平台验证**: 完成了从 Mac (`logger`, `nc`) 和 Windows (PowerShell, .NET) 的发送测试，实现零乱码。
- **完善技术文档**: 添加了 `DOCS/syslog_verification_report.md`。记录了实地验证证据。

### [修复]
- 修复了从 Windows 等旧版源发送的 Shift_JIS 消息被误判为 UTF-8 的问题。

---

## v0.1.0 (2026-02-01) - "The First Signal"
项目初始发布。便携版的基准功能实现。

### [初始特性]
- 使用 Rust 构建高性能异步 UDP Syslog 服务器引擎。
- 通过 GUI（便携版）进行实时日志监控。
- 支持 RFC 5424 / RFC 3164 的基本解析。
- 提供实用且轻量级的可执行二进制文件（便携版）。
