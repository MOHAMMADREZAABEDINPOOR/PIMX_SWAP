$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$stage=[IO.Path]::GetFullPath((Join-Path $taskRoot '.tools\installer-check'))
if(!$stage.StartsWith($taskRoot+[IO.Path]::DirectorySeparatorChar)){throw 'Invalid staging path'}
$backup=Join-Path $taskRoot '.tools\installer-backup'
New-Item -ItemType Directory -Force $backup | Out-Null
$regKeys=@('HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\PIMXSWAP','HKCU\Software\pimxswap')
$regCopies=@()
for($i=0;$i -lt $regKeys.Count;$i++){
 $path=Join-Path $backup "registry-$i.reg"
 & reg.exe export $regKeys[$i] $path /y 2>$null | Out-Null
 $regCopies+=@{key=$regKeys[$i];path=$path;existed=($LASTEXITCODE -eq 0)}
}
$links=@((Join-Path ([Environment]::GetFolderPath('Desktop')) 'PIMXSWAP.lnk'),(Join-Path ([Environment]::GetFolderPath('Programs')) 'PIMXSWAP.lnk'))
$linkCopies=@();foreach($link in $links){$linkCopies+=@{path=$link;bytes=$(if(Test-Path -LiteralPath $link){[IO.File]::ReadAllBytes($link)}else{$null})}}
$configPath=Join-Path $env:APPDATA 'com.pimxswap.desktop\settings.json'
$configBytes=if(Test-Path -LiteralPath $configPath){[IO.File]::ReadAllBytes($configPath)}else{$null}
$results=New-Object System.Collections.Generic.List[object]
try {
 Get-Process PIMXSWAP -ErrorAction SilentlyContinue | Where-Object {$_.Path -like "$taskRoot\*"} | Stop-Process
 foreach($phase in @('install','reinstall-update')) {
  $process=Start-Process -FilePath (Join-Path $taskRoot 'release\PIMXSWAP-Setup.exe') -ArgumentList '/S','/UPDATE',"/D=$stage" -WindowStyle Hidden -PassThru
  if(!$process.WaitForExit(45000)){throw 'Installer did not exit within 45 seconds'}
  if($process.ExitCode -ne 0){throw "Installer exit code $($process.ExitCode)"}
  $installed=Join-Path $stage 'PIMXSWAP.exe'
  if(!(Test-Path -LiteralPath $installed)){throw 'Installed executable missing'}
  $portable=[IO.File]::ReadAllBytes((Join-Path $taskRoot 'release\PIMXSWAP.exe'))
  $payload=[IO.File]::ReadAllBytes($installed)
  $same=$portable.Length -eq $payload.Length
  $differences=New-Object System.Collections.Generic.List[int]
  if($same){for($index=0;$index -lt $portable.Length;$index++){if($portable[$index] -ne $payload[$index]){$differences.Add($index)}}}
  if($differences.Count -gt 0){
   $same=$differences.Count -eq 3 -and $differences[1] -eq $differences[0]+1 -and $differences[2] -eq $differences[0]+2
   if($same){$same=[Text.Encoding]::ASCII.GetString($portable,$differences[0],3) -eq 'UNK' -and [Text.Encoding]::ASCII.GetString($payload,$differences[0],3) -eq 'NSS'}
  }
  # Tauri tags the standalone executable UNK and the NSIS payload NSS; every other byte must match.
  $results.Add([pscustomobject]@{test=$phase;passed=$same;exitCode=$process.ExitCode})
  foreach($name in @('THIRD-PARTY-NOTICES.txt','PRIVACY.md','QUICKSTART.md')){
   $actual=Join-Path $stage $name
   if(!(Test-Path -LiteralPath $actual)){throw "Bundled resource missing: $name"}
   if((Get-FileHash -LiteralPath $actual).Hash -ne (Get-FileHash -LiteralPath (Join-Path $taskRoot "docs\$name")).Hash){throw "Bundled resource differs: $name"}
  }
  Write-Output "$phase : $same"
 }
 Copy-Item -LiteralPath (Join-Path $stage 'PIMXSWAP.exe') -Destination (Join-Path $backup 'installed-payload.exe')
 $uninstaller=Join-Path $stage 'uninstall.exe'
 $process=Start-Process -FilePath $uninstaller -ArgumentList '/S','/UPDATE' -WindowStyle Hidden -PassThru
 if(!$process.WaitForExit(45000)){throw 'Uninstaller did not exit'}
 for($i=0;$i -lt 50 -and (Test-Path -LiteralPath (Join-Path $stage 'PIMXSWAP.exe'));$i++){Start-Sleep -Milliseconds 100}
 $removed=!(Test-Path -LiteralPath (Join-Path $stage 'PIMXSWAP.exe'))
 $results.Add([pscustomobject]@{test='uninstall';passed=$removed;exitCode=$process.ExitCode})
 Write-Output "uninstall : $removed"
 $preserved=if($configBytes){[Convert]::ToBase64String([IO.File]::ReadAllBytes($configPath)) -eq [Convert]::ToBase64String($configBytes)}else{!(Test-Path -LiteralPath $configPath)}
 $results.Add([pscustomobject]@{test='settings-preserved';passed=$preserved})
 if(!$preserved){throw 'User preferences changed during packaging test'}
} finally {
 foreach($item in $regCopies){if($item.existed){& reg.exe import $item.path | Out-Null}else{& reg.exe delete $item.key /f 2>$null | Out-Null}}
 foreach($item in $linkCopies){if($item.bytes){[IO.File]::WriteAllBytes($item.path,$item.bytes)}elseif(Test-Path -LiteralPath $item.path){Remove-Item -LiteralPath $item.path}}
 if($configBytes){[IO.File]::WriteAllBytes($configPath,$configBytes)}
 $results | ConvertTo-Json | Set-Content -Encoding UTF8 (Join-Path $taskRoot 'test-results\installer-release.json')
}
if(@($results | Where-Object {!$_.passed}).Count){exit 1}
