# =============================================================================
#  update.ps1 -- Update mang.sh to the latest version (Windows / PowerShell)
#  https://github.com/paulfxyz/mang
#
#  Usage:
#    iwr -useb https://mang.sh/update.ps1 | iex
#
#  See install.ps1 for the detailed explanation of why we do NOT use
#  $ErrorActionPreference = "Stop" and why we check $LASTEXITCODE instead.
# =============================================================================

$ErrorActionPreference = "Continue"

function Write-OK   { param($msg) Write-Host "  [ok] $msg" -ForegroundColor Green }
function Write-Info { param($msg) Write-Host "  [..] $msg" -ForegroundColor Cyan }
function Write-Warn { param($msg) Write-Host "  [!!] $msg" -ForegroundColor Yellow }
function Write-Fail {
    param($msg)
    Write-Host "  [!!] $msg" -ForegroundColor Red
    Write-Host "  https://github.com/paulfxyz/mang/issues" -ForegroundColor DarkGray
    Write-Host ""
    exit 1
}

$RAW_BASE = "https://raw.githubusercontent.com/paulfxyz/mang/main"
$ZIP_URL = "https://github.com/paulfxyz/mang/archive/refs/heads/main.zip"
$TMP_DIR  = Join-Path $env:TEMP ("mang-update-" + [System.Guid]::NewGuid().ToString("N").Substring(0,8))

Write-Host ""
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host "  |          Updating  mang.sh            |" -ForegroundColor Cyan
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host ""

# -- Find existing binary -----------------------------------------------------
$env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","User") + ";" +
            [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" +
            (Join-Path $env:USERPROFILE ".cargo\bin")

$YoBin = Get-Command yo -ErrorAction SilentlyContinue
$YoBinPath = $null
if ($YoBin) {
    $YoBinPath = $YoBin.Source
} else {
    $Default = Join-Path $env:LOCALAPPDATA "mang\bin\yo.exe"
    if (Test-Path $Default) { $YoBinPath = $Default }
}

if (-not $YoBinPath) {
    Write-Warn "mang.sh does not appear to be installed."
    Write-Host "      Install first: iwr -useb $RAW_BASE/install.ps1 | iex" -ForegroundColor DarkGray
    exit 1
}
Write-OK "Found: $YoBinPath"

# -- Read installed version from binary bytes ---------------------------------
$InstalledVersion = "unknown"
try {
    $bytes = [System.IO.File]::ReadAllBytes($YoBinPath)
    # Read as ASCII, replace non-printable chars with space, then regex for version
    $text = [System.Text.Encoding]::ASCII.GetString($bytes) -replace '[^\x20-\x7E]', ' '
    $m = [regex]::Match($text, 'v(\d+\.\d+\.\d+)')
    if ($m.Success) { $InstalledVersion = $m.Value }
} catch { }
Write-Host "      Installed: $InstalledVersion" -ForegroundColor DarkGray

# -- Fetch latest version -----------------------------------------------------
Write-Info "Checking latest version on GitHub..."
try {
    $CargoToml = (Invoke-WebRequest -Uri "$RAW_BASE/Cargo.toml" -UseBasicParsing -TimeoutSec 15).Content
    $m = [regex]::Match($CargoToml, '^version\s*=\s*"([^"]+)"', [System.Text.RegularExpressions.RegexOptions]::Multiline)
    $LatestVersion = if ($m.Success) { $m.Groups[1].Value } else { "unknown" }
} catch {
    Write-Fail "Could not reach GitHub. Check your connection and try again."
}
Write-Host "      Latest:    v$LatestVersion" -ForegroundColor DarkGray

# -- Early exit if already current --------------------------------------------
if ($InstalledVersion -eq "v$LatestVersion") {
    Write-Host ""
    Write-OK "Already up to date ($InstalledVersion). Nothing to do."
    Write-Host ""
    exit 0
}

Write-Host ""
Write-Info "Updating $InstalledVersion --> v$LatestVersion..."
Write-Host ""

# -- Ensure Rust ---------------------------------------------------------------
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Fail "Rust/cargo not found. Run install.ps1 to reinstall."
}

# -- Download and extract source ZIP ------------------------------------------
New-Item -ItemType Directory -Force -Path $TMP_DIR | Out-Null
Write-Info "Downloading latest source..."

$ZipDest = Join-Path $TMP_DIR "mang.zip"
try {
    $WebClient = New-Object System.Net.WebClient
    $WebClient.DownloadFile($ZIP_URL, $ZipDest)
} catch {
    Write-Fail "Download failed: $_"
}

Expand-Archive -Path $ZipDest -DestinationPath $TMP_DIR -Force
Remove-Item $ZipDest -Force -ErrorAction SilentlyContinue

$SrcDir = Join-Path $TMP_DIR "mang-main"
if (-not (Test-Path $SrcDir)) {
    $SrcDir = (Get-ChildItem $TMP_DIR -Directory | Select-Object -First 1).FullName
}

# -- Build (same pattern as install.ps1: no 2>&1, check $LASTEXITCODE) --------
Write-Info "Building release binary..."
Write-Host "      cargo output will appear below. This is normal." -ForegroundColor DarkGray
Write-Host ""

Push-Location $SrcDir
& cargo build --release
$BuildExit = $LASTEXITCODE
Pop-Location

if ($BuildExit -ne 0) {
    Write-Fail "Build failed (exit $BuildExit). See output above for details."
}

$NewBin = Join-Path $SrcDir "target\release\yo.exe"
if (-not (Test-Path $NewBin)) {
    Write-Fail "Build succeeded but yo.exe not found."
}
Write-OK "Build complete."

# -- Replace binary in-place --------------------------------------------------
Copy-Item -Path $NewBin -Destination $YoBinPath -Force
Write-OK "Updated: $YoBinPath"

# -- Cleanup ------------------------------------------------------------------
Remove-Item -Recurse -Force $TMP_DIR -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host "  |           Update complete!              |" -ForegroundColor Cyan
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host ""
Write-Host "  mang.sh v$LatestVersion is ready." -ForegroundColor Green
Write-Host "  Your config was not changed." -ForegroundColor DarkGray
Write-Host ""
