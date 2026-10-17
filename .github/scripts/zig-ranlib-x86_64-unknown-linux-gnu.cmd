@echo off
setlocal
"E:\cargo-targets\zircon-local\ci\miri-scene-ecs\zig-0.14.1\zig.exe" ranlib %*
exit /b %ERRORLEVEL%
