# Lịch sử Phiên bản vlt-syslogd

## v0.2.0 (2026-02-07) - "The Real World Provider"
Cải thiện đáng kể logic phân tích cú pháp dựa trên phân tích nhị phân các thông báo Syslog được gửi từ các công cụ chính như `logger`, `nc` trên Mac và PowerShell (.NET), MegaLog trên Windows. Vì đã xác nhận rằng hầu hết các công cụ gửi không tuân thủ nghiêm ngặt RFC 5424 (đặc biệt là yêu cầu BOM), chúng tôi đã tối ưu hóa cho "Phân tích linh hoạt" (Tolerant Parsing) phù hợp với cấu trúc dữ liệu thực tế.

### [Thêm mới / Cải thiện]
- **Triển khai Phân tích linh hoạt**: Giới thiệu logic để nhận dạng các gói tin UTF-8 không có BOM trong RFC 5424 (tiêu chuẩn mới) là "Cạm bẫy BOM" và tự động khôi phục chúng.
- **Luồng giải mã 3 tầng**: 
  1. Xác định BOM (Byte Order Mark)
  2. Phân tích tham số `charset` trong Dữ liệu có cấu trúc (SD)
  3. Ước tính mã hóa thông qua các phương pháp thống kê (`chardetng`)
- **Xác minh đa nền tảng**: Hoàn thành các thử nghiệm truyền từ Mac (`logger`, `nc`) và Windows (PowerShell, .NET), đạt được tỷ lệ lỗi ký tự bằng không.
- **Nâng cao Tài liệu Kỹ thuật**: Thêm `DOCS/syslog_verification_report.md`. Ghi lại bằng chứng xác minh thực địa.

### [Sửa lỗi]
- Sửa lỗi các thông báo Shift_JIS dik gửi từ các nguồn cũ như Windows bị nhận dạng sai thành UTF-8.

---

## v0.1.0 (2026-02-01) - "The First Signal"
Phát hành lần đầu của dự án. Triển khai chức năng cơ bản của phiên bản portable.

### [Tính năng ban đầu]
- Xây dựng công cụ máy chủ Syslog UDP không đồng bộ hiệu suất cao bằng Rust.
- Giám sát nhật ký thời gian thực thông qua GUI (phiên bản portable).
- Hỗ trợ phân tích cú pháp cơ bản của RFC 5424 / RFC 3164.
- Cung cấp tệp thực thi nhẹ và thực tế (phiên bản portable).
