$ErrorActionPreference = 'Stop'

# ソースPNG（ピクセルパーフェクト素材）の定義
$png16 = Join-Path $PSScriptRoot "vlt_syslogd_icon_16px.png"
$png32 = Join-Path $PSScriptRoot "vlt_syslogd_icon_32px.png"
$pngLarge = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"

$targetIco = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Assembling ICO using RAW PNG bytes (zero conversion)..."

# バイナリとして直接読み込み (System.Drawingは一切使用しない)
$data16 = [System.IO.File]::ReadAllBytes($png16)
$data32 = [System.IO.File]::ReadAllBytes($png32)
$data256 = [System.IO.File]::ReadAllBytes($pngLarge)

$images = @(
    @{ Size = 16; Data = $data16 }
    @{ Size = 32; Data = $data32 }
    @{ Size = 256; Data = $data256 }
)

$count = $images.Count
$fs = New-Object System.IO.FileStream($targetIco, [System.IO.FileMode]::Create)
try {
    $bw = New-Object System.IO.BinaryWriter($fs)

    # ICO Header (6 bytes)
    $bw.Write([uint16]0) # Reserved
    $bw.Write([uint16]1) # Type (1=Icon)
    $bw.Write([uint16]$count)

    # Directory Entries (16 bytes each)
    $offset = 6 + (16 * $count)

    foreach ($img in $images) {
        $size = $img.Size
        $w = if ($size -ge 256) { 0 } else { $size }
        
        $bw.Write([byte]$w)      # Width
        $bw.Write([byte]$w)      # Height
        $bw.Write([byte]0)       # Palette
        $bw.Write([byte]0)       # Reserved
        $bw.Write([uint16]1)     # Planes
        $bw.Write([uint16]32)    # BPP
        $bw.Write([uint32]$img.Data.Length) # Size
        $bw.Write([uint32]$offset) # Offset
        
        $offset += $img.Data.Length
    }

    # Image Data (RAW PNG BYTES)
    foreach ($img in $images) {
        $bw.Write($img.Data)
    }

    $bw.Flush()
}
finally {
    $fs.Close()
}

Write-Host "Successfully generated pixel-perfect ICO: $targetIco"
