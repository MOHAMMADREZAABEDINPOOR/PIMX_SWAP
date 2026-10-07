# Dot-source this file to use an optional repository-local toolchain.
$taskRoot = Split-Path -Parent $PSScriptRoot
$nodeDir = Get-ChildItem -LiteralPath (Join-Path $taskRoot '.tools') -Filter 'node-v*-win-x64' -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nodeDir) { $env:Path = "$($nodeDir.FullName);$env:Path" }
if (Test-Path (Join-Path $taskRoot '.tools\cargo\bin')) {
    $env:CARGO_HOME = Join-Path $taskRoot '.tools\cargo'
    $env:RUSTUP_HOME = Join-Path $taskRoot '.tools\rustup'
    $env:Path = "$env:CARGO_HOME\bin;$env:Path"
}
