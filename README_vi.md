# vlt-syslog-server (Workspace)

Giải pháp Syslog dành cho Windows, được thiết kế để mang lại khả năng hiển thị cao và tính thực tiễn trong môi trường ký tự đa byte (CJK/Tiếng Nhật).

## Cấu trúc và Mục đích của Dự án

Dự án này được tổ chức thành hai thành phần để phục vụ các trường hợp sử dụng khác nhau.

### 1. [Portable](./Portable) (Đã phát hành v0.2.0)
- **Trạng thái**: Đã phát hành. Việc bảo trì sẽ tiếp tục chừng nào sức khỏe và năng lượng còn cho phép.
- **Khái niệm**: "Con dao đa năng của kỹ sư dành cho ổ USB của bạn".
- **Tính năng**: Một tệp thực thi duy nhất tích hợp cả GUI và công cụ máy chủ. Không yêu cầu cài đặt hoặc truy cập internet, mang lại tính cơ động tối ưu cho việc gỡ lỗi tại chỗ và thu thập nhật ký tạm thời.
- **Triết lý**: Được tạo ra bởi một kỹ sư bận rộn, người đã nghĩ rằng: "Cái này sẽ rất tiện lợi". Tôi hy vọng các kỹ sư đồng nghiệp sẽ thấy nó hữu ích và dễ dàng triển khai.

### 2. [Server](./Server) (Đang phát triển)
- **Trạng thái**: Hiện đang được phát triển như một cột mốc tiếp theo.
- **Khái niệm**: "Vận hành ổn định cấp chuyên nghiệp cho Máy chủ Windows".
- **Tính năng**: Tách biệt công cụ chạy ngầm (chạy như một Dịch vụ Windows) khỏi giao diện giám sát (chỉ kết nối khi cần thiết).
- **Mục tiêu**: Cung cấp cơ sở hạ tầng thu thập nhật ký mạnh mẽ cho các môi trường sản xuất yêu cầu hoạt động 24/7.

---

## Bối cảnh phát triển

Được xây dựng từ đầu bằng ngôn ngữ Rust để khắc phục "hạn chế hiển thị tiếng Nhật/tiếng Hoa" của các máy chủ Syslog Windows nhẹ hiện có. Nó có tính năng "Tolerant Parsing" (Phân tích cú pháp linh hoạt) để xử lý chính xác các biến thể dữ liệu thực tế, bao gồm các bảng mã khác nhau như Shift_JIS và UTF-8 có hoặc không có BOM.

Để biết chi tiết, vui lòng tham khảo README trong mỗi thư mục.
