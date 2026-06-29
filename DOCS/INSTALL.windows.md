# vlt-syslogd — Installation Guide (Windows)

日本語: [INSTALL.windows.ja.md](INSTALL.windows.ja.md) ／ Other OS: [macOS](INSTALL.macos.md) · [Linux](INSTALL.linux.md) ／ [Back to start](INSTALL.md)

This page is **Windows only**. On macOS / Linux, use the links above.

vlt-syslogd receives and displays syslog. There are three ways to use it, so **start by picking what you want to do**.

---

## Start here — which one do you install?

| What you want | What to install | Sections to follow |
|---|---|---|
| **Just try it / keep everything on one machine** | Portable only (no install) | §1 → §2 |
| **Receive syslog continuously (server use)** | Server (background) + Console (the viewer) | §1 → §3 → §4 → §5 → §6 |
| **View an already-running Server from another screen** | Console only | §1 → §5 |

If you're unsure, start with **Portable** at the top. It needs no service registration and no admin rights — launch it and it just works.

> Want to know the difference between Server / Console / Portable first? See the [Components Reference](REFERENCE.md). You don't need it just to install.

---

## 1. Get the executable

Download the **Windows** files (`.exe`) from GitHub Releases. Grab only the ones your path uses:

- `vlt-syslogd-srv.exe` (Server)
- `vlt-syslogd-console.exe` (Console)
- `vlt-syslogd-portable.exe` (Portable)

If SmartScreen warns you, choose "More info" → "Run anyway".

> **Want to build from source?** See [BUILD.md](BUILD.md). After building, the steps are identical to the prebuilt case.

---

## 2. Run Portable to try it (no install)

Portable is a GUI that listens on UDP 514 by itself. No service registration — **just double-click the downloaded `vlt-syslogd-portable.exe`.**

When the window opens, send a test message from PowerShell; if a row appears, it works.

```powershell
$u = New-Object System.Net.Sockets.UdpClient
$b = [Text.Encoding]::ASCII.GetBytes('<34>Oct 11 22:14:15 myhost myapp: hello')
$u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
```

That completes the "just try it" path. When you want it running permanently, continue to §3.

---

## 3. Install the Server (Windows Service)

The Server is a headless background program registered as a Windows Service. The install script in `Server/` handles everything: place the binary, create the data directory, register the service, and start it. Admin rights are needed **only at install time**; afterwards it runs on its own and starts at boot.

From an **elevated** PowerShell (Run as Administrator):

```powershell
cd Server
powershell -ExecutionPolicy Bypass -File .\install-windows.ps1
# or: ... -File .\install-windows.ps1 -BinPath C:\path\to\vlt-syslogd-srv.exe
```

Locations:

- Binary: `C:\Program Files\vlt-syslogd\vlt-syslogd-srv.exe`
- Data/logs: `C:\ProgramData\vlt-syslogd\`
- Service: `vlt-syslogd-srv` (start type Automatic)
- Status: `sc.exe query vlt-syslogd-srv`

> The PowerShell scripts are written in **English (ASCII only)** to avoid CP932 mojibake. Japanese explanations live in this manual.

---

## 4. The configuration file (only if you need it)

On its **first run** the Server auto-generates `config.toml` in `C:\ProgramData\vlt-syslogd\`. It works as-is, so **read this only when you want to change ports or networking**.

```toml
[server]
bind_addr    = "0.0.0.0:514"        # syslog reception (UDP)
stream_addr  = "127.0.0.1:5141"     # delivery to Console (TCP)
control_addr = "127.0.0.1:5142"     # control channel (TCP)

[logging]
level        = "info"
max_size_mb  = 10
keep_files   = 7
```

After changing it, restart the service (or use the Console's **Apply (restart)** button). You can override the data directory for testing with `VLT_SYSLOGD_DATA_DIR`.

### Network topology — who can reach what

The three ports are exposed differently **on purpose**:

| Port | Default bind | Reachable from | To change |
|---|---|---|---|
| 514/udp (reception) | `0.0.0.0` | any host (so remote devices can send syslog) | keep `0.0.0.0`, or pin to a LAN IP |
| 5141/tcp (stream) | `127.0.0.1` | **same host only** | set a routable bind for a remote Console (see caution) |
| 5142/tcp (control) | `127.0.0.1` | **same host only** | set a routable bind for a remote Console (see caution) |

- **Receiving from remote devices** works only if `bind_addr` stays `0.0.0.0` (or a LAN IP) **and** the firewall allows UDP 514. With `bind_addr = "127.0.0.1:514"`, only the local host can send.
- **The Console must run on the same machine as the Server by default**, because `stream_addr` / `control_addr` are loopback-only. To use a Console on another machine, change those to a routable address — but this **exposes the control channel** (`set_config` can rewrite the Server config). Tunneling over SSH is recommended.

### Firewall — open the syslog port

A running service is not the same as a reachable port. If remote devices' logs never appear even though the service is "running", the Windows firewall is the usual cause. From an **elevated** PowerShell, open UDP 514 (and only 5141/5142 if you deliberately exposed them for a remote Console):

```powershell
New-NetFirewallRule -DisplayName "vlt-syslogd 514/udp" -Direction Inbound -Protocol UDP -LocalPort 514 -Action Allow
```

---

## 5. Run the Console

The Console is a GUI that connects to the Server to show and control it. It does not need to be a service. **Just double-click the downloaded `vlt-syslogd-console.exe`.**

On first launch, open **⚙ Settings** and confirm the connection targets match your Server:

- Stream address → the Server's `stream_addr` (default `127.0.0.1:5141`)
- Control address → the Server's `control_addr` (default `127.0.0.1:5142`)

The Console has buttons to **Start / Stop / Apply-restart** the Server and a **service status** line. These work when the Server is installed via the shipped installer. Expected behavior:

- Start/Stop/Restart escalate via **UAC (User Account Control)**. Approve to proceed.
- **Service not installed**: the operation fails immediately with a clear message. **Saving the config still succeeds** — only the restart is skipped.

---

## 6. Verify

1. Confirm the service is **running**:
   `sc.exe query vlt-syslogd-srv` (look for `RUNNING`)
2. Confirm the Server is **listening**:
   `Get-NetUDPEndpoint -LocalPort 514` and `Get-NetTCPConnection -State Listen -LocalPort 5141,5142`
3. Send a **local** test message and confirm it appears in the Console:
   ```powershell
   $u = New-Object System.Net.Sockets.UdpClient
   $b = [Text.Encoding]::ASCII.GetBytes('<34>Oct 11 22:14:15 myhost myapp: hello')
   $u.Send($b, $b.Length, '127.0.0.1', 514) | Out-Null; $u.Close()
   ```
   The **status indicator should turn green (🟢 / "● Receiving")** and the line should appear in the table.
4. **Remote reachability** (only if remote devices will send): from *another* host, confirm UDP 514 is open and the message arrives. If nothing arrives, re-check `bind_addr` (must be `0.0.0.0`/LAN IP, not `127.0.0.1`) and the firewall (§4). "Service running" does **not** guarantee "port reachable".

---

## 7. Uninstall

From an **elevated** PowerShell:

```powershell
cd Server; powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1
```

Stops and removes the service and deletes the installed binary. **Config and logs are kept**; remove `C:\ProgramData\vlt-syslogd\` manually for a clean slate.

---

## 8. Troubleshooting

| Symptom | Cause / Fix |
|---|---|
| Japanese console output is garbled | The console code page is CP932. The installer scripts and service are ASCII-only to avoid this; it does not affect operation. |
| Console always shows "○ Disconnected" | Server not running, or stream address mismatch. Check the Server is up and the Console's stream address equals the Server's `stream_addr`. |
| "Fetch current values" fails in Settings | Control address mismatch, or an old Server without the control port. Verify `control_addr` and reinstall the Server. |
| Console service status shows "Not installed" after install | Happens when you renamed the service. The Console and installer must point at the same service name (they match by default). Identifiers are in the [Components Reference](REFERENCE.md). |
| Service runs, but remote devices' logs never arrive | "Running" ≠ "reachable". Check (1) `bind_addr` is `0.0.0.0`/LAN IP, not `127.0.0.1`; (2) the Windows firewall allows UDP 514 (§4); (3) the sender targets the Server's actual IP. |
| Remote Console can't connect | `stream_addr`/`control_addr` are loopback-only by default. Set a routable bind and open the firewall. The control port can rewrite the Server config, so restrict to trusted hosts or tunnel over SSH. |
| Install script won't run | Open PowerShell as Administrator and run it with `-ExecutionPolicy Bypass`. |

---

For the component layout, ports, and service identifiers, see the [Components Reference (REFERENCE.md)](REFERENCE.md).
