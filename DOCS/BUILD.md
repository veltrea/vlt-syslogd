# vlt-syslogd — Build Guide

Japanese version: [BUILD.ja.md](BUILD.ja.md)

How to build the executables from source. **If you only want to install and use a prebuilt build, you don't need this guide** — grab a prebuilt binary and go to [INSTALL.md](INSTALL.md).

---

## 1. Prerequisites

You need the [Rust toolchain](https://rustup.rs) (`cargo`). A minimal profile is enough:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal
```

---

## 2. Build everything

Run this **from the root** of the folder where you extracted the repository:

```bash
cargo build --release --workspace
```

`--workspace` builds all three components at once. The binaries land in `target/release/`:

- `target/release/vlt-syslogd-srv` (Server)
- `target/release/vlt-syslogd-console` (Console)
- `target/release/vlt-syslogd-portable` (Portable)

> Names like `vlt-syslogd-srv` are **crate names** — the formal name that tells cargo which component to act on, written after `-p` (package). To build just one: `cargo build --release -p vlt-syslogd-portable`.

---

## 3. Run directly without installing (development)

To launch and check a component on the spot, without registering a service, use `cargo run`:

```bash
# Portable (listens on UDP 514 by itself)
cargo run --release -p vlt-syslogd-portable

# Console (GUI that connects to the Server)
cargo run --release -p vlt-syslogd-console

# Server (headless engine, in the foreground, not registered as a service)
cargo run --release -p vlt-syslogd-srv
```

`cargo run` builds first and then launches, so you don't need to run §2 separately.

---

## 4. To distribute or run your build as a service

- **Install as a background service** → [INSTALL.md](INSTALL.md) §3. The install scripts auto-locate the executable in `target/release/`.
- **Distribute on macOS** → bundle into a `.app` and ad-hoc sign it (see the project's signing notes).

---

## 5. Platform notes

- **Linux**: the GUI (Console / Portable) needs CJK fonts to render; without them Japanese shows as □ (tofu). Install Noto Sans CJK / IPA, etc.
- **Windows**: depending on the target you may need the MSVC toolchain (Visual Studio Build Tools).
- **Cross-building**: building each OS's binary on that OS is the reliable path.
