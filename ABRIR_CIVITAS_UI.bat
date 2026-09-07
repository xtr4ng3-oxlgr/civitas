@echo off
title CIVITAS
color 0C
chcp 65001 >nul
cd /d "%~dp0"
if not exist bridge\civitas_core.exe (
    echo No se encontro bridge\civitas_core.exe
    echo Primero ejecuta build_windows\BUILD_CORE_RUST.bat
    pause
    exit /b
)
py -3 ui-python\civitas_ui.py
pause
