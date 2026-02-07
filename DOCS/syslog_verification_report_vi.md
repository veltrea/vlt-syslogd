# Báo cáo Xác minh Syslog: Hành vi Mã hóa trong các Công cụ Thực tế

## 1. Mục đích Xác minh
Để chứng minh sự khác biệt giữa đặc tả RFC 5424 (yêu cầu BOM cho UTF-8) và hành vi thực tế của các công cụ gửi Syslog, và xác nhận tính hiệu quả của "Phân tích linh hoạt" (Tolerant Parsing) đã được triển khai.

## 2. Môi trường Xác minh
- **Máy chủ**: `vlt-syslog-portable` (Mac 192.168.1.22)
- **Khách 1 (Mac)**: `nc` (netcat), `logger`
- **Khách 2 (Windows)**: `LLM-SVR1` (PowerShell .NET)

## 3. Kết quả Xác minh: Phân tích "Dữ liệu Thô" từ Máy thực tế

Phân tích các gói tin nhận được (HEX) được ghi lại trong `logs/debug_raw.log`.

| Công cụ gửi / Định dạng | Hành vi mong đợi (RFC) | Hành vi thực tế (RAW) | Kết quả | Ghi chú |
| :--- | :--- | :--- | :--- | :--- |
| **Mac `nc` (UTF-8)** | Có BOM | **Không có BOM** | ✅ UTF-8 (Implicit) | Quên BOM là phổ biến ngay cả khi gửi thủ công |
| **Windows PowerShell** | Có BOM | **Không có BOM** | ✅ UTF-8 (Implicit) | Xác nhận thiếu BOM do hành vi tiêu chuẩn của .NET |
| **MegaLog / Các SD khác** | Như chỉ định | Như chỉ định | ✅ Các Charset khác | Tham số `charset` của SD có độ tin cậy cao |
| **Legacy (RFC 3164)** | Không quy định | Không BOM (SJIS/UTF8) | ✅ Shift_JIS (Guess) | Khôi phục chính xác thông qua ước tính thống kê |

## 4. Nhận định về "Cạm bẫy BOM" và Giải pháp
Xác minh này đã chỉ ra rõ ràng rằng **"Việc phân tích chỉ dựa vào sự hiện diện của BOM sẽ không hoạt động trong thế giới thực."**

### Thực trạng
- Trong các lần truyền Windows tiêu chuẩn (PowerShell, v.v.), thông thường không có BOM ngay cả đối với UTF-8.
- Nếu chúng ta coi đây là "Không có BOM, do đó nó là SJIS (hoặc dữ liệu không hợp lệ)", tất cả nhật ký từ các hệ thống hiện đại sẽ bị lỗi hiển thị.

### Giải pháp của chúng tôi (Logic 3 tầng)
1. **Phát hiện BOM**: Nếu có, xử lý là 100% UTF-8.
2. **Phân tích SD (Structured Data)**: Phân tích các thẻ `charset` (`UTF-8`, `MSG-UTF8`, `Shift_JIS`, v.v.) để khôi phục.
3. **Ước tính thống kê (Chốt chặn cuối cùng)**: Sử dụng `chardetng` để đưa ra phán đoán tốt nhất cho môi trường CJK/Việt Nam dựa trên đặc điểm chuỗi byte.

## 5. Kết luận
Đã chứng minh rằng `vlt-syslog-portable` có khả năng phục hồi cực cao trước "dữ liệu không hoàn hảo" trong thế giới thực trong khi vẫn tôn trọng các lý tưởng của RFC. Điều này cho phép giám sát nhật ký ổn định mà không bị hỏng ký tự ngay cả trong môi trường đa nền tảng hỗn hợp.

---

> [!IMPORTANT]
> **Yêu cầu gửi tới cộng đồng người dùng quốc tế**
> Hiện tại, môi trường thử nghiệm của nhà phát triển chỉ giới hạn ở **tiếng Nhật**. Mặc dù chúng tôi đã xác nhận hỗ trợ cho các bảng mã chính, chúng tôi sẽ rất biết ơn nếu bạn báo cáo nếu gặp bất kỳ vấn đề hiển thị hoặc "lỗi chữ" nào trong khu vực cụ thể của bạn (Tiếng Việt, Tiếng Anh, Tiếng Hàn, Tiếng Trung, v.v.). Phản hồi của bạn giúp chúng tôi hoàn thiện công cụ này cho tất cả mọi người!
