@echo off
title CIVITAS - Build Release Package
color 0C
cd /d "%~dp0\.."
call build_windows\BUILD_CORE_RUST.bat
rmdir /s /q RELEASE 2>nul
mkdir RELEASE
xcopy /E /I /Y bridge RELEASE\bridge
xcopy /E /I /Y ui-python RELEASE\ui-python
xcopy /E /I /Y docs RELEASE\docs
copy /Y README.md RELEASE\README.md
copy /Y SECURITY.md RELEASE\SECURITY.md
copy /Y CONTRIBUTING.md RELEASE\CONTRIBUTING.md
copy /Y LICENSE RELEASE\LICENSE
copy /Y ABRIR_CIVITAS_UI.bat RELEASE\ABRIR_CIVITAS_UI.bat
copy /Y INSTALAR_UI_PYTHON.bat RELEASE\INSTALAR_UI_PYTHON.bat
powershell -NoProfile -Command "Compress-Archive -Path RELEASE\* -DestinationPath civitas-release.zip -Force"
pause
