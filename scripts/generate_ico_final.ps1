$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# 定義 - 相対パスを scripts フォルダからの視点に調整
$baseDir = Split-Path $PSScriptRoot -Parent
$icoPath = Join-Path $baseDir "vlt_syslogd_icon.ico"
$iconDir = Join-Path $baseDir "icons"

# ユーザーの意図通り、自動縮小ではなく「個別に用意されたPNG」を使用します
$sources = @(
    @{ Size = 16; Path = Join-Path $iconDir "vlt_syslogd_icon_16px.png" }
    @{ Size = 24; Path = Join-Path $iconDir "vlt_syslogd_icon_24px.png" }
    @{ Size = 32; Path = Join-Path $iconDir "vlt_syslogd_icon_32px.png" }
    @{ Size = 48; Path = Join-Path $iconDir "vlt_syslogd_icon_48px.png" }
    @{ Size = 64; Path = Join-Path $iconDir "vlt_syslogd_icon_64px.png" }
    @{ Size = 256; Path = Join-Path $iconDir "vlt_syslogd_icon_256px.png" }
)

Write-Host "Assembling multi-resolution ICO from individual PNGs..."

$imagesData = New-Object System.Collections.Generic.List[byte[]]

foreach ($src in $sources) {
    if (-not (Test-Path $src.Path)) { throw "Missing source: $($src.Path)" }
    $bmp = [System.Drawing.Bitmap]::FromFile($src.Path)
    try {
        $ms = New-Object System.IO.MemoryStream
        $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $imagesData.Add($ms.ToArray())
        $ms.Dispose()
    }
    finally {
        $bmp.Dispose()
    }
}

$fs = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
try {
    $bw = New-Object System.IO.BinaryWriter($fs)
    $bw.Write([uint16]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]$sources.Count)
    
    $offset = 6 + (16 * $sources.Count)
    for ($i = 0; $i -lt $sources.Count; $i++) {
        $size = $sources[$i].Size
        $data = $imagesData[$i]
        $w = if ($size -ge 256) { 0 } else { $size }
        $bw.Write([byte]$w)
        $bw.Write([byte]$w)
        $bw.Write([byte]0)
        $bw.Write([byte]0)
        $bw.Write([uint16]1)
        $bw.Write([uint16]32)
        $bw.Write([uint32]$data.Length)
        $bw.Write([uint32]$offset)
        $offset += $data.Length
    }
    foreach ($data in $imagesData) { $bw.Write($data) }
    $bw.Flush()
}
finally {
    $fs.Close()
}

Write-Host "ICO successfully generated: $icoPath"
