@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
set "TARGET_SCRIPT=dev-fast-server-check-debug.cmd"
for %%A in (%*) do (
  if /I "%%~A"=="-Release" set "TARGET_SCRIPT=dev-fast-server-check-release.cmd"
)

call "%SCRIPT_DIR%%TARGET_SCRIPT%"
set "EXIT_CODE=%ERRORLEVEL%"

endlocal & exit /b %EXIT_CODE%
