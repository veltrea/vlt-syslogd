$ErrorActionPreference = 'Stop'
$pngPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Manually wrapping $pngPath into ICO format..."

if (-not (Test-Path $pngPath)) {
    throw "Source PNG not found: $pngPath"
}

$pngBytes = [System.IO.File]::ReadAllBytes($pngPath)
$pngSize = $pngBytes.Length

# ICO Header (6 bytes) + Directory Entry (16 bytes)
$icoData = New-Object byte[] (22 + $pngSize)

# ICO Header
$icoData[0] = 0x00; $icoData[1] = 0x00 # Reserved
$icoData[2] = 0x01; $icoData[3] = 0x00 # Type: Icon
$icoData[4] = 0x01; $icoData[5] = 0x00 # Count: 1

# Directory Entry
$icoData[6] = 0x00 # Width: 256
$icoData[7] = 0x00 # Height: 256
$icoData[8] = 0x00 # Color count
$icoData[9] = 0x00 # Reserved
$icoData[10] = 0x01; $icoData[11] = 0x00 # Color planes
$icoData[12] = 0x20; $icoData[13] = 0x00 # Bits per pixel: 32
# Size of data (4 bytes)
$sizeBytes = [System.BitConverter]::GetBytes($pngSize)
[System.Array]::Copy($sizeBytes, 0, $icoData, 14, 4)
# Offset (4 bytes): Header(6) + Entry(16) = 22
$icoData[18] = 0x16; $icoData[19] = 0x00; $icoData[20] = 0x00; $icoData[21] = 0x00

# Copy PNG data
[System.Array]::Copy($pngBytes, 0, $icoData, 22, $pngSize)

[System.IO.File]::WriteAllBytes($icoPath, $icoData)

Write-Host "Successfully generated PNG-wrapped ICO: $icoPath ($($icoData.Length) bytes)"
