@echo off
setlocal
echo ==========================================================
echo   BPSL IT Support - Windows MSI Installer Release Build
echo ==========================================================
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build-msi.ps1" %*

