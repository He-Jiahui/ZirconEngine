[CmdletBinding()]
param(
    [switch]$ReadOnlyConsole
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if (-not $ReadOnlyConsole) {
    throw 'Use -ReadOnlyConsole to run the M2 observer-surface acceptance gate.'
}

Push-Location $repoRoot
try {
    python -m tools.tests.workflow_control_center_smoke --repo-root $repoRoot --read-only-console
    if ($LASTEXITCODE -ne 0) {
        throw "Read-only console smoke failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}
