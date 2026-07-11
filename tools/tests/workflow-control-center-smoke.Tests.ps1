[CmdletBinding()]
param(
    [switch]$ReadOnlyConsole,
    [switch]$ControlledActions
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if ($ReadOnlyConsole -eq $ControlledActions) {
    throw 'Select exactly one acceptance gate: -ReadOnlyConsole or -ControlledActions.'
}

Push-Location $repoRoot
try {
    $gate = if ($ControlledActions) { '--controlled-actions' } else { '--read-only-console' }
    python -m tools.tests.workflow_control_center_smoke --repo-root $repoRoot $gate
    if ($LASTEXITCODE -ne 0) {
        throw "Workflow control-center smoke failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}
