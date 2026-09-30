$ErrorActionPreference = 'Stop'

Write-Host "Running cargo fmt..." -ForegroundColor Cyan
cargo fmt --check
if ($LASTEXITCODE -ne 0) { throw "cargo fmt failed" }

Write-Host "Running cargo clippy..." -ForegroundColor Cyan
cargo clippy -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed" }

Write-Host "Running cargo build..." -ForegroundColor Cyan
cargo build
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

Write-Host "Running cargo test..." -ForegroundColor Cyan
cargo test
if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }

Write-Host "Running smoke tests..." -ForegroundColor Cyan
$sgl = ".\target\debug\sgl.exe"

& $sgl info
if ($LASTEXITCODE -ne 0) { throw "smoke test 'info' failed" }

& $sgl info --json | Out-Null
if ($LASTEXITCODE -ne 0) { throw "smoke test 'info --json' failed" }

& $sgl --help | Out-Null
if ($LASTEXITCODE -ne 0) { throw "smoke test '--help' failed" }

Write-Host "All checks PASS!" -ForegroundColor Green
