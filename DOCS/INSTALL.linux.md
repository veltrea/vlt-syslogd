# vlt-syslogd — Installation Guide (Linux)

日本語: [INSTALL.linux.ja.md](INSTALL.linux.ja.md) ／ Other OS: [macOS](INSTALL.macos.md) · [Windows](INSTALL.windows.md) ／ [Back to start](INSTALL.md)

This page is **Linux only** (systemd-based distributions assumed). On macOS / Windows, use the links above.

vlt-syslogd receives and displays syslog. There are three ways to use it, so **start by picking what you want to do**.

---

## Start here — which one do you install?

| What you want | What to install | Sections to follow |
|---|---|---|
| **Just try it / keep everything on one machine** | Portable only (no install) | §1 → §2 |
| **Receive syslog continuously (server use)** | Server (background) + Console (the viewer) | §1 → §3 → §4 → §5 → §6 |
| **View an already-running Server from another screen** | Console only | §1 → §5 |

If you're unsure, start with **Portable** at the top. (Binding port 514 needs privileges, as noted below.)

> Want to know the difference between Server / Console / Portable first? See the [Components Reference](REFERENCE.md). You don't need it just to install.

---

## 1. Get the executable

Download the **Linux** files from GitHub Releases. Grab only the ones your path uses:

- `vlt-syslogd-srv` (Server)
- `vlt-syslogd-console` (Console)
- `vlt-syslogd-portable` (Portable)

After downloading, make them executable: `chmod +x vlt-syslogd-*`.

> **Want to build from source?** See [BUILD.md](BUILD.md). After building, the steps are identical to the prebuilt case.
>
> The GUI (Console / Portable) needs CJK fonts to render Japanese, otherwise it shows □ (tofu) — see §8.

---

## 2. Run Portable to try it (no install)

Portable is a GUI that listens on UDP 514 by itself. No service registration — **just run the downloaded `vlt-syslogd-portable`.**

```bash
./vlt-syslogd-portable
```

Port 514 is privileged; if the bind fails, run as root or grant `CAP_NET_BIND_SERVICE` (§8).

When the window opens, send a test message from another terminal; if a row appears, it works.

```bash
printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
```

That completes the "just try it" path. When you want it running permanently, continue to §3.

---

## 3. Install the Server (systemd service)

The Server is a headless background program registered as a systemd service. The install script in `Server/` handles everything: place the binary, create the data directory, register the service, and start it. You're asked for root **only at install time**; afterwards it runs on its own and starts at boot.

```bash
cd Server
sudo ./install-linux.sh            # or: sudo ./install-linux.sh /path/to/vlt-syslogd-srv
```

The script auto-locates the binary in this order: the path you pass as an argument → a copy next to the script → `../target/release/` → `./target/release/`.

Locations:

- Binary: `/usr/local/bin/vlt-syslogd-srv`
- Data/logs: `/var/lib/vlt-syslogd/`
- Unit: `/etc/systemd/system/vlt-syslogd-srv.service`
- Status: `systemctl status vlt-syslogd-srv.service`
- Live logs: `journalctl -u vlt-syslogd-srv.service -f`

The unit runs as **root** so it can bind port 514. To run unprivileged instead, the unit file contains commented hints for a dedicated user plus `AmbientCapabilities=CAP_NET_BIND_SERVICE`.

---

## 4. The configuration file (only if you need it)

On its **first run** the Server auto-generates `config.toml` in `/var/lib/vlt-syslogd/`. It works as-is, so **read this only when you want to change ports or networking**.

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

A running service is not the same as a reachable port. If remote devices' logs never appear even though the service is "running", the firewall is the usual cause. Open **UDP 514** (and only 5141/5142 if you deliberately exposed them for a remote Console):

```bash
# ufw
sudo ufw allow 514/udp

# firewalld
sudo firewall-cmd --permanent --add-port=514/udp && sudo firewall-cmd --reload
```

---

## 5. Run the Console

The Console is a GUI that connects to the Server to show and control it. It does not need to be a service. **Just run the downloaded `vlt-syslogd-console`.**

```bash
./vlt-syslogd-console
```

On first launch, open **⚙ Settings** and confirm the connection targets match your Server:

- Stream address → the Server's `stream_addr` (default `127.0.0.1:5141`)
- Control address → the Server's `control_addr` (default `127.0.0.1:5142`)

The Console has buttons to **Start / Stop / Apply-restart** the Server and a **service status** line. These work when the Server is installed via the shipped installer. Expected behavior:

- Start/Stop/Restart escalate via `pkexec` or `sudo`.
- **Service not installed**: the operation fails immediately with a clear message. **Saving the config still succeeds** — only the restart is skipped.

---

## 6. Verify

1. Confirm the service is **active**:
   `systemctl is-active vlt-syslogd-srv.service`
2. Confirm the Server is **listening**:
   `lsof -nP -iUDP:514` and `lsof -nP -iTCP -sTCP:LISTEN | grep -E "5141|5142"`
3. Send a **local** test message and confirm it appears in the Console:
   ```bash
   printf '<34>Oct 11 22:14:15 myhost myapp: hello' | nc -u -w1 127.0.0.1 514
   ```
   The **status indicator should turn green (🟢 / "● Receiving")** and the line should appear in the table.
4. **Remote reachability** (only if remote devices will send): from *another* host, confirm UDP 514 is open and the message arrives:
   ```bash
   # from another machine — replace SERVER_IP
   printf '<34>Oct 11 22:14:15 dev1 app: remote-test' | nc -u -w1 SERVER_IP 514
   ```
   If nothing arrives, re-check `bind_addr` (must be `0.0.0.0`/LAN IP, not `127.0.0.1`) and the firewall (§4). "Service running" does **not** guarantee "port reachable".

---

## 7. Uninstall

```bash
cd Server && sudo ./uninstall-linux.sh
```

Stops and removes the service and deletes the installed binary. **Config and logs are kept**; remove `/var/lib/vlt-syslogd/` manually for a clean slate.

---

## 8. Troubleshooting

| Symptom | Cause / Fix |
|---|---|
| Port 514 bind fails | Needs root or `CAP_NET_BIND_SERVICE`. The systemd unit runs as root by default. For standalone runs, use `sudo` or grant the cap: `sudo setcap 'cap_net_bind_service=+ep' ./vlt-syslogd-portable`. |
| Japanese text shows as □ (tofu) in the GUI | No CJK fonts. Install Noto Sans CJK / IPA (e.g. `sudo apt install fonts-noto-cjk` / `sudo dnf install google-noto-sans-cjk-fonts`). |
| Console always shows "○ Disconnected" | Server not running, or stream address mismatch. Check the Server is up and the Console's stream address equals the Server's `stream_addr`. |
| "Fetch current values" fails in Settings | Control address mismatch, or an old Server without the control port. Verify `control_addr` and reinstall the Server. |
| Console service status shows "Not installed" after install | Happens when you renamed the systemd unit. The Console and installer must point at the same unit (they match by default). Identifiers are in the [Components Reference](REFERENCE.md). |
| Service runs, but remote devices' logs never arrive | "Running" ≠ "reachable". Check (1) `bind_addr` is `0.0.0.0`/LAN IP, not `127.0.0.1`; (2) the firewall allows UDP 514 (§4); (3) the sender targets the Server's actual IP. Verify from another host with `nc -u SERVER_IP 514`. |
| Remote Console can't connect | `stream_addr`/`control_addr` are loopback-only by default. Set a routable bind and open the firewall. The control port can rewrite the Server config, so restrict to trusted hosts or tunnel over SSH. |

---

For the component layout, ports, and service identifiers, see the [Components Reference (REFERENCE.md)](REFERENCE.md).
