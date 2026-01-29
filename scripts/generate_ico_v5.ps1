$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$pngPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Generating pixel-perfect multi-resolution ICO..."

if (-not (Test-Path $pngPath)) {
    throw "Source PNG not found: $pngPath"
}

$sourceImg = [System.Drawing.Image]::FromFile($pngPath)

try {
    # 全ての表示モードに対応する解像度セット
    $sizes = @(16, 24, 32, 48, 64, 256)
    $imagesData = New-Object System.Collections.Generic.List[byte[]]

    foreach ($size in $sizes) {
        $bitmap = New-Object System.Drawing.Bitmap($size, $size)
        try {
            $g = [System.Drawing.Graphics]::FromImage($bitmap)
            
            # 画質設定を最高に
            $g.Clear([System.Drawing.Color]::Transparent)
            $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
            $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
            $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
            
            # 描画
            $g.DrawImage($sourceImg, 0, 0, $size, $size)
            $g.Dispose()

            # PNGとして保存 (透過とアルファチャンネルを完全に維持)
            $ms = New-Object System.IO.MemoryStream
            $bitmap.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
            $imagesData.Add($ms.ToArray())
            $ms.Close()
        }
        finally {
            $bitmap.Dispose()
        }
    }

    # ICOバイナリ構築
    $fs = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
    try {
        $bw = New-Object System.IO.BinaryWriter($fs)

        # Header
        $bw.Write([uint16]0)
        $bw.Write([uint16]1)
        $bw.Write([uint16]$sizes.Count)

        $offset = 6 + (16 * $sizes.Count)

        # Directory Entries
        for ($i = 0; $i -lt $sizes.Count; $i++) {
            $size = $sizes[$i]
            $data = $imagesData[$i]

            $w = if ($size -ge 256) { 0 } else { $size }
            $h = if ($size -ge 256) { 0 } else { $size }

            $bw.Write([byte]$w)
            $bw.Write([byte]$h)
            $bw.Write([byte]0) # Colors
            $bw.Write([byte]0) # Reserved
            $bw.Write([uint16]1)
            $bw.Write([uint16]32)
            $bw.Write([uint32]$data.Length)
            $bw.Write([uint32]$offset)

            $offset += $data.Length
        }

        # PNG Data
        foreach ($data in $imagesData) {
            $bw.Write($data)
        }
        $bw.Flush()
    }
    finally {
        $fs.Close()
    }
}
finally {
    $sourceImg.Dispose()
}

Write-Host "Success: Multi-res ICO created at $icoPath"
