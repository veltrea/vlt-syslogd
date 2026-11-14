# Veltrea Syslog Server

[English](README.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · 简体中文 · [繁體中文](README.tw.md) · [Tiếng Việt](README.vi.md)

**每条消息单独判定 Shift_JIS 或 UTF-8 再解码。同一端口上编码混在一起，也不会变成乱码。**

**[网站（English）](https://veltrea.github.io/vlt-syslogd/index.en.html)** · [日本語](https://veltrea.github.io/vlt-syslogd/) · **[发行版 v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)**

![Portable 日志窗口。Enc 列同时出现 UTF-8、Shift_JIS 和 Shift_JIS (MSG-SD/BOM-Missing)](DOCS/assets/portable-main.png)

*与网站上的是同一张图：v0.4.0 的 Windows 构建。Enc 列是这一行的判定结果，会出现 `UTF-8`、`Shift_JIS`，以及 RFC 5424 结构化数据声明了字符集时的 `Shift_JIS (MSG-SD/BOM-Missing)`。在消息上右键，选 **Copy as Hex**，即可把原始字节复制成十六进制。*

最新发行版是 **v0.4.0**，面向 Windows、macOS 和 Linux（x86_64）。拿不准下哪个文件时，从 **Portable** 开始。

窗口里的文字是日语。Windows 和 Linux 的菜单写成 `ファイル(File)`、`編集(Edit)`、`ヘルプ(Help)`。

## 三种形态

读取 syslog 的思路相同，差别只在怎么运行。

| | Portable | Server | Console |
|---|---|---|---|
| 二进制 | `vlt-syslogd-portable` | `vlt-syslogd-srv` | `vlt-syslogd-console` |
| 是什么 | 自己打开 UDP 套接字的 GUI。不用安装，也不用注册服务。 | 没有界面的引擎。作为 Windows 服务、launchd LaunchDaemon 或 systemd 运行。开机即接收，并写入磁盘。 | 通过 TCP 连到正在运行的 Server 的 GUI。关掉窗口，收集并不停止。服务的启动、停止和重启也在这个窗口里。 |
| 端口 | UDP，默认 `0.0.0.0:514` | UDP `514`（syslog）。TCP `5141`（送给 Console 的流）和 TCP `5142`（控制）默认绑在 `127.0.0.1` | 连接到 Server 的流地址和控制地址 |
| v0.4.0 的发布文件 | Windows、macOS、Linux x86_64 | Windows、macOS。Linux 需从源码构建（仓库里有 `Server/install-linux.sh`；此发行版没有 Linux 的 Server 二进制） | Windows、macOS。Linux 需从源码构建 |

**App** 和 **Portable** 是同一个 GUI，只是数据放的位置不同。

| 形态 | 数据目录 |
|---|---|
| **App** | Windows 为 `%APPDATA%\vlt-syslogd`，macOS 为 `~/Library/Application Support/vlt-syslogd`，Linux 为 `$XDG_DATA_HOME/vlt-syslogd` 或 `~/.local/share/vlt-syslogd` |
| **Portable** | 可执行文件旁边。macOS 上在 `.app` 旁边（放进 U 盘带走，不往系统里写） |

两种形态都认 `VLT_SYSLOGD_DATA_DIR`。Server 则使用系统目录：`C:\ProgramData\vlt-syslogd`、`/usr/local/var/vlt-syslogd` 或 `/var/lib/vlt-syslogd`。

偏好设置（**ファイル → 環境設定…**）只有两项：监听端口（**待ち受けポート**）和日志目录（**ログ保存先**）。按 **適用** 写入。

## 下载 v0.4.0，发一条消息

下面的文件是 [v0.4.0 的发行资产](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)。测试消息发到日志第一行写出的端口。出现 `Listening on 0.0.0.0:514 (UDP)` 表示该端口已打开。

要常驻，或从其他设备接收，见[安装指南](DOCS/INSTALL.md)。

### Windows

| 形态 | 文件 |
|---|---|
| Portable | [vlt-syslogd-portable-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-portable-windows-v0.4.0.zip) → `vlt-syslogd-portable.exe` |
| App | [vlt-syslogd-app-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-app-windows-v0.4.0.zip) → `vlt-syslogd-windows-x86_64-app-v0.4.0.exe` |
| Server | [vlt-syslogd-srv-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-windows-v0.4.0.zip) → `vlt-syslogd-srv.exe` 以及 `install-windows.ps1` |
| Console | [vlt-syslogd-console-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-windows-v0.4.0.zip) → `vlt-syslogd-console.exe` |

解压后双击 Portable 的 exe。若出现 SmartScreen，选 **更多信息**，再选 **仍要运行**。绑定 UDP 514 不需要管理员权限。

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::UTF8.GetBytes('<132>auth: 認証に失敗しました')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

多出一行就说明到了。这段正文是不带 BOM 的 UTF-8，正是这个程序常见的输入。

### macOS

| 形态 | 文件 |
|---|---|
| Portable | [vlt-syslogd-macos-portable-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.dmg) 或 [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.zip) → `vlt-syslogd-portable.app` |
| App | [vlt-syslogd-macos-app-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.dmg) 或 [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.zip) → `vlt-syslogd.app` |
| Server | [vlt-syslogd-srv-macos-v0.4.0.tar.gz](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-macos-v0.4.0.tar.gz) → `vlt-syslogd-srv` 以及 `install-macos.sh` |
| Console | [vlt-syslogd-console-macos-app-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-macos-app-v0.4.0.zip) → `vlt-syslogd-console.app` |

这些构建是 ad-hoc 签名，没有公证。下载的应用会被 Gatekeeper 加上隔离属性。清一次，或右键应用选择 **打开**。

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

Portable 把数据放在 `.app` 旁边。隔离属性还在时，App Translocation 会让应用看不到那个目录。CJK 字体使用 macOS 自带的冬青黑体（Hiragino）。

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

macOS 的 `logger` 写入本机系统日志，不能指定主机和 UDP 端口。这里用 `nc`。先看 [端口 514](#端口-514)，再认定 514 已经打开。

### Linux（x86_64）

这个发行版里只有 GUI。没有预编译的 Server 或 Console。需要的话从源码构建（[BUILD.md](DOCS/BUILD.md)、[INSTALL.linux.md](DOCS/INSTALL.linux.md)）。

| 形态 | 文件 |
|---|---|
| Portable | [vlt-syslogd-linux-x86_64-portable-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-portable-v0.4.0) |
| App | [vlt-syslogd-linux-x86_64-app-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-app-v0.4.0) |

它们是动态链接的 x86_64 ELF。打开窗口需要显示器（X11 或 Wayland）。v0.4.0 的发行说明还把 OpenGL 列为需要的条件。

```bash
chmod +x vlt-syslogd-linux-x86_64-portable-v0.4.0
./vlt-syslogd-linux-x86_64-portable-v0.4.0
```

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

util-linux 的 `logger` 可以发 UDP。macOS 的 `logger` 没有这组参数。下面是 Linux 上的写法。

```bash
logger -n 127.0.0.1 -P 514 -d 'hello from logger'
```

要显示日文，需要 GUI 会去找的 CJK 字体：Noto Sans CJK（`/usr/share/fonts` 下的 `NotoSansCJK-Regular.ttc`）或 IPA Gothic（`/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf`）。否则这些字形会变成 □。Debian / Ubuntu 的 `fonts-noto-cjk` 会把 Noto 放到其中第一条路径。

## 事先说明

### 端口 514

日志第一行就是 bind 的结果。出现失败行时，套接字还没打开，发到 514 的测试消息不会显示。

- **Windows。** 绑定 UDP 514 不需要管理员。安装程序只在注册服务时才要求管理员。
- **Linux。** 1024 以下的端口需要 root 或 `CAP_NET_BIND_SERVICE`。对 Portable 二进制可以这样：

  ```bash
  sudo setcap 'cap_net_bind_service=+ep' ./vlt-syslogd-linux-x86_64-portable-v0.4.0
  ```

  `Server/install-linux.sh` 安装的 systemd 单元以 root 运行。单元文件里有一段注释掉的、改用 `AmbientCapabilities=CAP_NET_BIND_SERVICE` 的例子。
- **macOS。** [DOCS/portable_macos_port_behavior.md](DOCS/portable_macos_port_behavior.md) 记录了一次非 root 绑定 `0.0.0.0:514` 得到 `EACCES`。应用里的提示仍然写着：在 macOS 上做通配绑定不需要 root。以日志第一行为准。

bind 失败时，横幅会预先填上 **5514**。**待ち受け開始** 只在本次会话监听该端口，并不保存。要留下来，打开 **環境設定** 再按 **適用**，或用 `VLT_SYSLOGD_BIND=0.0.0.0:5514` 启动。测试消息发到日志行写出的端口。要在 macOS 上占用 514 本身，请用管理员权限启动 Portable，或安装以 root LaunchDaemon 运行的 Server（`com.veltrea.vlt-syslogd-srv`）。

### macOS 的 Gatekeeper

这些构建背后没有 Apple Developer 账号。右键 → **打开**，或按上面的方法去掉 `com.apple.quarantine` 之前，第一次启动会被拦住。

### 服务在跑，端口仍可能到不了

其他设备要能送到，`bind_addr` 须是 `0.0.0.0` 或局域网地址，并且防火墙放行 UDP 514。流（`5141`）和控制（`5142`）在你改掉之前留在环回地址上。把这两个口暴露到本机之外，也就暴露了可以改写 Server 配置的 `set_config`。安装指南建议远程 Console 走 SSH 隧道。

```powershell
# Windows，提升后的 PowerShell
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

```bash
# Linux
sudo ufw allow 514/udp
# firewalld：
sudo firewall-cmd --permanent --add-port=514/udp && sudo firewall-cmd --reload
```

macOS 的应用程序防火墙按应用过滤。出现提示时，允许 `vlt-syslogd-srv` 的传入连接。

## 背景

作者长期用的那些轻量 Windows syslog 服务器显示不好日文，于是用 Rust 从头写了这个。每个数据报单独解码：BOM、RFC 5424 结构化数据里的 `charset`，或者字节本身（包括用 `chardetng` 对 Shift_JIS 的估计）。Enc 列就是这次判定。维护会在体力和精力允许的时候继续。作者是 veltrea。

## 文档

| | |
|---|---|
| 安装 | [INSTALL.md](DOCS/INSTALL.md) — [Windows](DOCS/INSTALL.windows.md) · [macOS](DOCS/INSTALL.macos.md) · [Linux](DOCS/INSTALL.linux.md) |
| 端口与服务名 | [REFERENCE.md](DOCS/REFERENCE.md) |
| 构建 | [BUILD.md](DOCS/BUILD.md) — `cargo build --release --workspace`（edition 2024） |
| 编码笔记 | [syslog_verification_report.zh.md](DOCS/syslog_verification_report.zh.md) |

## 许可证

[MIT](LICENSE)。Copyright (c) 2026 veltrea。
