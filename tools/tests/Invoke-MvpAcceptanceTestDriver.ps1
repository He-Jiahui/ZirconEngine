[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArguments
)

$ErrorActionPreference = 'Stop'
$WarningPreference = 'SilentlyContinue'

$driver = Join-Path $PSScriptRoot '..\mvp\Invoke-MvpAcceptance.ps1'
$treeManifestModule = Join-Path $PSScriptRoot '..\mvp\MvpAcceptanceStagingTreeManifest.psm1'
Import-Module $treeManifestModule -Force -ErrorAction Stop
$stagingRootIndex = [Array]::IndexOf($RemainingArguments, '-StagingRoot')
if ($stagingRootIndex -ge 0 -and $stagingRootIndex + 1 -lt $RemainingArguments.Count) {
    # Test fixtures deliberately mutate isolated staging evidence between negative cases.
    # Republish the producer handoff immediately before invoking the production consumer.
    $stagingRoot = [string]$RemainingArguments[$stagingRootIndex + 1]
    $stagingRootItem = Get-Item -LiteralPath $stagingRoot -Force -ErrorAction Stop
    if (-not [bool]($stagingRootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        Write-MvpAcceptanceStagingTreeManifest -StagingRoot $stagingRoot | Out-Null
    }
}
$output = @(& pwsh -NoProfile -File $driver @RemainingArguments 2>&1)
$exitCode = $LASTEXITCODE
if ($exitCode -ne 0) {
    $detail = @($output | ForEach-Object { $_.ToString() }) -join [Environment]::NewLine
    throw "MVP acceptance driver failed with exit code $exitCode.$([Environment]::NewLine)$detail"
}

$output
