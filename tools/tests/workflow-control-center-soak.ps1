[CmdletBinding()]
param(
    [ValidateRange(0.001, 168.0)]
    [double]$Hours = 24.0,
    [ValidateRange(0.1, 3600.0)]
    [double]$IntervalSeconds = 60.0,
    [string]$OutputPath,
    [string]$WorkRoot
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$stamp = [DateTime]::Now.ToString('yyyyMMdd-HHmmss')
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path ([System.IO.Path]::GetTempPath()) "zircon-control-soak-$stamp.json"
}
$localState = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'Zircon Session Coordinator\soak-runs'
if ([string]::IsNullOrWhiteSpace($WorkRoot)) {
    $WorkRoot = Join-Path $localState $stamp
}
$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$resolvedWorkRoot = [System.IO.Path]::GetFullPath($WorkRoot)
if ($resolvedOutput.StartsWith($repoRoot + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Soak samples must be written outside Git; publish only a sanitized summary after acceptance.'
}
if ($resolvedWorkRoot.StartsWith($repoRoot + '\', [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Soak workspace must remain outside Git.'
}

Push-Location $repoRoot
try {
    python -m tools.session_coordinator.soak `
        --hours $Hours `
        --interval-seconds $IntervalSeconds `
        --output $resolvedOutput `
        --work-root $resolvedWorkRoot
    if ($LASTEXITCODE -ne 0) {
        throw "Workflow control-center soak failed; inspect $resolvedOutput"
    }
    Write-Host "workflow control-center soak passed: $resolvedOutput"
}
finally {
    Pop-Location
}
