# vlt-syslog-server (Workspace)

A Syslog solution for Windows, designed for high visibility and practical utility in multi-byte character (CJK) environments.

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
