# Veltrea Syslog Server

[English](README.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh.md) · [繁體中文](README.tw.md) · Tiếng Việt

**Mỗi thông điệp được giải mã riêng theo Shift_JIS hoặc UTF-8. Nhiều bảng mã trên cùng một cổng vẫn đọc được, không lỗi chữ.**

**[Trang web (English)](https://veltrea.github.io/vlt-syslogd/index.en.html)** · [日本語](https://veltrea.github.io/vlt-syslogd/) · **[Bản phát hành v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)**

![Cửa sổ nhật ký Portable. Cột Enc hiện UTF-8, Shift_JIS và Shift_JIS (MSG-SD/BOM-Missing) trên cùng một màn hình](DOCS/assets/portable-main.png)

*Cùng một ảnh với trang web: bản Windows v0.4.0. Cột Enc là quyết định của dòng đó — `UTF-8`, `Shift_JIS`, hoặc `Shift_JIS (MSG-SD/BOM-Missing)` khi dữ liệu có cấu trúc RFC 5424 khai báo charset. Bấm chuột phải vào thông điệp và chọn **Copy as Hex** để sao chép byte gốc dạng hex.*

Bản mới nhất là **v0.4.0**, cho Windows, macOS và Linux (x86_64). Nếu chưa chắc tải tệp nào, hãy bắt đầu bằng **Portable**.

Chữ trên cửa sổ là tiếng Nhật. Trên Windows và Linux, menu được ghi song ngữ: `ファイル(File)`, `編集(Edit)`, `ヘルプ(Help)`.

## Ba dạng

Phần đọc syslog cùng một cách làm. Khác nhau ở chỗ bạn chạy nó thế nào.

| | Portable | Server | Console |
|---|---|---|---|
| Tệp thực thi | `vlt-syslogd-portable` | `vlt-syslogd-srv` | `vlt-syslogd-console` |
| Là gì | GUI tự mở socket UDP. Không trình cài đặt, không dịch vụ. | Engine không giao diện. Chạy như Windows Service, launchd LaunchDaemon hoặc systemd. Nhận từ lúc khởi động và ghi ra đĩa. | GUI gắn vào Server đang chạy qua TCP. Đóng cửa sổ không dừng việc thu thập. Nút chạy, dừng và khởi động lại dịch vụ nằm ở đây. |
| Cổng | UDP, mặc định `0.0.0.0:514` | UDP `514` (syslog). TCP `5141` (luồng tới Console) và TCP `5142` (điều khiển), mặc định `127.0.0.1` | Kết nối tới địa chỉ luồng và địa chỉ điều khiển của Server |
| Có trong bản v0.4.0 | Windows, macOS, Linux x86_64 | Windows, macOS. Linux: dựng từ mã nguồn (`Server/install-linux.sh` nằm trong kho; bản này không có binary Server cho Linux) | Windows, macOS. Linux: dựng từ mã nguồn |

**App** và **Portable** là cùng một GUI. Chúng chỉ khác chỗ giữ dữ liệu.

| Dạng | Thư mục dữ liệu |
|---|---|
| **App** | Windows `%APPDATA%\vlt-syslogd`, macOS `~/Library/Application Support/vlt-syslogd`, Linux `$XDG_DATA_HOME/vlt-syslogd` hoặc `~/.local/share/vlt-syslogd` |
| **Portable** | Cạnh tệp thực thi. Trên macOS, cạnh file `.app` (để trên USB, không ghi vào hệ thống) |

`VLT_SYSLOGD_DATA_DIR` ghi đè thư mục đó cho cả hai dạng. Server dùng thư mục hệ thống: `C:\ProgramData\vlt-syslogd`, `/usr/local/var/vlt-syslogd`, hoặc `/var/lib/vlt-syslogd`.

Cửa sổ tùy chọn (**ファイル → 環境設定…**) có hai ô: cổng lắng nghe (**待ち受けポート**) và thư mục nhật ký (**ログ保存先**). **適用** ghi chúng lại.

## Tải v0.4.0 và gửi một thông điệp

Các tệp dưới đây là [tài sản của bản v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0). Gửi thông điệp thử tới cổng được ghi ở dòng nhật ký đầu tiên. `Listening on 0.0.0.0:514 (UDP)` nghĩa là cổng đó đang mở.

Muốn chạy thường trực, hoặc nhận từ máy khác, xem [hướng dẫn cài đặt](DOCS/INSTALL.md).

### Windows

| Dạng | Tệp |
|---|---|
| Portable | [vlt-syslogd-portable-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-portable-windows-v0.4.0.zip) → `vlt-syslogd-portable.exe` |
| App | [vlt-syslogd-app-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-app-windows-v0.4.0.zip) → `vlt-syslogd-windows-x86_64-app-v0.4.0.exe` |
| Server | [vlt-syslogd-srv-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-windows-v0.4.0.zip) → `vlt-syslogd-srv.exe` cùng `install-windows.ps1` |
| Console | [vlt-syslogd-console-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-windows-v0.4.0.zip) → `vlt-syslogd-console.exe` |

Giải nén và bấm đúp exe Portable. Nếu SmartScreen hiện ra, chọn **More info**, rồi **Run anyway**. Gắn cổng UDP 514 không cần quyền quản trị.

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::UTF8.GetBytes('<132>auth: 認証に失敗しました')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

Một dòng mới xuất hiện là đã tới. Nội dung này là UTF-8 không BOM, đúng dạng chương trình này vẫn nhận.

### macOS

| Dạng | Tệp |
|---|---|
| Portable | [vlt-syslogd-macos-portable-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.dmg) hoặc [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.zip) → `vlt-syslogd-portable.app` |
| App | [vlt-syslogd-macos-app-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.dmg) hoặc [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.zip) → `vlt-syslogd.app` |
| Server | [vlt-syslogd-srv-macos-v0.4.0.tar.gz](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-macos-v0.4.0.tar.gz) → `vlt-syslogd-srv` cùng `install-macos.sh` |
| Console | [vlt-syslogd-console-macos-app-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-macos-app-v0.4.0.zip) → `vlt-syslogd-console.app` |

Bản dựng được ký ad-hoc, chưa công chứng. Gatekeeper gắn cờ cách ly cho ứng dụng vừa tải. Gỡ một lần, hoặc bấm chuột phải vào ứng dụng và chọn **Open**.

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

Portable giữ dữ liệu cạnh file `.app`. Cờ cách ly còn đó thì App Translocation khiến ứng dụng không thấy thư mục đó. Phông CJK là Hiragino, có sẵn trên macOS.

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

`logger` trên macOS ghi vào nhật ký hệ thống cục bộ. Nó không nhận máy chủ và cổng UDP. Ở đây dùng `nc`. Xem [Cổng 514](#cổng-514) trước khi coi như 514 đã mở.

### Linux (x86_64)

Bản phát hành chỉ có GUI. Không có binary Server hay Console dựng sẵn; cần thì dựng từ mã nguồn ([BUILD.md](DOCS/BUILD.md), [INSTALL.linux.md](DOCS/INSTALL.linux.md)).

| Dạng | Tệp |
|---|---|
| Portable | [vlt-syslogd-linux-x86_64-portable-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-portable-v0.4.0) |
| App | [vlt-syslogd-linux-x86_64-app-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-app-v0.4.0) |

Đây là ELF x86_64 liên kết động. Mở cửa sổ cần một màn hình (X11 hoặc Wayland). Ghi chú phát hành v0.4.0 cũng yêu cầu OpenGL.

```bash
chmod +x vlt-syslogd-linux-x86_64-portable-v0.4.0
./vlt-syslogd-linux-x86_64-portable-v0.4.0
```

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

`logger` của util-linux gửi được UDP. `logger` của macOS thì không. Dạng dưới đây là cho Linux:

```bash
logger -n 127.0.0.1 -P 514 -d 'hello from logger'
```

Chữ Nhật cần một phông CJK mà GUI biết đường tìm: Noto Sans CJK (`NotoSansCJK-Regular.ttc` trong `/usr/share/fonts`) hoặc IPA Gothic (`/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf`). Không có thì những ký tự đó thành □. Gói `fonts-noto-cjk` trên Debian/Ubuntu đặt Noto vào đường dẫn đầu tiên trong số đó.

## Điều cần biết trước

### Cổng 514

Dòng nhật ký đầu tiên là kết quả bind. Dòng báo lỗi nghĩa là socket chưa mở, nên thông điệp thử gửi tới 514 sẽ không hiện.

- **Windows.** Gắn UDP 514 không cần quản trị viên. Trình cài chỉ xin quyền quản trị khi đăng ký dịch vụ.
- **Linux.** Cổng dưới 1024 cần root hoặc `CAP_NET_BIND_SERVICE`. Với binary Portable:

  ```bash
  sudo setcap 'cap_net_bind_service=+ep' ./vlt-syslogd-linux-x86_64-portable-v0.4.0
  ```

  `Server/install-linux.sh` cài một unit systemd chạy bằng root. Trong file unit có một phương án bị chú thích, dùng `AmbientCapabilities=CAP_NET_BIND_SERVICE`.
- **macOS.** Một lần thử không phải root gắn `0.0.0.0:514` nhận `EACCES`, ghi trong [DOCS/portable_macos_port_behavior.md](DOCS/portable_macos_port_behavior.md). Dòng gợi ý trong ứng dụng vẫn nói bind wildcard trên macOS không cần root. Hãy tin dòng nhật ký đầu tiên.

Khi bind thất bại, banner điền sẵn **5514**. **待ち受け開始** lắng nghe cổng đó trong phiên này và không lưu lại. Muốn giữ, mở **環境設定** rồi bấm **適用**, hoặc chạy với `VLT_SYSLOGD_BIND=0.0.0.0:5514`. Gửi thông điệp thử tới cổng mà dòng nhật ký nêu. Muốn giữ 514 trên macOS, hãy mở Portable bằng quyền quản trị, hoặc cài Server (LaunchDaemon chạy bằng root, `com.veltrea.vlt-syslogd-srv`).

### Gatekeeper trên macOS

Không có tài khoản Apple Developer đứng sau các bản dựng này. Lần mở đầu bị chặn cho đến khi bạn bấm chuột phải → **Open**, hoặc xóa `com.apple.quarantine` như trên.

### Dịch vụ đang chạy vẫn có thể không nhận được gì

Thiết bị từ xa chỉ tới được Server khi `bind_addr` là `0.0.0.0` hoặc một địa chỉ LAN **và** tường lửa cho phép UDP 514. Luồng (`5141`) và điều khiển (`5142`) ở lại loopback cho đến khi bạn đổi. Mở hai cổng đó ra khỏi máy cũng mở `set_config`, lệnh có thể viết lại cấu hình Server. Hướng dẫn cài đặt khuyên dùng đường hầm SSH cho Console ở máy khác.

```powershell
# Windows, PowerShell được nâng quyền
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

```bash
# Linux
sudo ufw allow 514/udp
# firewalld:
sudo firewall-cmd --permanent --add-port=514/udp && sudo firewall-cmd --reload
```

Trên macOS, tường lửa ứng dụng lọc theo từng ứng dụng. Khi được hỏi, cho phép kết nối đến `vlt-syslogd-srv`.

## Bối cảnh

Viết bằng Rust, từ đầu, vì các máy chủ syslog Windows gọn nhẹ mà tác giả vẫn dùng không hiển thị tiếng Nhật cho ra hồn. Mỗi datagram được giải mã riêng: BOM, tham số `charset` trong dữ liệu có cấu trúc RFC 5424, hoặc chính các byte (kể cả ước lượng Shift_JIS bằng `chardetng`). Cột Enc là quyết định đó. Việc bảo trì tiếp tục chừng nào sức khỏe và sức lực còn cho phép. Tác giả là veltrea.

## Tài liệu

| | |
|---|---|
| Cài đặt | [INSTALL.md](DOCS/INSTALL.md) — [Windows](DOCS/INSTALL.windows.md) · [macOS](DOCS/INSTALL.macos.md) · [Linux](DOCS/INSTALL.linux.md) |
| Cổng và tên dịch vụ | [REFERENCE.md](DOCS/REFERENCE.md) |
| Dựng | [BUILD.md](DOCS/BUILD.md) — `cargo build --release --workspace` (edition 2024) |
| Ghi chú về bảng mã | [syslog_verification_report.vi.md](DOCS/syslog_verification_report.vi.md) |

## Giấy phép

[MIT](LICENSE). Copyright (c) 2026 veltrea.
