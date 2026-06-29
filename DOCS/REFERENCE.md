# vlt-syslogd — Components Reference

Japanese version: [REFERENCE.ja.md](REFERENCE.ja.md)

Not needed to install. This is reference for understanding how it works or changing the configuration yourself. For install steps, see the per-OS pages: [macOS](INSTALL.macos.md) · [Linux](INSTALL.linux.md) · [Windows](INSTALL.windows.md).

---

## The three components

| Name | Executable | What it is | Runs as |
|---|---|---|---|
| **Server** | `vlt-syslogd-srv` (`.exe` on Windows) | Headless syslog engine. Receives UDP syslog and serves it to the Console. | System service (launchd / systemd / Windows Service) |
| **Console** | `vlt-syslogd-console` (`.exe` on Windows) | GUI viewer that connects to the Server over TCP and shows/controls it. | Desktop app |
| **Portable** | `vlt-syslogd-portable` (`.exe` on Windows) | Standalone GUI that listens for UDP syslog by itself (no service needed). | Desktop app |

---

## Ports used by the Server

- **514/udp** — syslog reception (the standard port; needs admin/root to bind on macOS / Linux — see below)
- **5141/tcp** — stream delivery to the Console (JSON Lines, one-way)
- **5142/tcp** — control channel (`get_config` / `set_config`)

For default exposure and the cautions when changing it, see the "Network topology" section of each per-OS page.

### Why binding port 514 needs admin privileges (macOS / Linux)

Ports 0–1023 are **privileged ports (well-known ports)**: on Unix-like systems only a process with root (admin) privileges may bind (start listening on) them. It's a long-standing safeguard that stops an unprivileged program from hijacking the ports of standard services like syslog, SSH, or HTTP and impersonating them. Syslog's standard port, 514, falls in this range.

**macOS is BSD-derived, so the same restriction applies** (it is *not* exempt just because it's a Mac). How each OS handles it:

- **macOS** — the Server is registered as a launchd **LaunchDaemon (runs as root)**, so it can bind 514. That's why install asks for admin authentication. If you launch Portable as a normal user, binding the default 514 may fail; in that case switch to a port ≥ 1024 (e.g. `bind_addr = "0.0.0.0:5514"`, and point the senders at that port) or launch with admin privileges.
- **Linux** — the systemd unit runs as root by default, so it can bind. To avoid root, grant `CAP_NET_BIND_SERVICE` ([INSTALL.linux.md](INSTALL.linux.md) §8).
- **Windows** — has no "privileged port" concept, so **binding 514 itself needs no admin rights** (the installer uses admin only to register the service, which is a separate matter).

---

## Service identifiers

The shipped installer and Console already agree on these names, so **you normally never need to think about them**. Only if you modify the installer to rename the service do you also set the Console to the same name.

| OS | Identifier |
|---|---|
| macOS (launchd label) | `com.veltrea.vlt-syslogd-srv` |
| Linux (systemd unit) | `vlt-syslogd-srv.service` |
| Windows (service name) | `vlt-syslogd-srv` |
