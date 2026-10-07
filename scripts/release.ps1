$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\env.ps1"
$taskRoot = Split-Path -Parent $PSScriptRoot
Push-Location $taskRoot
try {
    npm.cmd ci
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
    cargo fmt --manifest-path src-tauri/Cargo.toml --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed' }
    cargo test --locked --manifest-path src-tauri/Cargo.toml -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed' }
    cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    node scripts/notices.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Dependency notices failed' }
    npm.cmd audit --omit=dev
    if ($LASTEXITCODE -ne 0) { throw 'Production dependency audit failed' }
    node scripts/audit-dependencies.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Rust dependency advisory check failed' }
    npm.cmd run release -- -- --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tauri build failed' }
    New-Item -ItemType Directory -Force release | Out-Null
    Copy-Item -LiteralPath 'src-tauri\target\release\PIMXSWAP.exe' -Destination 'release\PIMXSWAP.exe'
    $setup = Get-ChildItem 'src-tauri\target\release\bundle\nsis\*.exe' | Select-Object -First 1
    Copy-Item -LiteralPath $setup.FullName -Destination 'release\PIMXSWAP-Setup.exe'
    & "$PSScriptRoot\package-release.ps1"
} finally { Pop-Location }
