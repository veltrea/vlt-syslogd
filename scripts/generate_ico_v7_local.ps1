$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# ローカルフォルダの素材を使用
$png16Path = Join-Path $PSScriptRoot "vlt_syslogd_icon_16px.png"
$png32Path = Join-Path $PSScriptRoot "vlt_syslogd_icon_32px.png"
$pngOriginalPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Assembling multi-res ICO from local assets..."

$imagesData = New-Object System.Collections.Generic.List[byte[]]
# サポートする全サイズ。16と32は専用ドット絵、24/48などは拡大・縮小
$sizes = @(16, 24, 32, 48, 64, 256)

foreach ($size in $sizes) {
    if ($size -le 16) { $sourceFile = $png16Path }
    elseif ($size -le 32) { $sourceFile = $png32Path }
    else { $sourceFile = $pngOriginalPath }

    if (-not (Test-Path $sourceFile)) { throw "Source missing: $sourceFile" }

    $img = [System.Drawing.Image]::FromFile($sourceFile)
    try {
        $bitmap = New-Object System.Drawing.Bitmap($size, $size)
        try {
            $g = [System.Drawing.Graphics]::FromImage($bitmap)
            $g.Clear([System.Drawing.Color]::Transparent)
            
            # ドット絵サイズはニアレストネイバーで維持
            if ($size -eq 16 -or $size -eq 32) {
                $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
            }
            else {
                $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            }
            
            $g.DrawImage($img, 0, 0, $size, $size)
            $g.Dispose()

            $ms = New-Object System.IO.MemoryStream
            $bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
            $imagesData.Add($ms.ToArray())
            $ms.Close()
        }
        finally {
            $bitmap.Dispose()
        }
    }
    finally {
        $img.Dispose()
    }
}

# ICOバイナリ構築
$fs = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
try {
    $bw = New-Object System.IO.BinaryWriter($fs)
    $bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$sizes.Count)
    $offset = 6 + (16 * $sizes.Count)
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $size = $sizes[$i]; $data = $imagesData[$i]
        $w = if ($size -ge 256) { 0 } else { $size }
        $bw.Write([byte]$w); $bw.Write([byte]$w)
        $bw.Write([byte]0); $bw.Write([byte]0)
        $bw.Write([uint16]1); $bw.Write([uint16]32)
        $bw.Write([uint32]$data.Length); $bw.Write([uint32]$offset)
        $offset += $data.Length
    }
    foreach ($data in $imagesData) { $bw.Write($data) }
}
finally {
    $fs.Close()
}

Write-Host "Success: Local multi-res ICO generated."
