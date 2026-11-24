<#
.SYNOPSIS
Builds and packages BPSL IT Support for Linux distributions from Windows.

.DESCRIPTION
Executes the comprehensive Linux packaging suite (build-linux.sh) via Git Bash,
WSL (Windows Subsystem for Linux), or a Docker container.

Supported Distributions & Formats:
  - Debian, Ubuntu, Linux Mint, Pop!_OS (.deb)
  - Fedora, RHEL, CentOS, Rocky Linux, openSUSE (.rpm)
  - Arch Linux, Manjaro, EndeavourOS (.pkg.tar.zst + PKGBUILD)
  - Universal Standalone AppImage (.AppImage)
  - Flatpak Manifest (com.bpsl.itsupport.yml)
  - Universal Portable Archive (.tar.gz with self-contained installer)
#>

param(
    [switch]$All,
    [switch]$Deb,
    [switch]$Rpm,
    [switch]$Arch,
    [switch]$AppImage,
    [switch]$Flatpak,
    [switch]$Tarball,
    [switch]$SkipBuild,
    [switch]$Clean,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Host @"
BPSL IT Support - Windows Linux Packaging Helper

Usage:
  .\build-linux.ps1 [options]
  .\build-linux.bat [options]

Options:
  -All          Build packages for all Linux distributions (Default)
  -Deb          Build Debian / Ubuntu / Linux Mint package (.deb)
  -Rpm          Build Fedora / RHEL / CentOS package (.rpm)
  -Arch         Build Arch Linux package (.pkg.tar.zst) and PKGBUILD
  -AppImage     Build Universal standalone AppImage (.AppImage)
  -Flatpak      Generate Flatpak manifest and build scripts
  -Tarball      Build Universal Linux portable archive (.tar.gz)
  -SkipBuild    Skip compilation if release binary already exists
  -Clean        Clean previous dist/ outputs before building
  -Help         Show this help message

Examples:
  .\build-linux.ps1 -All
  .\build-linux.ps1 -Deb
  .\build-linux.ps1 -Rpm -SkipBuild
"@
    exit 0
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  BPSL IT Support - Multi-Distro Linux Packaging Tool     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Locate Bash / Linux Execution Environment
$bashExe = $null

$gitBashPaths = @(
    "C:\Program Files\Git\bin\bash.exe",
    "C:\Program Files\Git\usr\bin\bash.exe",
    "$env:LOCALAPPDATA\Programs\Git\bin\bash.exe"
)

foreach ($path in $gitBashPaths) {
    if (Test-Path $path) {
        $bashExe = $path
        break
    }
}

$hasWsl = $false
try {
    $wslCheck = Get-Command "wsl.exe" -ErrorAction SilentlyContinue
    if ($wslCheck) {
        $wslOut = & wsl.exe -l -q 2>$null
        if ($LASTEXITCODE -eq 0 -and $wslOut.Count -gt 0) {
            $hasWsl = $true
        }
    }
} catch {
    $hasWsl = $false
}

# 2. Build Argument Array for build-linux.sh
$argsList = @()

if ($Clean) { $argsList += "--clean" }
if ($SkipBuild) { $argsList += "--skip-build" }

$specificTarget = $false
if ($Deb) { $argsList += "--deb"; $specificTarget = $true }
if ($Rpm) { $argsList += "--rpm"; $specificTarget = $true }
if ($Arch) { $argsList += "--arch"; $specificTarget = $true }
if ($AppImage) { $argsList += "--appimage"; $specificTarget = $true }
if ($Flatpak) { $argsList += "--flatpak"; $specificTarget = $true }
if ($Tarball) { $argsList += "--tarball"; $specificTarget = $true }

if (-not $specificTarget -or $All) {
    $argsList += "--all"
}

$scriptPath = (Join-Path $PSScriptRoot "build-linux.sh").Replace("\", "/")

# 3. Execution via Git Bash or WSL
if ($bashExe) {
    Write-Host "[Env] Using Bash environment: $bashExe" -ForegroundColor DarkGray
    Write-Host "[Exec] Running: bash build-linux.sh $($argsList -join ' ')...`n" -ForegroundColor Yellow
    & $bashExe "$scriptPath" @argsList
    if ($LASTEXITCODE -ne 0) {
        Write-Error "build-linux.sh exited with error code $LASTEXITCODE."
    }
} elseif ($hasWsl) {
    Write-Host "[Env] Using Windows Subsystem for Linux (WSL)" -ForegroundColor DarkGray
    $wslScript = "/mnt/" + $PSScriptRoot.Substring(0,1).ToLower() + $PSScriptRoot.Substring(2).Replace("\", "/") + "/build-linux.sh"
    Write-Host "[Exec] Running in WSL: bash $wslScript $($argsList -join ' ')...`n" -ForegroundColor Yellow
    & wsl.exe bash "$wslScript" @argsList
    if ($LASTEXITCODE -ne 0) {
        Write-Error "WSL build-linux.sh exited with error code $LASTEXITCODE."
    }
} else {
    Write-Error "No Linux bash environment detected. Please install Git for Windows (includes Git Bash) or enable WSL."
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "  Linux packaging process completed successfully!        " -ForegroundColor Green
Write-Host "  Outputs available in: $(Join-Path $PSScriptRoot 'dist') " -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Green
