$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$release=Join-Path $taskRoot 'release'
New-Item -ItemType Directory -Force $release | Out-Null
foreach($name in @('PIMXSWAP.exe','PIMXSWAP-Setup.exe')){if(!(Test-Path -LiteralPath (Join-Path $release $name))){throw "Missing release binary: $name"}}
Copy-Item -LiteralPath (Join-Path $taskRoot 'docs\THIRD-PARTY-NOTICES.txt'),(Join-Path $taskRoot 'docs\PRIVACY.md'),(Join-Path $taskRoot 'docs\QUICKSTART.md'),(Join-Path $taskRoot 'docs\release-checklist.md') -Destination $release
$faGuide=Get-ChildItem -LiteralPath (Join-Path $taskRoot 'docs') -Filter '*.md' | Where-Object {$_.Name -match '[^\x00-\x7F]'}
foreach($file in $faGuide){Copy-Item -LiteralPath $file.FullName -Destination $release}
& "$PSScriptRoot\archive-source.ps1"
$stage=[IO.Path]::GetFullPath((Join-Path $taskRoot '.tools\release-package'))
$workspace=[IO.Path]::GetFullPath($taskRoot)+[IO.Path]::DirectorySeparatorChar
if(!$stage.StartsWith($workspace,[StringComparison]::OrdinalIgnoreCase)){throw 'Packaging stage is outside the workspace'}
if(Test-Path -LiteralPath $stage){Remove-Item -LiteralPath $stage -Recurse -Force}
New-Item -ItemType Directory -Path $stage | Out-Null
Get-ChildItem -LiteralPath $release -File | Where-Object {$_.Extension -ne '.zip' -and $_.Name -ne 'SHA256.txt'} | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination $stage}
$stageHashes=@(Get-ChildItem -LiteralPath $stage -File | Get-FileHash -Algorithm SHA256 | ForEach-Object {"$($_.Hash)  $(Split-Path -Leaf $_.Path)"})
[IO.File]::WriteAllLines((Join-Path $stage 'SHA256.txt'),$stageHashes,(New-Object Text.UTF8Encoding($false)))
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath (Join-Path $release 'PIMXSWAP-Windows-x64.zip') -Force
$releaseHashes=@(Get-ChildItem -LiteralPath $release -File | Where-Object Name -ne 'SHA256.txt' | Get-FileHash -Algorithm SHA256 | ForEach-Object {"$($_.Hash)  $(Split-Path -Leaf $_.Path)"})
[IO.File]::WriteAllLines((Join-Path $release 'SHA256.txt'),$releaseHashes,(New-Object Text.UTF8Encoding($false)))
Write-Output 'Distribution ZIP, source archive and SHA-256 manifests refreshed.'
