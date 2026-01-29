$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$pngPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

Write-Host "Converting $pngPath to $icoPath..."

if (-not (Test-Path $pngPath)) {
    throw "Source PNG not found: $pngPath"
}

$img = [System.Drawing.Image]::FromFile($pngPath)
try {
    # 256x256のビットマップを作成
    $bitmap = New-Object System.Drawing.Bitmap(256, 256)
    try {
        $g = [System.Drawing.Graphics]::FromImage($bitmap)
        $g.Clear([System.Drawing.Color]::Transparent)
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.DrawImage($img, 0, 0, 256, 256)
        $g.Dispose()

        # GetHiconは透過を保持しないことが多いため、Iconとしての保存を試行
        $hIcon = $bitmap.GetHicon()
        try {
            $icon = [System.Drawing.Icon]::FromHandle($hIcon)
            $stream = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
            $icon.Save($stream)
            $stream.Close()
            $icon.Dispose()
        }
        finally {
            # Hiconはアンマネージドリソースなので必ず解放が必要
            [System.Runtime.InteropServices.Marshal]::FreeHGlobal($hIcon)
        }
    }
    finally {
        $bitmap.Dispose()
    }
}
finally {
    $img.Dispose()
}

Write-Host "Successfully generated ICO with transparency."
