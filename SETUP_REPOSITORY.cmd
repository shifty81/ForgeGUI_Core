@echo off
setlocal EnableExtensions
cd /d "%~dp0"
echo ============================================================================
echo  ForgeGUI_Core repository setup
echo ============================================================================
echo.
echo Intended remote:
echo   https://github.com/shifty81/ForgeGUI_Core.git
echo.
echo This initializes/normalizes the local Git repository and remote through the
echo project-owned PCC. Source commit/push remains blocked until Full Gate is GREEN.
echo.
call PROJECT_CONTROL_CENTER.cmd -Operation git.setup
set "RC=%ERRORLEVEL%"
echo.
if not "%RC%"=="0" (
  echo [FAIL] Repository setup returned exit code %RC%.
  pause
  exit /b %RC%
)
echo [PASS] Local Git setup completed.
echo Next: run PROJECT_CONTROL_CENTER.cmd and select 1 - FULL QUALITY GATE.
pause
exit /b 0
