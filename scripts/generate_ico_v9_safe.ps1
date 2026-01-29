$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# 素材パス
$png16 = Join-Path $PSScriptRoot "vlt_syslogd_icon_16px.png"
$png32 = Join-Path $PSScriptRoot "vlt_syslogd_icon_32px.png"
$pngLarge = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$targetIco = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Creating ICO using System.Drawing (Safe Mode)..."

# 各素材をBitmapとして読み込み
$bmp16 = [System.Drawing.Bitmap]::FromFile($png16)
$bmp32 = [System.Drawing.Bitmap]::FromFile($png32)
$bmpLarge = [System.Drawing.Bitmap]::FromFile($pngLarge)

# .NETの標準機能では複数のBitmapから1つのIconを作るのが難しいため、
# 互換性の高い構造で再構築します。
$images = @(
    @{ Size = 16; Bmp = $bmp16 }
    @{ Size = 32; Bmp = $bmp32 }
    @{ Size = 256; Bmp = $bmpLarge }
)

$msList = New-Object System.Collections.Generic.List[System.IO.MemoryStream]
try {
    foreach ($img in $images) {
        $ms = New-Object System.IO.MemoryStream
        # PNGとして保存（ICO内の各エントリはPNGをサポートしている）
        $img.Bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $msList.Add($ms)
    }

    $fs = New-Object System.IO.FileStream($targetIco, [System.IO.FileMode]::Create)
    try {
        $bw = New-Object System.IO.BinaryWriter($fs)

        # ICO Header
        $bw.Write([uint16]0)
        $bw.Write([uint16]1)
        $bw.Write([uint16]$images.Count)

        $offset = 6 + (16 * $images.Count)
        foreach ($ms in $msList) {
            $data = $ms.ToArray()
            $idx = $msList.IndexOf($ms)
            $size = $images[$idx].Size
            $w = if ($size -ge 256) { 0 } else { $size }

            # Directory Entry
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

        # Data
        foreach ($ms in $msList) {
            $bw.Write($ms.ToArray())
        }
    }
    finally {
        $fs.Close()
    }
}
finally {
    foreach ($ms in $msList) { $ms.Dispose() }
    $bmp16.Dispose()
    $bmp32.Dispose()
    $bmpLarge.Dispose()
}

Write-Host "Success: Generated safe ICO at $targetIco"
