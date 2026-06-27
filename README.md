# vlt-syslog-server (Workspace)

A Syslog solution for Windows, macOS, and Linux, designed for high visibility and practical utility in multi-byte character (CJK) environments.

## Project Structure and Purpose

This project is organized into two components to serve different use cases.

### 1. [Portable](./Portable) (Released v0.2.0)
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

## Building & Running on macOS / Linux

The GUI builds (the root crate and `Portable`) run on Windows, macOS, and Linux. The Japanese font is auto-detected per platform — on macOS it loads Hiragino Kaku Gothic, on Linux it looks for Noto Sans CJK / IPA fonts — so multi-byte text renders correctly without any extra setup.

### Prerequisites

- Rust 1.85 or newer (the crates use `edition = "2024"`).

### Build

```bash
# Portable (GUI + server engine in a single binary — recommended)
cd Portable
cargo build --release        # binary: target/release/vlt-syslog-portable

# Root crate (the basic, UTF-8 only GUI)
cargo build --release        # from the repository root; binary: target/release/vlt-syslogd
```

### Listening port (514 needs root on macOS / Linux)

Port 514 is the standard syslog port, but on macOS and Linux ports below 1024 are privileged and require root. The bind result (success or failure) is shown in the first row of the log view.

```bash
# Option A: listen on the standard port 514 with root
sudo ./target/release/vlt-syslog-portable

# Option B: listen on a non-privileged port without root
VLT_SYSLOGD_BIND=0.0.0.0:5514 ./target/release/vlt-syslog-portable
```

`VLT_SYSLOGD_BIND` overrides the listen address for both GUI builds.

### Server engine (console daemon on macOS / Linux)

The `Server` crate runs as a Windows Service on Windows. On macOS and Linux it builds and runs the same engine as a foreground console daemon (intended to be supervised by `launchd` or `systemd`). The listen address is read from `config.toml` (created on first run in the current directory).

```bash
cd Server
cargo build --release
./target/release/vlt-syslog-srv run
```
