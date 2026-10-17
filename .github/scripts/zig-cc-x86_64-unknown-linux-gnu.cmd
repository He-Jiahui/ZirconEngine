@echo off
setlocal
"E:\cargo-targets\zircon-local\ci\miri-scene-ecs\zig-0.14.1\zig.exe" cc %* -target x86_64-linux-gnu
exit /b %ERRORLEVEL%
