<#
.SYNOPSIS
Builds the production release and creates the .msi installer for BPSL IT Support.

.DESCRIPTION
1. Generates a fresh, pristine empty SQLite database (empty.db) with all schema tables and 0 rows.
2. Ensures WiX toolset is accessible on PATH.
3. Compiles and packages the release into a Windows .msi installer using dx bundle.
4. Outputs the ready-to-distribute .msi installer in target\dx\management\bundle\windows\msi\.
#>

param(
    [switch]$DebugBuild
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  BPSL IT Support - Windows MSI Installer Release Build   " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Configure local WiX Toolset on PATH if present
$wixTools = Join-Path $PSScriptRoot "tools\wix"
if (Test-Path (Join-Path $wixTools "candle.exe")) {
    $env:PATH = "$wixTools;$env:PATH"
    Write-Host "[Env] Using local WiX toolset from tools\wix" -ForegroundColor DarkGray
}

# 2. Generate a 100% clean empty database with all tables and 0 rows
Write-Host "`n[1/3] Generating clean empty SQLite database (empty.db) with all tables..." -ForegroundColor Yellow
node (Join-Path $PSScriptRoot "scripts\generate_empty_db.js") "empty.db"
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to generate empty.db."
}

# Also ensure management.db in repo root is cleaned so development starts fresh
node (Join-Path $PSScriptRoot "scripts\generate_empty_db.js") "management.db"

# 3. Build and package .msi installer
$mode = if ($DebugBuild) { "" } else { "--release" }
$modeLabel = if ($DebugBuild) { "debug" } else { "release" }
Write-Host "`n[2/3] Compiling and packaging .msi installer (mode: $modeLabel)..." -ForegroundColor Yellow

if ($mode) {
    dx bundle --package-types msi --platform desktop $mode
} else {
    dx bundle --package-types msi --platform desktop
}

if ($LASTEXITCODE -ne 0) {
    Write-Error "dx bundle failed to generate MSI installer."
}

# 4. Locate and verify resulting installer
Write-Host "`n[3/3] Locating created MSI installer..." -ForegroundColor Green
$msiFolder = "target\dx\management\bundle\windows\msi"
$msiFiles = Get-ChildItem -Path $msiFolder -Filter "*.msi" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending

if ($msiFiles.Count -gt 0) {
    $latestMsi = $msiFiles[0]
    Write-Host "`n==========================================================" -ForegroundColor Green
    Write-Host "  SUCCESS! Windows MSI Installer is ready:" -ForegroundColor Green
    Write-Host "  Path: $($latestMsi.FullName)" -ForegroundColor White
    Write-Host "  Size: $([math]::Round($latestMsi.Length / 1MB, 2)) MB" -ForegroundColor White
    Write-Host "==========================================================" -ForegroundColor Green
} else {
    Write-Warning "Bundle completed, but no .msi was found in $msiFolder."
}
