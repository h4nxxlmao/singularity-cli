# scripts/smoke.ps1 — Real-repo smoke test for singularity-cli (sgl)
# Usage: powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1

$ErrorActionPreference = 'Continue'

$CargoDir = Resolve-Path "$PSScriptRoot\.."
cargo build --manifest-path "$CargoDir\Cargo.toml"
$Sgl = "$CargoDir\target\debug\sgl.exe"

$TmpDir = Join-Path $env:TEMP ("sgl-smoke-" + [System.Guid]::NewGuid().ToString().Substring(0, 8))
New-Item -ItemType Directory -Force $TmpDir | Out-Null

Write-Host "Running smoke tests in $TmpDir..."

$Projects = @(
    @{ Name = "Vite App"; Url = "https://github.com/vitejs/vite"; Expected = "node" },
    @{ Name = "Next.js App"; Url = "https://github.com/vercel/next-learn"; Expected = "node" },
    @{ Name = "npm library"; Url = "https://github.com/lodash/lodash"; Expected = "node" },
    @{ Name = "ripgrep (Rust)"; Url = "https://github.com/BurntSushi/ripgrep"; Expected = "rust" },
    @{ Name = "Go CLI"; Url = "https://github.com/charmbracelet/glow"; Expected = "go" },
    @{ Name = "FastAPI project"; Url = "https://github.com/tiangolo/full-stack-fastapi-template"; Expected = "python" },
    @{ Name = "Maven project"; Url = "https://github.com/spring-projects/spring-petclinic"; Expected = "java" },
    @{ Name = "Gradle project"; Url = "https://github.com/square/okhttp"; Expected = "java" }
)

$Verbs = @("setup", "dev", "test", "build", "lint", "fmt")

Write-Host ""
Write-Host ("{0,-24} {1,-8} {2,-6} {3,-6} {4,-6} {5,-6} {6,-6} {7,-6}" -f "Project", "Kind", "setup", "dev", "test", "build", "lint", "fmt")
Write-Host ("-" * 79)

$Total = 0
$Passed = 0

foreach ($p in $Projects) {
    $dirName = $p.Name -replace '[ /]', '_'
    $cloneDir = Join-Path $TmpDir $dirName

    git clone --depth 1 -q $p.Url $cloneDir 2>$null
    if (-not (Test-Path $cloneDir)) {
        Write-Host "Warning: failed to clone $($p.Url), skipping"
        continue
    }

    $infoOut = & $Sgl info --json --cwd $cloneDir 2>$null | Out-String
    $detectedKind = "none"
    if ($infoOut -match '"kind":\s*"([^"]+)"') {
        $detectedKind = $Matches[1]
    }

    $kindStatus = if ($detectedKind -eq $p.Expected -or $detectedKind -ne "none") { "ok" } else { "FAIL" }

    $verbStatuses = @()
    foreach ($v in $Verbs) {
        & $Sgl --explain $v --cwd $cloneDir 2>$null | Out-Null
        if ($LASTEXITCODE -eq 0) {
            $verbStatuses += "✓"
        } else {
            $verbStatuses += "-"
        }
    }

    $Total++
    if ($kindStatus -eq "ok") { $Passed++ }

    Write-Host ("{0,-24} {1,-8} {2,-6} {3,-6} {4,-6} {5,-6} {6,-6} {7,-6}" -f `
        $p.Name, $detectedKind, $verbStatuses[0], $verbStatuses[1], $verbStatuses[2], $verbStatuses[3], $verbStatuses[4], $verbStatuses[5])
}

Write-Host ("-" * 79)
Write-Host "Smoke test complete: $Passed / $Total projects detected."

Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
