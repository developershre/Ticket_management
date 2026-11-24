@echo off
setlocal
echo ==========================================================
echo   BPSL IT Support - Multi-Distro Linux Packaging Tool
echo ==========================================================
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build-linux.ps1" %*
