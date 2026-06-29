# vlt-syslogd-server (Workspace)

A Syslog solution for Windows and macOS, designed for high visibility and practical utility in multi-byte character (CJK) environments.

## Project Structure and Purpose

This project is organized into two components to serve different use cases.

### 1. [Portable](./Portable) (Released v0.3.0)
- **Status**: Released. Maintenance will continue as long as health and energy permit.
- **Concept**: "An engineer's Swiss Army knife for your USB drive."
- **Features**: A single binary integrating both a GUI and a server engine. Requires no installation or internet access, providing ultimate mobility for on-site debugging and temporary log collection.
- **Philosophy**: Created by a busy engineer who thought, "This would be handy." I hope fellow engineers find it useful and easy to deploy.

### 2. [Server](./Server) (Under Development)
- **Status**: Currently under development as the next milestone.
- **Concept**: "Professional-grade stable operation for Windows Servers."
- **Features**: Separates the background engine (running as a Windows Service) from the monitoring frontend (connected only when needed).
- **Goal**: Provides a robust log collection infrastructure for production environments requiring 24/7 operation.

---

## Background

Built from scratch using Rust to overcome the "Japanese display limitations" of existing lightweight Windows Syslog servers. It features "Tolerant Parsing" to accurately handle real-world data variations, including various encodings like Shift_JIS and UTF-8 with or without BOM.

For details, please refer to the README in each directory.

---

## Downloads (macOS)

Prebuilt, ad-hoc–signed macOS bundles are produced by `./Portable/build-macos.sh` into `dist/`. The GUI ships in two variants that differ only in **where they keep their data** — pick the one that matches how you run it:

| Variant | Data location | Best for | Artifact |
|---|---|---|---|
| **App** | `~/Library/Application Support/vlt-syslogd/` | Dropping into `/Applications`, everyday desktop use | `vlt-syslogd-macos-app-v<ver>.zip` |
| **Portable** | next to the `.app` itself | Carrying on a USB stick, zero system footprint | `vlt-syslogd-macos-portable-v<ver>.zip` |

Both run from anywhere without admin rights. On macOS (Mojave and later), binding the standard syslog port `514` on `0.0.0.0` does **not** require root, so you can just double-click and go. (Binding a *specific* interface such as `127.0.0.1:514` still needs root — see the listening-port notes below.)

Downloaded apps are quarantined by Gatekeeper on first launch. The **Portable** variant especially keeps its data next to the app, which App Translocation breaks while the app is quarantined, so clear the quarantine flag once:

```bash
xattr -dr com.apple.quarantine vlt-syslogd-portable.app
```

Alternatively, right-click the app and choose **Open** the first time. To receive on `514` from a device that only sends to a fixed port while something else holds the port — or to run a boot-time daemon — use the Server variant.

## Building & Running on macOS

The GUI build (`Portable`) runs on Windows and macOS. The Japanese font is auto-detected per platform — on macOS it loads Hiragino Kaku Gothic — so multi-byte text renders correctly without any extra setup.

### Prerequisites

- Rust 1.85 or newer (the crates use `edition = "2024"`).

### Build

```bash
# GUI (single binary). Default = App build; add the feature for the Portable build.
cd Portable
cargo build --release                       # App build (data in ~/Library/Application Support)
cargo build --release --features portable   # Portable build (data next to the binary)
```

To produce ready-to-ship, ad-hoc–signed macOS `.app` bundles + zips for both variants in `dist/`:

```bash
./Portable/build-macos.sh
```

### Listening port

Port 514 is the standard syslog port. On macOS (Mojave and later) you can bind it on `0.0.0.0` **without** root; only binding a *specific* interface (e.g. `127.0.0.1`) still requires root. On Linux, ports below 1024 require root or `CAP_NET_BIND_SERVICE`. The bind result (success or failure) is shown in the first row of the log view, and if a bind fails you can pick another port from the Preferences window (Settings → Preferences).

```bash
# Option A: listen on the standard port 514 with root
sudo ./target/release/vlt-syslogd-portable

# Option B: listen on a non-privileged port without root
VLT_SYSLOGD_BIND=0.0.0.0:5514 ./target/release/vlt-syslogd-portable
```

`VLT_SYSLOGD_BIND` overrides the listen address for the GUI build.

### Server engine (console daemon on macOS)

The `Server` crate runs as a Windows Service on Windows. On macOS it builds and runs the same engine as a foreground console daemon (intended to be supervised by `launchd`). The listen address is read from `config.toml`, which is created on first run in the platform data directory — macOS: `/usr/local/var/vlt-syslogd`, Linux: `/var/lib/vlt-syslogd`, Windows: `C:\ProgramData\vlt-syslogd`. Set `VLT_SYSLOGD_DATA_DIR` to override this location.

```bash
cd Server
cargo build --release
./target/release/vlt-syslogd-srv run
```
