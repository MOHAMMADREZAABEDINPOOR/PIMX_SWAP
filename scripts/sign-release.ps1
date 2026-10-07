param(
    [Parameter(Mandatory=$true)][string]$Thumbprint,
    [Parameter(Mandatory=$true)][string]$TimestampUrl,
    [Parameter(Mandatory=$true)][string]$SignTool
)
$ErrorActionPreference='Stop'
. "$PSScriptRoot\env.ps1"
$taskRoot=Split-Path -Parent $PSScriptRoot
$cert=Get-Item -LiteralPath "Cert:\CurrentUser\My\$Thumbprint"
if(!$cert.HasPrivateKey -or $cert.NotAfter -le [DateTime]::Now){throw 'Certificate must have an accessible private key and be unexpired.'}
if(!($cert.EnhancedKeyUsageList.ObjectId -contains '1.3.6.1.5.5.7.3.3')){throw 'Certificate is not enabled for code signing.'}
$signConfig=Join-Path $taskRoot '.tools\signing-config.json'
New-Item -ItemType Directory -Force (Split-Path -Parent $signConfig) | Out-Null
# Let Tauri sign the embedded application before packaging, then sign the installer.
# Signing just the standalone release copy would leave the installer payload unsigned.
$configuration=@{bundle=@{windows=@{signCommand=@{cmd=$SignTool;args=@('sign','/sha1',$Thumbprint,'/s','My','/fd','SHA256','/tr',$TimestampUrl,'/td','SHA256','%1')}}}}
[IO.File]::WriteAllText($signConfig,($configuration | ConvertTo-Json -Depth 6),(New-Object Text.UTF8Encoding($false)))
Push-Location $taskRoot
try {
    npm.cmd run release -- --config $signConfig -- --locked
    if($LASTEXITCODE -ne 0){throw 'Signed Tauri build failed'}
    Copy-Item -LiteralPath 'src-tauri\target\release\PIMXSWAP.exe' -Destination 'release\PIMXSWAP.exe'
    $setup=Get-ChildItem 'src-tauri\target\release\bundle\nsis\*.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    Copy-Item -LiteralPath $setup.FullName -Destination 'release\PIMXSWAP-Setup.exe'
    foreach($name in @('PIMXSWAP.exe','PIMXSWAP-Setup.exe')) {
        & $SignTool verify /pa (Join-Path $taskRoot "release\$name")
        if($LASTEXITCODE -ne 0){throw "Signature verification failed: $name"}
    }
    & "$PSScriptRoot\package-release.ps1"
} finally {
    Pop-Location
    Remove-Item -LiteralPath $signConfig -ErrorAction SilentlyContinue
}
