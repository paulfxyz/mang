# =============================================================================
#  uninstall.ps1 -- Remove mang.sh from Windows (PowerShell)
#  https://github.com/paulfxyz/mang
#
#  Usage:
#    iwr -useb https://mang.sh/uninstall.ps1 | iex
#
#  What it removes:
#    - The yo.exe binary (wherever installed)
#    - mang.sh directory from LOCALAPPDATA (if default install location)
#    - The install directory from $env:PATH (user PATH entry)
#    - yo / hi / hello aliases from $PROFILE
#    - Config directory (%APPDATA%\mang) -- ASKS before deleting
#
#  What it keeps:
#    - Rust / rustup (you may use it for other projects)
# =============================================================================

#Requires -Version 5.1

Set-StrictMode -Version Latest
$ErrorActionPreference = "SilentlyContinue"

function Log-OK    { param($msg) Write-Host "  [ok] $msg"  -ForegroundColor Green }
function Log-Warn  { param($msg) Write-Host "  [!!] $msg"  -ForegroundColor Yellow }
function Log-Skip  { param($msg) Write-Host "  [--] $msg"  -ForegroundColor DarkGray }

function Ask-YesNo {
    param([string]$Question, [bool]$DefaultYes = $true)
    $hint = if ($DefaultYes) { "[Y/N]" } else { "[y/N]" }
    Write-Host "  [??] $Question $hint " -ForegroundColor Yellow -NoNewline
    $reply = Read-Host
    if ($reply -match '^[Yy]$') { return $true }
    if ($reply -match '^[Nn]$') { return $false }
    return $DefaultYes
}

Write-Host ""
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host "  |       Uninstalling  mang.sh           |" -ForegroundColor Cyan
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host ""
Write-Host "  This will remove mang.sh from your system." -ForegroundColor DarkGray
Write-Host "  Your API key will be kept unless you say otherwise." -ForegroundColor DarkGray
Write-Host ""

# -- Confirm ------------------------------------------------------------------
if (-not (Ask-YesNo "Are you sure you want to uninstall mang.sh?" $true)) {
    Write-Host ""
    Write-Host "  Cancelled. Nothing was changed." -ForegroundColor DarkGray
    Write-Host ""
    exit 0
}
Write-Host ""

# -- Step 1: Find and remove binary -------------------------------------------
$YoBin = Get-Command yo -ErrorAction SilentlyContinue
$YoBinPath = $null

if ($YoBin) {
    $YoBinPath = $YoBin.Source
} else {
    $DefaultPath = Join-Path $env:LOCALAPPDATA "mang\bin\yo.exe"
    if (Test-Path $DefaultPath) { $YoBinPath = $DefaultPath }
}

if ($YoBinPath -and (Test-Path $YoBinPath)) {
    $InstallDir = Split-Path $YoBinPath
    Remove-Item $YoBinPath -Force -ErrorAction SilentlyContinue
    Log-OK "Removed binary: $YoBinPath"

    # Remove the empty install directory if it was our default location
    $DefaultInstallDir = Join-Path $env:LOCALAPPDATA "mang\bin"
    if ($InstallDir -eq $DefaultInstallDir) {
        Remove-Item (Join-Path $env:LOCALAPPDATA "mang.sh") -Recurse -Force -ErrorAction SilentlyContinue
        Log-OK "Removed install directory: $(Join-Path $env:LOCALAPPDATA 'mang.sh')"
    }

    # Remove from user PATH
    $UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    $NewPath = ($UserPath -split ';' | Where-Object { $_ -ne $InstallDir -and $_ -ne "" }) -join ';'
    if ($NewPath -ne $UserPath) {
        [System.Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
        Log-OK "Removed $InstallDir from user PATH."
    }
} else {
    Log-Skip "Binary not found in PATH or default location — may already be removed."
}

# -- Step 2: Remove config (ask first) ----------------------------------------
Write-Host ""
$ConfigDir = Join-Path $env:APPDATA "mang.sh"
if (Test-Path $ConfigDir) {
    Log-Warn "Config found: $ConfigDir"
    Write-Host "      Contains your OpenRouter API key and preferences." -ForegroundColor DarkGray
    Write-Host "      Keeping it means reinstalling picks up your settings automatically." -ForegroundColor DarkGray
    Write-Host ""
    if (Ask-YesNo "Delete config? (you'll need to re-enter your API key if you reinstall)" $false) {
        Remove-Item $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue
        Log-OK "Removed config: $ConfigDir"
    } else {
        Log-Skip "Config kept at: $ConfigDir"
    }
} else {
    Log-Skip "Config directory not found."
}

# -- Step 3: Remove aliases from $PROFILE ------------------------------------
Write-Host ""
if ($PROFILE -and (Test-Path $PROFILE)) {
    $Content = Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue
    if ($Content -like "*mang aliases*") {
        # Remove the alias block — everything between the comment and end of aliases
        $Cleaned = $Content -replace '(?ms)\r?\n# mang aliases.*?Set-Alias -Name hello.*?\r?\n', "`n"
        Set-Content $PROFILE $Cleaned.TrimEnd() -NoNewline
        Log-OK "Removed mang aliases from $PROFILE"
    } else {
        Log-Skip "No mang aliases found in $PROFILE"
    }
} else {
    Log-Skip "No PowerShell profile found."
}

# -- Done ---------------------------------------------------------------------
Write-Host ""
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host "  |          Uninstall complete!            |" -ForegroundColor Cyan
Write-Host "  +==========================================+" -ForegroundColor Cyan
Write-Host ""
Write-Host "  mang.sh has been removed. 句芒"
Write-Host "  Rust itself was NOT removed." -ForegroundColor DarkGray
Write-Host "  To remove Rust: rustup self uninstall" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  To reinstall:" -ForegroundColor DarkGray
Write-Host "  iwr -useb https://mang.sh/install.ps1 | iex" -ForegroundColor DarkGray
Write-Host ""
