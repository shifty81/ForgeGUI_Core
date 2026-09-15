@echo off
setlocal
set ROOT=%~dp0..
echo.
echo ForgeGUI Lab Suite
echo ==================
echo 1. Contracts smoke
echo 2. Micro Lab
echo 3. Universal App Lab
echo 4. Heavy ForgeGUI Lab
choice /c 1234 /n /m "Select lab: "
if errorlevel 4 cargo run -p forge_gui_lab & goto :eof
if errorlevel 3 cargo run -p forge_gui_app_lab & goto :eof
if errorlevel 2 cargo run -p forge_gui_micro_lab & goto :eof
if errorlevel 1 cargo run -p forge_gui_contracts_smoke
