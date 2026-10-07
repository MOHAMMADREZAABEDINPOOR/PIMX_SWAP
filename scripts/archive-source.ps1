$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$taskRoot = Split-Path -Parent $PSScriptRoot
$outputDir = Join-Path $taskRoot 'release'
New-Item -ItemType Directory -Force $outputDir | Out-Null
$output = Join-Path $outputDir 'PIMXSWAP-Source.zip'
$stream = [System.IO.File]::Open($output,[System.IO.FileMode]::Create)
$archive = New-Object System.IO.Compression.ZipArchive($stream,[System.IO.Compression.ZipArchiveMode]::Create)
try {
    $entries = @('src','assets','docs','language_profiles','scripts','public','test-results')
    $files = @($entries | ForEach-Object { Get-ChildItem -LiteralPath (Join-Path $taskRoot $_) -Recurse -File })
    $files += @(Get-ChildItem -LiteralPath (Join-Path $taskRoot 'src-tauri\src') -Recurse -File)
    $files += @(Get-ChildItem -LiteralPath (Join-Path $taskRoot 'src-tauri\tests') -Recurse -File)
    $files += @(Get-ChildItem -LiteralPath (Join-Path $taskRoot 'src-tauri\icons') -Recurse -File)
    $files += @(Get-ChildItem -LiteralPath (Join-Path $taskRoot 'src-tauri\capabilities') -Recurse -File)
    foreach ($name in @('.gitignore','package.json','package-lock.json','index.html','README.md','tsconfig.json','vite.config.ts','src-tauri\Cargo.toml','src-tauri\Cargo.lock','src-tauri\build.rs','src-tauri\tauri.conf.json')) { $files += Get-Item -LiteralPath (Join-Path $taskRoot $name) }
    foreach ($file in $files) {
        $entry = $file.FullName.Substring($taskRoot.Length + 1).Replace('\','/')
        [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive,$file.FullName,$entry,[System.IO.Compression.CompressionLevel]::Optimal)
    }
} finally { $archive.Dispose(); $stream.Dispose() }
Write-Output $output
