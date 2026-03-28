# =============================================================================
#  install.ps1 -- Install mang.sh on Windows (PowerShell 5.1+ / PowerShell 7+)
#  https://github.com/paulfxyz/mang
#
#  Usage -- paste this into any PowerShell window:
#    iwr -useb https://mang.sh/install.ps1 | iex
#
#  Or save and run:
#    powershell -ExecutionPolicy Bypass -File install.ps1
#
#  Tested on:
#    Windows PowerShell 5.1 (Desktop edition, built into Windows)
#    PowerShell 7.x (pwsh, cross-platform)
#
#  IMPORTANT: WHY WE DO NOT USE $ErrorActionPreference = "Stop"
#  ─────────────────────────────────────────────────────────────
#  In PowerShell 5.1, any stderr output from a native executable (cargo.exe,
#  rustup-init.exe, git.exe) is captured as an ErrorRecord.  When
#  $ErrorActionPreference is "Stop", the first ErrorRecord terminates the
#  script — even when the native command actually succeeded (exit code 0).
#
#  cargo.exe writes progress messages ("Updating crates.io index", "Compiling
#  foo v1.0") to stderr even on a completely successful build.  Under Stop
#  mode this immediately kills the script with a TerminatingError before the
#  build even finishes.
#
#  Solution: keep $ErrorActionPreference = "Continue" (the default) throughout.
#  We check $LASTEXITCODE after every native command call.  This is the correct
#  and idiomatic way to handle native-executable errors in PowerShell.
# =============================================================================

# Do NOT use Set-StrictMode or $ErrorActionPreference = "Stop" here.
# See the long comment above.
$ErrorActionPreference = "Continue"

# -- Colour helpers -----------------------------------------------------------
# We use Write-Host directly (not echo / Write-Output) so output goes to the
# host and is not captured by the pipeline when running via | iex.
function Write-Banner { param($msg) Write-Host "  $msg" -ForegroundColor Cyan }
function Write-OK     { param($msg) Write-Host "  [ok] $msg" -ForegroundColor Green }
function Write-Info   { param($msg) Write-Host "  [..] $msg" -ForegroundColor Cyan }
function Write-Warn   { param($msg) Write-Host "  [!!] $msg" -ForegroundColor Yellow }
function Write-Fail   {
    param($msg)
    Write-Host "  [!!] $msg" -ForegroundColor Red
    Write-Host "" 
    Write-Host "  If this keeps happening, please open an issue:" -ForegroundColor DarkGray
    Write-Host "  https://github.com/paulfxyz/mang/issues" -ForegroundColor DarkGray
    Write-Host ""
    # Use exit with a non-zero code so callers can detect failure.
    # We cannot use "throw" here because we are running inside | iex which
    # swallows unhandled exceptions silently in PS5.
    exit 1
}

# -- Helper: run a native command and check $LASTEXITCODE --------------------
# This is the correct PS idiom for native commands.  We do NOT rely on
# $ErrorActionPreference to detect failures -- we check $LASTEXITCODE explicitly.
function Invoke-Native {
    param(
        [string]$Description,
        [scriptblock]$Command
    )
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Fail "$Description failed (exit code $LASTEXITCODE)"
    }
}

# -- Constants ----------------------------------------------------------------
$REPO_URL = "https://github.com/paulfxyz/mang"
$RAW_BASE = "https://raw.githubusercontent.com/paulfxyz/mang/main"
$ZIP_URL = "https://github.com/paulfxyz/mang/archive/refs/heads/main.zip"
$INSTALL_DIR = Join-Path $env:LOCALAPPDATA "mang\bin"
$TMP_DIR   = Join-Path $env:TEMP ("mang-install-" + [System.Guid]::NewGuid().ToString("N").Substring(0,8))

# Ensure tmp dir is cleaned up on exit (success or failure)
# We register a cleanup action that runs when the script exits
$script:TmpDirToClean = $TMP_DIR
Register-EngineEvent -SourceIdentifier ([System.Management.Automation.PsEngineEvent]::Exiting) -Action {
    if ($script:TmpDirToClean -and (Test-Path $script:TmpDirToClean)) {
        Remove-Item -Recurse -Force $script:TmpDirToClean -ErrorAction SilentlyContinue
    }
} -SupportEvent | Out-Null

# -- Banner -------------------------------------------------------------------
Write-Host ""
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host "  |        Installing  mang.sh            |" -ForegroundColor Cyan
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host ""

# -- Step 1: Detect existing install ------------------------------------------
$ExistingYo = Get-Command yo -ErrorAction SilentlyContinue
if ($ExistingYo) {
    Write-Warn "yo is already installed at $($ExistingYo.Source)"
    Write-Host "      Reinstalling will replace the binary. Your config is safe." -ForegroundColor DarkGray
    Write-Host ""
}

# -- Show target version ------------------------------------------------------
try {
    $CargoTomlContent = (Invoke-WebRequest -Uri "$RAW_BASE/Cargo.toml" -UseBasicParsing -TimeoutSec 15).Content
    $VersionMatch = [regex]::Match($CargoTomlContent, '^version\s*=\s*"([^"]+)"', [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if ($VersionMatch.Success) {
        $TargetVersion = $VersionMatch.Groups[1].Value
        Write-Host "      Target version: v$TargetVersion" -ForegroundColor DarkGray
        Write-Host ""
    }
} catch {
    # Non-fatal — version display is cosmetic
}

# -- Step 2: Ensure Rust / Cargo is available ---------------------------------
# Refresh PATH in case Rust was installed earlier in this session
$UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
$MachinePath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
$CargoBinPath = Join-Path $env:USERPROFILE ".cargo\bin"
$env:PATH = "$UserPath;$MachinePath;$CargoBinPath"

$CargoCmd = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $CargoCmd) {
    Write-Warn "Rust not found. Installing via rustup..."
    Write-Host "      Downloading rustup-init.exe..." -ForegroundColor DarkGray

    $RustupPath = Join-Path $env:TEMP "rustup-init-$([System.Guid]::NewGuid().ToString('N').Substring(0,6)).exe"
    try {
        # Invoke-WebRequest can be slow on PS5 — use .NET directly for reliability
        $WebClient = New-Object System.Net.WebClient
        $WebClient.DownloadFile("https://win.rustup.rs/x86_64", $RustupPath)
    } catch {
        Write-Fail "Could not download rustup-init.exe: $_`n  Check your internet connection."
    }

    Write-Host "      Installing Rust (this may take a few minutes)..." -ForegroundColor DarkGray

    # Run rustup-init synchronously.
    # -y = accept defaults, --quiet = minimal output
    # We use Start-Process with -Wait so we can check ExitCode cleanly.
    $RustupProc = Start-Process -FilePath $RustupPath `
        -ArgumentList "--quiet", "-y", "--default-toolchain", "stable", "--profile", "minimal" `
        -Wait -PassThru -NoNewWindow
    Remove-Item $RustupPath -Force -ErrorAction SilentlyContinue

    if ($RustupProc.ExitCode -ne 0) {
        Write-Fail "rustup-init.exe exited with code $($RustupProc.ExitCode). Install Rust manually from https://rustup.rs"
    }

    # Reload PATH after rustup install
    $UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $env:PATH = "$UserPath;$MachinePath;$CargoBinPath"

    $CargoCmd = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $CargoCmd) {
        Write-Fail "Cargo still not in PATH after Rust install.`n  Please close and reopen PowerShell, then re-run this script."
    }

    Write-OK "Rust installed."
} else {
    # Show Rust version.
    # IMPORTANT: capture output WITHOUT 2>&1 redirection on PS5.
    # 2>&1 in PS5 converts stderr lines to ErrorRecord objects which trigger
    # Stop behaviour even with $ErrorActionPreference = "Continue".
    $RustVersion = & rustc --version
    Write-OK "Rust: $RustVersion"
}

# -- Step 3: Download source as ZIP ------------------------------------------
# We use ZIP download instead of `git clone` so that Git is not required.
# This also avoids git.exe stderr output triggering PS5 issues.
Write-Info "Downloading mang.sh source..."
New-Item -ItemType Directory -Force -Path $TMP_DIR | Out-Null

$ZipDest = Join-Path $TMP_DIR "mang.zip"
try {
    # Use .NET WebClient for reliable binary download on PS5
    # Invoke-WebRequest -OutFile can fail silently on slow connections in PS5
    $WebClient = New-Object System.Net.WebClient
    $WebClient.DownloadFile($ZIP_URL, $ZipDest)
} catch {
    Write-Fail "Could not download source ZIP: $_"
}

if (-not (Test-Path $ZipDest) -or (Get-Item $ZipDest).Length -lt 1000) {
    Write-Fail "Downloaded ZIP appears empty or missing. Check your internet connection."
}

try {
    Expand-Archive -Path $ZipDest -DestinationPath $TMP_DIR -Force
} catch {
    Write-Fail "Could not extract ZIP: $_"
}
Remove-Item $ZipDest -Force -ErrorAction SilentlyContinue

# The ZIP extracts to a folder named mang-main
$SrcDir = Join-Path $TMP_DIR "mang-main"
if (-not (Test-Path $SrcDir)) {
    # Fallback: find any subdirectory (in case GitHub changes the ZIP structure)
    $Found = Get-ChildItem $TMP_DIR -Directory | Select-Object -First 1
    if ($Found) {
        $SrcDir = $Found.FullName
    } else {
        Write-Fail "Could not find extracted source directory in $TMP_DIR"
    }
}

# -- Step 4: Build the release binary -----------------------------------------
Write-Info "Building release binary (~2 min on first build, much faster after)..."
Write-Host "      You will see cargo's build output below. This is normal." -ForegroundColor DarkGray
Write-Host ""

Push-Location $SrcDir

# CRITICAL: Do NOT use 2>&1 or | Out-Null here.
# cargo.exe writes progress to stderr on a successful build.
# In PS5, capturing stderr with 2>&1 converts those lines to ErrorRecord
# objects and -- even with $ErrorActionPreference = "Continue" -- can cause
# the pipeline to misbehave.
# 
# We let cargo's stdout and stderr flow directly to the host (no redirection),
# then check $LASTEXITCODE afterward.  This is the only reliable method for
# running long-running native build tools in PS5.
& cargo build --release

$BuildExitCode = $LASTEXITCODE
Pop-Location

if ($BuildExitCode -ne 0) {
    Write-Fail "cargo build --release failed (exit code $BuildExitCode).`n  Run the build manually in $SrcDir to see full error output."
}

Write-Host ""
$BinaryPath = Join-Path $SrcDir "target\release\yo.exe"
if (-not (Test-Path $BinaryPath)) {
    Write-Fail "Build reported success but yo.exe not found at $BinaryPath. Please open an issue."
}
Write-OK "Build complete."

# -- Step 5: Install binary ---------------------------------------------------
# Use existing install location if reinstalling; otherwise use LOCALAPPDATA.
if ($ExistingYo -and (Test-Path $ExistingYo.Source)) {
    $TargetDir  = Split-Path $ExistingYo.Source
    $TargetPath = $ExistingYo.Source
} else {
    $TargetDir  = $INSTALL_DIR
    $TargetPath = Join-Path $INSTALL_DIR "yo.exe"
}

New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
Copy-Item -Path $BinaryPath -Destination $TargetPath -Force
Write-OK "Installed: $TargetPath"

# -- Step 6: Add install directory to user PATH --------------------------------
$CurrentUserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentUserPath -notlike "*$TargetDir*") {
    $NewUserPath = $CurrentUserPath.TrimEnd(";") + ";$TargetDir"
    [System.Environment]::SetEnvironmentVariable("PATH", $NewUserPath, "User")
    $env:PATH = "$env:PATH;$TargetDir"
    Write-OK "Added $TargetDir to your user PATH."
} else {
    Write-OK "PATH already contains $TargetDir"
}

# -- Step 7: PowerShell aliases in $PROFILE ------------------------------------
# Set-Alias does not persist across sessions on its own — it must be in $PROFILE.
Set-Alias -Name yo    -Value $TargetPath -Option AllScope -Scope Global -ErrorAction SilentlyContinue
Set-Alias -Name hi    -Value $TargetPath -Option AllScope -Scope Global -ErrorAction SilentlyContinue
Set-Alias -Name hello -Value $TargetPath -Option AllScope -Scope Global -ErrorAction SilentlyContinue

$AliasLines = @"

# mang aliases -- added by install.ps1
Set-Alias -Name yo    -Value "$TargetPath" -Option AllScope -Scope Global
Set-Alias -Name hi    -Value "$TargetPath" -Option AllScope -Scope Global
Set-Alias -Name hello -Value "$TargetPath" -Option AllScope -Scope Global
"@

if ($PROFILE) {
    $ProfileDir = Split-Path $PROFILE
    if ($ProfileDir -and -not (Test-Path $ProfileDir)) {
        New-Item -ItemType Directory -Force -Path $ProfileDir | Out-Null
    }
    if (-not (Test-Path $PROFILE)) {
        New-Item -ItemType File -Force -Path $PROFILE | Out-Null
    }

    $ProfileContent = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
    if (-not ($ProfileContent -like "*mang aliases*")) {
        Add-Content -Path $PROFILE -Value $AliasLines
        Write-OK "Aliases added to $PROFILE  (yo / hi / hello)"
    } else {
        Write-OK "Aliases already in $PROFILE"
    }
} else {
    Write-Warn "Could not locate PowerShell profile. Add manually:"
    Write-Host "      Set-Alias -Name yo -Value `"$TargetPath`"" -ForegroundColor DarkGray
}

# -- Step 8: Cleanup ----------------------------------------------------------
Remove-Item -Recurse -Force $TMP_DIR -ErrorAction SilentlyContinue

# -- Done ---------------------------------------------------------------------
Write-Host ""
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host "  |        mang.sh installed!  句芒 is ready           |" -ForegroundColor Cyan
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host ""
Write-Host "  You can type  " -NoNewline
Write-Host "yo" -ForegroundColor Cyan -NoNewline
Write-Host "  right now in this window."
Write-Host ""
Write-Host "  In new PowerShell windows: restart for PATH changes to take effect." -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Update:    iwr -useb $RAW_BASE/update.ps1 | iex" -ForegroundColor DarkGray
Write-Host "  Uninstall: iwr -useb $RAW_BASE/uninstall.ps1 | iex" -ForegroundColor DarkGray
Write-Host ""
