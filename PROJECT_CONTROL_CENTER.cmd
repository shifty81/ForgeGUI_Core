@echo off
setlocal EnableExtensions
cd /d "%~dp0"
set "PCC_SCRIPT=%~dp0tools\pcc\ForgeGuiControl.ps1"
if not exist "%PCC_SCRIPT%" (
  echo [FORGEGUI PCC] FATAL: missing %PCC_SCRIPT%
  if "%~1"=="" pause
  exit /b 2
)
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%PCC_SCRIPT%" %*
set "PCC_EXIT=%ERRORLEVEL%"
if not "%PCC_EXIT%"=="0" if "%~1"=="" pause
exit /b %PCC_EXIT%
