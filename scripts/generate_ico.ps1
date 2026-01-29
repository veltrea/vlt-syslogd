$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$pngPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.png"
$icoPath = Join-Path $PSScriptRoot "vlt_syslogd_icon.ico"

if (-not (Test-Path $pngPath)) {
    Write-Error "PNG icon not found: $pngPath"
}

$img = [System.Drawing.Image]::FromFile($pngPath)
$bitmap = New-Object System.Drawing.Bitmap($img)
$iconHandle = $bitmap.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($iconHandle)

$stream = New-Object System.IO.FileStream($icoPath, [System.IO.FileMode]::Create)
$icon.Save($stream)
$stream.Close()

$icon.Dispose()
[System.Runtime.InteropServices.Marshal]::FreeHGlobal($iconHandle)
$bitmap.Dispose()
$img.Dispose()

Write-Host "Successfully generated $icoPath"
