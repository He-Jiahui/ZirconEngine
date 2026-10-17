@echo off
setlocal
"%ZR_CI_OUTPUT_ROOT%\zig-0.14.1\zig.exe" cc %* -target x86_64-linux-gnu
exit /b %ERRORLEVEL%
