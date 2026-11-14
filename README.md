# Veltrea Syslog Server

English · [日本語](README.ja.md) · [한국어](README.ko.md) · [简体中文](README.zh.md) · [繁體中文](README.tw.md) · [Tiếng Việt](README.vi.md)

**Each message is decoded on its own as Shift_JIS or UTF-8, so mixed encodings on one port stay readable — no mojibake.**

**[Website](https://veltrea.github.io/vlt-syslogd/index.en.html)** · [日本語のサイト](https://veltrea.github.io/vlt-syslogd/) · **[Release v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0)**

![Portable log view. The Enc column shows UTF-8, Shift_JIS, and Shift_JIS (MSG-SD/BOM-Missing) on one screen](DOCS/assets/portable-main.png)

*The same capture as the website: the v0.4.0 Windows build. The Enc column is the decision for that row — `UTF-8`, `Shift_JIS`, or `Shift_JIS (MSG-SD/BOM-Missing)` when RFC 5424 structured data declared the charset. Right-click a message and choose **Copy as Hex** for the original bytes.*

Latest release is **v0.4.0**, for Windows, macOS, and Linux (x86_64). If you are not sure which file to take, start with **Portable**.

The window labels are in Japanese. On Windows and Linux the menus are bilingual (`ファイル(File)`, `編集(Edit)`, `ヘルプ(Help)`).

## Three shapes

The part that reads syslog is the same idea in all three. Only how you run it changes.

| | Portable | Server | Console |
|---|---|---|---|
| Binary | `vlt-syslogd-portable` | `vlt-syslogd-srv` | `vlt-syslogd-console` |
| What it is | A GUI that opens the UDP socket itself. No installer, no service. | Headless engine. Windows Service, launchd LaunchDaemon, or systemd. Writes logs to disk from boot. | A GUI that attaches to a running Server over TCP. Closing it leaves collection running. Service start, stop, and restart are in this window. |
| Ports | UDP, default `0.0.0.0:514` | UDP `514` (syslog). TCP `5141` (stream to Console) and TCP `5142` (control), both `127.0.0.1` by default | Connects to the Server's stream and control addresses |
| In the v0.4.0 release | Windows, macOS, Linux x86_64 | Windows, macOS. Linux: build from source (`Server/install-linux.sh` is in the repo; this release has no Linux Server binary) | Windows, macOS. Linux: build from source |

**App** and **Portable** are the same GUI. They differ only in where they keep data:

| Variant | Data directory |
|---|---|
| **App** | Windows `%APPDATA%\vlt-syslogd`, macOS `~/Library/Application Support/vlt-syslogd`, Linux `$XDG_DATA_HOME/vlt-syslogd` or `~/.local/share/vlt-syslogd` |
| **Portable** | Next to the executable. On macOS, next to the `.app` (a USB stick, nothing written into the system) |

`VLT_SYSLOGD_DATA_DIR` overrides that directory for either variant. Server uses a system directory instead: `C:\ProgramData\vlt-syslogd`, `/usr/local/var/vlt-syslogd`, or `/var/lib/vlt-syslogd`.

Preferences (**ファイル → 環境設定…**) has two fields: the listen port (**待ち受けポート**) and the log directory (**ログ保存先**). **適用** writes them.

## Download v0.4.0 and send one message

Files below are the [v0.4.0 release assets](https://github.com/veltrea/vlt-syslogd/releases/tag/v0.4.0). Send the test message to the port named in the first log row. `Listening on 0.0.0.0:514 (UDP)` means that port is open.

Making it resident, or receiving from other machines, is in the [install guides](DOCS/INSTALL.md).

### Windows

| Shape | File |
|---|---|
| Portable | [vlt-syslogd-portable-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-portable-windows-v0.4.0.zip) → `vlt-syslogd-portable.exe` |
| App | [vlt-syslogd-app-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-app-windows-v0.4.0.zip) → `vlt-syslogd-windows-x86_64-app-v0.4.0.exe` |
| Server | [vlt-syslogd-srv-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-windows-v0.4.0.zip) → `vlt-syslogd-srv.exe` plus `install-windows.ps1` |
| Console | [vlt-syslogd-console-windows-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-windows-v0.4.0.zip) → `vlt-syslogd-console.exe` |

Unzip and double-click the Portable exe. If SmartScreen appears, choose **More info**, then **Run anyway**. Binding UDP 514 does not require an administrator.

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::UTF8.GetBytes('<132>auth: 認証に失敗しました')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

A new row should appear. The message is UTF-8 with no BOM; that is a normal case for this program.

### macOS

| Shape | File |
|---|---|
| Portable | [vlt-syslogd-macos-portable-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.dmg) or [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-portable-v0.4.0.zip) → `vlt-syslogd-portable.app` |
| App | [vlt-syslogd-macos-app-v0.4.0.dmg](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.dmg) or [.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-macos-app-v0.4.0.zip) → `vlt-syslogd.app` |
| Server | [vlt-syslogd-srv-macos-v0.4.0.tar.gz](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-srv-macos-v0.4.0.tar.gz) → `vlt-syslogd-srv` plus `install-macos.sh` |
| Console | [vlt-syslogd-console-macos-app-v0.4.0.zip](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-console-macos-app-v0.4.0.zip) → `vlt-syslogd-console.app` |

The builds are ad-hoc signed, not notarized. Gatekeeper quarantines a downloaded app. Clear it once, or right-click the app and choose **Open**:

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

Portable keeps its data beside the `.app`. Leaving the quarantine flag on lets App Translocation hide that folder from the app. The CJK font is Hiragino, which ships with macOS.

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

macOS `logger` writes to the local system log. It does not take a host and UDP port. Use `nc` here. See [Port 514](#port-514) before assuming 514 opened.

### Linux (x86_64)

The release contains the GUI only. There is no prebuilt Server or Console binary; build those from source if you need them ([BUILD.md](DOCS/BUILD.md), [INSTALL.linux.md](DOCS/INSTALL.linux.md)).

| Shape | File |
|---|---|
| Portable | [vlt-syslogd-linux-x86_64-portable-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-portable-v0.4.0) |
| App | [vlt-syslogd-linux-x86_64-app-v0.4.0](https://github.com/veltrea/vlt-syslogd/releases/download/v0.4.0/vlt-syslogd-linux-x86_64-app-v0.4.0) |

The files are dynamically linked x86_64 ELFs. Opening the window needs a display (X11 or Wayland). The v0.4.0 release notes also call for OpenGL.

```bash
chmod +x vlt-syslogd-linux-x86_64-portable-v0.4.0
./vlt-syslogd-linux-x86_64-portable-v0.4.0
```

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

util-linux `logger` can send UDP. macOS `logger` cannot; this is the Linux form:

```bash
logger -n 127.0.0.1 -P 514 -d 'hello from logger'
```

Japanese text needs a CJK font the GUI knows how to find: Noto Sans CJK (`NotoSansCJK-Regular.ttc` under `/usr/share/fonts`) or IPA Gothic (`/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf`). Otherwise those glyphs render as □. `fonts-noto-cjk` on Debian/Ubuntu places Noto on the first of those paths.

## Caveats

### Port 514

The first log row is the bind result. A failure row means the socket is not open, so a test message to 514 will not show up.

- **Windows.** Binding UDP 514 does not require an administrator. The installer asks for an administrator only when it registers the service.
- **Linux.** Ports below 1024 need root or `CAP_NET_BIND_SERVICE`. For the Portable binary:

  ```bash
  sudo setcap 'cap_net_bind_service=+ep' ./vlt-syslogd-linux-x86_64-portable-v0.4.0
  ```

  `Server/install-linux.sh` installs a systemd unit that runs as root. The unit file has a commented alternative that uses `AmbientCapabilities=CAP_NET_BIND_SERVICE`.
- **macOS.** A non-root bind of `0.0.0.0:514` returned `EACCES` in the check written up in [DOCS/portable_macos_port_behavior.md](DOCS/portable_macos_port_behavior.md). The hint inside the app still says a wildcard bind on macOS does not need root. Believe the first log row.

When bind fails, the banner prefills **5514**. **待ち受け開始** listens on that port for this session and does not save it. To keep it, use **環境設定** and **適用**, or launch with `VLT_SYSLOGD_BIND=0.0.0.0:5514`. Send the test message to the port the log row names. To hold 514 on macOS, start Portable with administrator rights, or install Server (a root LaunchDaemon, `com.veltrea.vlt-syslogd-srv`).

### macOS Gatekeeper

No Apple Developer account is behind these builds. The first launch is stopped until you right-click → **Open**, or clear `com.apple.quarantine` as above.

### A running service can still be unreachable

Remote devices reach the Server only when `bind_addr` is `0.0.0.0` or a LAN address **and** the firewall allows UDP 514. Stream (`5141`) and control (`5142`) stay on loopback unless you change them. Opening those two beyond the machine exposes `set_config`, which can rewrite the Server config. The install guides recommend an SSH tunnel for a remote Console.

```powershell
# Windows, elevated PowerShell
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

```bash
# Linux
sudo ufw allow 514/udp
# firewalld:
sudo firewall-cmd --permanent --add-port=514/udp && sudo firewall-cmd --reload
```

On macOS the application firewall is per app. Allow incoming connections for `vlt-syslogd-srv` when prompted.

## Background

Written in Rust, from scratch, because the lightweight Windows syslog servers the author had been using could not display Japanese reliably. Each datagram is decoded on its own: a BOM, a `charset` parameter in RFC 5424 structured data, or the bytes themselves (including a Shift_JIS guess via `chardetng`). The Enc column is that decision. Maintenance continues as long as health and energy allow. The author is veltrea.

## Documentation

| | |
|---|---|
| Install | [INSTALL.md](DOCS/INSTALL.md) — [Windows](DOCS/INSTALL.windows.md) · [macOS](DOCS/INSTALL.macos.md) · [Linux](DOCS/INSTALL.linux.md) |
| Ports and service names | [REFERENCE.md](DOCS/REFERENCE.md) |
| Build | [BUILD.md](DOCS/BUILD.md) — `cargo build --release --workspace` (edition 2024) |
| Encoding notes | [syslog_verification_report.md](DOCS/syslog_verification_report.md) |

## License

[MIT](LICENSE). Copyright (c) 2026 veltrea.
