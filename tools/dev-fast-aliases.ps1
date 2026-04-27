Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$runner = Join-Path $scriptDir "dev-fast-build.ps1"

function zr-client-check { & $runner -Profile client -Action check @args }
function zr-client-build { & $runner -Profile client -Action build @args }
function zr-client-test { & $runner -Profile client -Action test @args }
function zr-client-run { & $runner -Profile client -Action run @args }

function zr-server-check { & $runner -Profile server -Action check @args }
function zr-server-build { & $runner -Profile server -Action build @args }
function zr-server-test { & $runner -Profile server -Action test @args }

function zr-editor-check { & $runner -Profile editor -Action check @args }
function zr-editor-build { & $runner -Profile editor -Action build @args }
function zr-editor-test { & $runner -Profile editor -Action test @args }
function zr-editor-run { & $runner -Profile editor -Action run @args }

function zr-sccache-status {
    if (Get-Command sccache -ErrorAction SilentlyContinue) {
        sccache --show-stats
    } else {
        Write-Host "sccache is not installed. Run: .\scripts\dev-fast-build.ps1 -InstallSccache" -ForegroundColor Yellow
    }
}

Write-Host "Loaded aliases: zr-client-*, zr-server-*, zr-editor-*, zr-sccache-status" -ForegroundColor Green
