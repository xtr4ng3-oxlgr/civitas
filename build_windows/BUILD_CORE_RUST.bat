@echo off
title CIVITAS - Build Rust Core
color 0C
cd /d "%~dp0\.."

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Cargo/Rust no encontrado.
    echo Instala Rust desde rustup.rs y vuelve a intentar.
    pause
    exit /b
)

cd core-rust
cargo build --release
if %errorlevel% neq 0 (
    echo Build fallido.
    pause
    exit /b
)

cd ..
if not exist bridge mkdir bridge
copy /Y core-rust\target\release\civitas_core.exe bridge\civitas_core.exe

echo Core listo: bridge\civitas_core.exe
pause
