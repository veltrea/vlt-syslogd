# vlt-syslogd: Windows install script (Windows Service)
#
# Registers the server engine (vlt-syslogd-srv.exe) as a Windows service that
# starts at boot and listens on the standard syslog port 514 (UDP).
# Administrator rights are needed only for this install step.
#
# NOTE: This file is intentionally ASCII-only. Windows PowerShell 5.1 reads .ps1
# files as the system ANSI code page (CP932 on Japanese Windows) unless they have
# a UTF-8 BOM, so non-ASCII comments/strings would break. The Japanese
# explanation lives in the install manual (DOCS/INSTALL.ja.md).
#
# Usage (run from an elevated PowerShell):
#   powershell -ExecutionPolicy Bypass -File .\install-windows.ps1 [-BinPath C:\path\to\vlt-syslogd-srv.exe]
#
[CmdletBinding()]
param(
    [string]$BinPath = ""
)

$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ServiceName = "vlt-syslogd-srv"     # must match Console (service.rs WIN_SERVICE_NAME) and Server (SERVICE_NAME)
$BinName     = "vlt-syslogd-srv.exe"
$InstallDir  = Join-Path $env:ProgramFiles "vlt-syslogd"
$InstallBin  = Join-Path $InstallDir $BinName
$DataDir     = Join-Path $env:ProgramData "vlt-syslogd"   # matches platform.rs Windows data_dir()

# --- Require administrator ---
$isAdmin = ([Security.Principal.WindowsPrincipal] `
    [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "Administrator rights are required. Run PowerShell as Administrator and re-run this script."
    exit 1
}

# --- Locate the binary to install ---
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$candidates = @()
if ($BinPath -ne "") { $candidates += $BinPath }
$candidates += (Join-Path $ScriptDir $BinName)
$candidates += (Join-Path $ScriptDir "..\target\release\$BinName")
$candidates += (Join-Path $ScriptDir "target\release\$BinName")

$SrcBin = $null
foreach ($c in $candidates) {
    if (Test-Path -LiteralPath $c -PathType Leaf) { $SrcBin = (Resolve-Path -LiteralPath $c).Path; break }
}
if ($null -eq $SrcBin) {
    Write-Error ("{0} not found. Build it first with 'cargo build --release -p vlt-syslogd-srv', or pass -BinPath." -f $BinName)
    exit 1
}

# --- Place the binary ---
Write-Host "==> Installing binary to $InstallBin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -LiteralPath $SrcBin -Destination $InstallBin -Force

# --- Create data/log directory (config.toml is auto-generated on first run) ---
Write-Host "==> Creating data directory $DataDir"
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $DataDir "logs") | Out-Null

# --- (Re)create the service ---
$existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "==> Existing service found; stopping and removing it"
    if ($existing.Status -ne "Stopped") { Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue }
    sc.exe delete $ServiceName | Out-Null
    Start-Sleep -Seconds 1
}

Write-Host "==> Creating service '$ServiceName'"
# The exe with no arguments runs in Windows-service mode (service_dispatcher).
# Note: the '=' arguments to sc.exe require a trailing space (sc.exe quirk).
sc.exe create $ServiceName binPath= "`"$InstallBin`"" start= auto DisplayName= "vlt-syslogd syslog server" | Out-Null
sc.exe description $ServiceName "Receives syslog (UDP 514) and serves it to vlt-syslogd Console." | Out-Null

Write-Host "==> Starting service"
Start-Service -Name $ServiceName

Write-Host ""
Write-Host "Done."
Write-Host "  - Registered as a Windows service listening on UDP 514 (auto-starts at boot)."
Write-Host "  - Config file: $DataDir\config.toml (auto-generated on first run; edit to change ports etc.)"
Write-Host "  - Logs: $DataDir\logs\"
Write-Host "  - Status:    sc.exe query $ServiceName"
Write-Host "  - Uninstall: powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1"
