$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$taskRoot = Split-Path -Parent $PSScriptRoot
$iconDir = Join-Path $taskRoot 'src-tauri\icons'
$assetDir = Join-Path $taskRoot 'assets\icons'
$publicDir = Join-Path $taskRoot 'public'
foreach ($dir in @($iconDir,$assetDir,$publicDir)) { New-Item -ItemType Directory -Force $dir | Out-Null }
function Draw-Icon([int]$size,[bool]$tray) {
    $bmp = New-Object System.Drawing.Bitmap($size,$size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.ScaleTransform($size / 64.0,$size / 64.0)
    if (!$tray) {
        $path = New-Object System.Drawing.Drawing2D.GraphicsPath
        $path.AddArc(0,0,28,28,180,90); $path.AddArc(36,0,28,28,270,90)
        $path.AddArc(36,36,28,28,0,90); $path.AddArc(0,36,28,28,90,90); $path.CloseFigure()
        $brush = New-Object System.Drawing.SolidBrush([System.Drawing.ColorTranslator]::FromHtml('#15261e'))
        $g.FillPath($brush,$path); $brush.Dispose(); $path.Dispose()
    }
    $top = @(@(12,12),@(43,12),@(54,23),@(43,34),@(43,26),@(24,26),@(16,34),@(4,34),@(20,18),@(43,18),@(37,12))
    $bottom = @(@(52,52),@(21,52),@(10,41),@(21,30),@(21,38),@(40,38),@(48,30),@(60,30),@(44,46),@(21,46),@(27,52))
    foreach ($shape in @($top,$bottom)) {
        $points = [System.Drawing.PointF[]]@($shape | ForEach-Object { New-Object System.Drawing.PointF($_[0],$_[1]) })
        $alpha = if ($shape -eq $bottom -and !$tray) { 150 } else { 255 }
        $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb($alpha,156,232,195))
        $g.FillPolygon($brush,$points); $brush.Dispose()
    }
    $g.Dispose(); return $bmp
}
foreach ($size in @(16,32,48,64,128,256,512,1024)) {
    $bmp = Draw-Icon $size $false
    $bmp.Save((Join-Path $assetDir "$size.png"),[System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
}
Copy-Item -LiteralPath (Join-Path $assetDir '32.png') -Destination (Join-Path $iconDir '32x32.png')
Copy-Item -LiteralPath (Join-Path $assetDir '128.png') -Destination (Join-Path $iconDir '128x128.png')
Copy-Item -LiteralPath (Join-Path $assetDir '256.png') -Destination (Join-Path $iconDir '128x128@2x.png')
$trayBmp = Draw-Icon 32 $true
$trayBmp.Save((Join-Path $iconDir 'tray.png'),[System.Drawing.Imaging.ImageFormat]::Png); $trayBmp.Dispose()
$ico = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($ico)
$sizes = @(16,32,48,64,128,256)
$writer.Write([UInt16]0); $writer.Write([UInt16]1); $writer.Write([UInt16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
$buffers = @()
foreach ($size in $sizes) {
    $bytes = [System.IO.File]::ReadAllBytes((Join-Path $assetDir "$size.png")); $buffers += ,$bytes
    $dimension = if ($size -eq 256) {0} else {$size}
    $writer.Write([byte]$dimension); $writer.Write([byte]$dimension); $writer.Write([byte]0); $writer.Write([byte]0)
    $writer.Write([UInt16]1); $writer.Write([UInt16]32); $writer.Write([UInt32]$bytes.Length); $writer.Write([UInt32]$offset)
    $offset += $bytes.Length
}
foreach ($bytes in $buffers) {$writer.Write([byte[]]$bytes)}
$writer.Flush(); [System.IO.File]::WriteAllBytes((Join-Path $iconDir 'icon.ico'),$ico.ToArray())
Copy-Item -LiteralPath (Join-Path $iconDir 'icon.ico') -Destination (Join-Path $publicDir 'favicon.ico')
Copy-Item -LiteralPath (Join-Path $iconDir 'icon.ico') -Destination (Join-Path $taskRoot 'assets\favicon.ico')
$writer.Dispose(); $ico.Dispose()
Write-Output 'PIMXSWAP icons generated (16–1024 px).'
