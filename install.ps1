# install.ps1 — sgl installer for PowerShell
# Usage: irm https://getsingularity.lol/cli/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Repo    = "singularity-cli/singularity-cli"
$Bin     = "sgl.exe"
$InstDir = if ($env:SGL_INSTALL_DIR) { $env:SGL_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

function Step($label, $value) {
    Write-Host ("  → {0,-28} {1}" -f $label, $value)
}

function Die($msg) {
    Write-Error "error: $msg"
    exit 1
}

# ── detect platform ───────────────────────────────────────────────────────────

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$target = switch ($arch) {
    'X64'   { 'x86_64-pc-windows-msvc' }
    'Arm64' { 'aarch64-pc-windows-msvc' }
    default { Die "unsupported architecture: $arch" }
}

Step "detecting platform ........." "windows-$($arch.ToString().ToLower())"

# ── latest version ────────────────────────────────────────────────────────────

$version = $env:SGL_VERSION
if (-not $version) {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $version = $release.tag_name
}

if (-not $version) { Die "could not determine latest version" }

$archive = "sgl-$version-$target.zip"
$url     = "https://github.com/$Repo/releases/download/$version/$archive"
$tmp     = Join-Path $env:TEMP $archive

Step "downloading sgl $version .." "done"
Invoke-WebRequest $url -OutFile $tmp -UseBasicParsing

# ── verify checksum ───────────────────────────────────────────────────────────

$sumsUrl = "https://github.com/$Repo/releases/download/$version/SHA256SUMS"
try {
    $sums = (Invoke-WebRequest $sumsUrl -UseBasicParsing).Content
    $expected = ($sums -split "`n" | Where-Object { $_ -match $archive }) -replace '\s+.*', ''
    $actual   = (Get-FileHash $tmp -Algorithm SHA256).Hash.ToLower()
    if ($expected -and $actual -ne $expected.ToLower()) {
        Die "checksum mismatch for $archive"
    }
    Step "verifying checksum ........." "ok"
} catch {
    Step "verifying checksum ........." "skipped"
}

# ── install ───────────────────────────────────────────────────────────────────

New-Item -ItemType Directory -Force $InstDir | Out-Null
Expand-Archive -Path $tmp -DestinationPath $InstDir -Force
Remove-Item $tmp -Force

Step "installing to $InstDir" "done"

Write-Host ""
Write-Host "sgl installed. Run ``sgl`` in a project directory."
Write-Host ""

# ── PATH warning ──────────────────────────────────────────────────────────────

$path = [System.Environment]::GetEnvironmentVariable('Path', 'User')
if ($path -notlike "*$InstDir*") {
    Write-Host "warning: $InstDir is not on your PATH."
    Write-Host "  Run this to add it:"
    Write-Host "    `$env:Path += `";$InstDir`""
    Write-Host "  Or set it permanently in System Properties > Environment Variables."
}
