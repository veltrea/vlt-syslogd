# vlt-syslogd Version History

## v0.2.0 (2026-02-07) - "The Real World Provider"
Significant improvements to parsing logic based on binary analysis of Syslog messages sent from major tools such as Mac's `logger` and `nc`, and Windows' PowerShell (.NET) and MegaLog. Since it was confirmed that most sending tools do not strictly follow RFC 5424 (especially BOM requirements), we have optimized for "Tolerant Parsing" tailored to actual data structures.

### [Added / Improved]
- **Implementation of Tolerant Parsing**: Introduced logic to recognize UTF-8 packets without BOM in RFC 5424 (the new standard) as a "BOM Trap" and automatically recover them.
- **Three-Tiered Decoding Flow**: 
  1. BOM (Byte Order Mark) determination
  2. Analysis of the `charset` parameter in Structured Data (SD)
  3. Encoding estimation via statistical methods (`chardetng`)
- **Multi-Platform Verification**: Completed transmission tests from Mac (`logger`, `nc`) and Windows (PowerShell, .NET), achieving zero character corruption.
- **Technical Documentation Enhancement**: Added `DOCS/syslog_verification_report.md`. Recorded evidence of field verification.

### [Fixed]
- Fixed an issue where Shift_JIS messages sent from legacy sources like Windows were incorrectly identified as UTF-8.

---

## v0.1.0 (2026-02-01) - "The First Signal"
Initial release of the project. Basic functional implementation of the portable version.

### [Initial Features]
- Construction of a high-performance asynchronous UDP Syslog server engine in Rust.
- Real-time log monitoring via GUI (portable version).
- Support for basic parsing of RFC 5424 / RFC 3164.
- Provision of a practical and lightweight executable binary (portable version).
