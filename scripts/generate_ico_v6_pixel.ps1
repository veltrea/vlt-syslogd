$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# パスの定義
$baseDir = "C:\Users\pcadmin\.gemini\antigravity\brain\10eaacc2-a1b0-45da-8ef0-ce6bbd4a4296"
$png16Path = Join-Path $baseDir "vlt_syslogd_icon_16px_pixelart_a_1769674270695.png"
$png32Path = Join-Path $baseDir "vlt_syslogd_icon_32px_pixelart_a_1769674303371.png"
$pngOriginalPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Assembling pixel-perfect multi-resolution ICO..."

$imagesData = New-Object System.Collections.Generic.List[byte[]]
$sizes = @(16, 24, 32, 48, 64, 256)

foreach ($size in $sizes) {
    $sourceFile = ""
    if ($size -eq 16) { $sourceFile = $png16Path }
    elseif ($size -eq 32) { $sourceFile = $png32Path }
    else { $sourceFile = $pngOriginalPath }

    $img = [System.Drawing.Image]::FromFile($sourceFile)
    try {
        $bitmap = New-Object System.Drawing.Bitmap($size, $size)
        try {
            $g = [System.Drawing.Graphics]::FromImage($bitmap)
            $g.Clear([System.Drawing.Color]::Transparent)
            
            # ドット絵(16, 32)の場合は補間をニアレストネイバーにしてシャープに保つ
            # それ以外は高品質縮小を使用
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
    $bw.Write([uint16]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]$sizes.Count)

    $offset = 6 + (16 * $sizes.Count)
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $size = $sizes[$i]; $data = $imagesData[$i]
        $w = if ($size -ge 256) { 0 } else { $size }
        $bw.Write([byte]$w); $bw.Write([byte]$w) # Width/Height
        $bw.Write([byte]0); $bw.Write([byte]0)   # Colors/Reserved
        $bw.Write([uint16]1); $bw.Write([uint16]32) # Planes/BPP
        $bw.Write([uint32]$data.Length); $bw.Write([uint32]$offset)
        $offset += $data.Length
    }
    foreach ($data in $imagesData) { $bw.Write($data) }
    $bw.Flush()
}
finally {
    $fs.Close()
}

Write-Host "Success: Pixel-perfect ICO created at $icoPath"
