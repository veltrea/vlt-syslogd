# vlt-syslogd: Windows uninstall script (Windows Service)
#
# Stops and removes the service registered by install-windows.ps1 and deletes
# the installed binary. The data/log directory (%ProgramData%\vlt-syslogd) is
# kept on purpose.
#
# NOTE: ASCII-only on purpose (see install-windows.ps1). Japanese explanation is
# in the install manual (DOCS/INSTALL.ja.md).
#
# Usage (run from an elevated PowerShell):
#   powershell -ExecutionPolicy Bypass -File .\uninstall-windows.ps1
#
$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ServiceName = "vlt-syslogd-srv"
$InstallDir  = Join-Path $env:ProgramFiles "vlt-syslogd"
$InstallBin  = Join-Path $InstallDir "vlt-syslogd-srv.exe"
$DataDir     = Join-Path $env:ProgramData "vlt-syslogd"

# --- Require administrator ---
$isAdmin = ([Security.Principal.WindowsPrincipal] `
    [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Error "Administrator rights are required. Run PowerShell as Administrator and re-run this script."
    exit 1
}

$existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "==> Stopping and removing service '$ServiceName'"
    if ($existing.Status -ne "Stopped") { Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue }
    sc.exe delete $ServiceName | Out-Null
} else {
    Write-Host "==> Service '$ServiceName' not found (already removed)."
}

if (Test-Path -LiteralPath $InstallBin) {
    Write-Host "==> Removing binary $InstallBin"
    Remove-Item -LiteralPath $InstallBin -Force
}
# Remove the install dir if empty
if ((Test-Path -LiteralPath $InstallDir) -and -not (Get-ChildItem -LiteralPath $InstallDir -Force)) {
    Remove-Item -LiteralPath $InstallDir -Force
}

Write-Host ""
Write-Host "Done."
Write-Host "  - Config and logs were kept: $DataDir"
Write-Host "  - Remove them manually if unneeded: Remove-Item -Recurse -Force '$DataDir'"
