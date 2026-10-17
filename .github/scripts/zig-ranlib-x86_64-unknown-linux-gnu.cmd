@echo off
setlocal
"%ZR_CI_OUTPUT_ROOT%\zig-0.14.1\zig.exe" ranlib %*
exit /b %ERRORLEVEL%
