[CmdletBinding()]
param(
    [ValidateSet("Quick", "Release")]
    [string]$Profile = "Quick",

    [ValidateSet("H4", "Full")]
    [string]$Suite = "H4",

    [switch]$SkipWeb
)

$ErrorActionPreference = "Stop"
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$logRoot = Join-Path $env:LOCALAPPDATA "ZirconEngine\SessionCoordinator\validation"
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$logPath = Join-Path $logRoot "control-$($Suite.ToLowerInvariant())-$($Profile.ToLowerInvariant())-$timestamp.log"
New-Item -ItemType Directory -Force -Path $logRoot | Out-Null

$previousProfile = $env:ZIRCON_CONTROL_LOAD_PROFILE
$env:ZIRCON_CONTROL_LOAD_PROFILE = $Profile.ToLowerInvariant()
$exitCode = 0

Start-Transcript -LiteralPath $logPath -Force | Out-Null
try {
    Set-Location -LiteralPath $repoRoot
    Write-Host "Validation suite: $Suite"
    Write-Host "Load profile: $Profile"
    Write-Host "Log: $logPath"

    if ($Suite -eq "H4") {
        & python -m unittest -v `
            tools.session_coordinator.tests.test_control_snapshot `
            tools.session_coordinator.tests.test_control_events `
            tools.session_coordinator.tests.test_control_load
    }
    else {
        & python -m unittest discover -v -s tools/session_coordinator/tests
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Coordinator tests failed with exit code $LASTEXITCODE"
    }

    if (-not $SkipWeb) {
        & npm --prefix tools/session_coordinator/web run check
        if ($LASTEXITCODE -ne 0) {
            throw "Control Center web checks failed with exit code $LASTEXITCODE"
        }
    }
}
catch {
    $exitCode = 1
    Write-Error $_
}
finally {
    if ($null -eq $previousProfile) {
        Remove-Item Env:ZIRCON_CONTROL_LOAD_PROFILE -ErrorAction SilentlyContinue
    }
    else {
        $env:ZIRCON_CONTROL_LOAD_PROFILE = $previousProfile
    }
    Stop-Transcript | Out-Null
    Write-Host "Validation log retained at $logPath"
}

exit $exitCode
