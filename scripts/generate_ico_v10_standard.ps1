$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# 定義
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"
$sources = @(
    @{ Size = 16; Path = "vlt_syslogd_icon_16px.png" }
    @{ Size = 24; Path = "vlt_syslogd_icon_24px.png" }
    @{ Size = 32; Path = "vlt_syslogd_icon_32px.png" }
    @{ Size = 48; Path = "vlt_syslogd_icon_48px.png" }
    @{ Size = 64; Path = "vlt_syslogd_icon_64px.png" }
    @{ Size = 256; Path = "vlt_syslogd_icon.png" }
)

Write-Host "Assembling multi-resolution ICO (Standard Compatible Mode)..."

$imagesData = New-Object System.Collections.Generic.List[byte[]]
$headers = New-Object System.Collections.Generic.List[byte[]]

foreach ($src in $sources) {
    $fullPath = $src.Path
    if (-not (Test-Path $fullPath)) { throw "Missing source: $fullPath" }
    
    $bmp = [System.Drawing.Bitmap]::FromFile($fullPath)
    try {
        $ms = New-Object System.IO.MemoryStream
        if ($src.Size -eq 256) {
            # 256px以上はモダンなPNG圧縮形式
            $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        else {
            # 128px以下は古典的なDIB (Headerless BMP) 形式が最も互換性が高い
            # ここではIcon.Saveの内部ロジックを模倣してPNGとして保存しても良いが、
            # 破損報告があったため、確実性を期してPNG形式のまま「正しい」エントリヘッダーを書く
            $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        }
        $imagesData.Add($ms.ToArray())
        $ms.Dispose()
    }
    finally {
        $bmp.Dispose()
    }
}

# ファイル書き出し
$fs = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
try {
    $bw = New-Object System.IO.BinaryWriter($fs)
    
    # 1. ICONDIR Header
    $bw.Write([uint16]0)
    $bw.Write([uint16]1) # Icon
    $bw.Write([uint16]$sources.Count)
    
    # 2. ICONDIRENTRY (16 bytes each)
    $offset = 6 + (16 * $sources.Count)
    for ($i = 0; $i -lt $sources.Count; $i++) {
        $size = $sources[$i].Size
        $data = $imagesData[$i]
        
        $w = if ($size -ge 256) { 0 } else { $size }
        $bw.Write([byte]$w)      # Width
        $bw.Write([byte]$w)      # Height
        $bw.Write([byte]0)       # Color count
        $bw.Write([byte]0)       # Reserved
        $bw.Write([uint16]1)     # Planes
        $bw.Write([uint16]32)    # BitsPerPixel
        $bw.Write([uint32]$data.Length) # BytesInRes
        $bw.Write([uint32]$offset)      # ImageOffset
        
        $offset += $data.Length
    }
    
    # 3. Image Data
    foreach ($data in $imagesData) {
        $bw.Write($data)
    }
    
    $bw.Flush()
}
finally {
    $fs.Close()
}

Write-Host "ICO successfully generated: $icoPath"
