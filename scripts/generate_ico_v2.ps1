$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$pngPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

if (-not (Test-Path $pngPath)) {
    Write-Host "PNG not found at $pngPath"
    exit 1
}

$bmp = [System.Drawing.Image]::FromFile($pngPath)
# 多彩なサイズを含めるのが理想ですが、まずは高品質な256x256を確実に作成します
$newBmp = New-Object System.Drawing.Bitmap(256, 256)
$g = [System.Drawing.Graphics]::FromImage($newBmp)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.DrawImage($bmp, 0, 0, 256, 256)
$g.Dispose()

$hIcon = $newBmp.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($hIcon)

# 重要な修正: FileStreamを使用して確実に書き込む
$fs = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
$icon.Save($fs)
$fs.Close()

$icon.Dispose()
[System.Runtime.InteropServices.Marshal]::FreeHGlobal($hIcon)
$newBmp.Dispose()
$bmp.Dispose()

Write-Host "Generated high-quality ICO: $icoPath"
