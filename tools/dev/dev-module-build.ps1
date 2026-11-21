[CmdletBinding()]
param(
    [ValidatePattern('^[A-Za-z0-9_][A-Za-z0-9_.-]*$')][string]$Package = 'zircon_runtime',
    [ValidateSet('check', 'build', 'test')][string]$Action = 'check',
    [string[]]$Features = @(),
    [switch]$LibTests,
    [string]$TestTarget,
    [string]$TestFilter,
    [switch]$NoRun,
    [switch]$Release,
    [ValidateRange(1,8)][int]$Jobs = 1,
    [ValidateRange(1,8)][int]$TestThreads = 1,
    [switch]$Offline,
    [string]$PythonInterpreter,
    [string]$TargetDir,
    [string]$BuildRoot,
    [string]$ManifestPath,
    [string]$JenkinsIdentityFile,
    [switch]$DryRun,
    [switch]$QueryOnly
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
try {
    if ($PSBoundParameters.ContainsKey('JenkinsIdentityFile')) {
        throw 'Jenkins workflow is retired; omit -JenkinsIdentityFile for Agent-owned execution.'
    }
    if ($Action -ne 'test' -and ($LibTests -or $TestTarget -or $TestFilter -or $NoRun -or
            $PSBoundParameters.ContainsKey('TestThreads'))) { throw 'Test selectors require -Action test.' }
    if ($Action -eq 'test' -and (-not ($LibTests -or $TestTarget) -or ($LibTests -and $TestTarget))) {
        throw 'Select exactly one of -LibTests or -TestTarget for module validation.'
    }
    if ($NoRun -and $PSBoundParameters.ContainsKey('TestThreads')) { throw '-NoRun cannot use test harness options.' }
    if ($Package -eq 'zircon_runtime' -and -not $PSBoundParameters.ContainsKey('Features')) {
        $Features = @('core-min')
    }
    foreach ($feature in $Features) {
        if ($feature -notmatch '^[A-Za-z0-9_][A-Za-z0-9_./+-]*$') { throw 'Select explicit Cargo feature names.' }
    }
    $repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
    . (Join-Path $PSScriptRoot 'agent-cargo-common.ps1')
    $python = if ($PSBoundParameters.ContainsKey('PythonInterpreter')) {
        Resolve-AgentPython -PythonInterpreter $PythonInterpreter
    } else { Resolve-AgentPython }
    $arguments = @('-RepoRoot', $repo, '-PythonInterpreter', $python)
    if ($TargetDir) { $arguments += @('-TargetDir', $TargetDir) }
    if ($BuildRoot) { $arguments += @('-BuildRoot', $BuildRoot) }
    if ($DryRun) { $arguments += '-DryRun' }
    if ($QueryOnly) { $arguments += '-QueryOnly' }
    $cargo = @($Action, '--package', $Package, '--no-default-features', '--locked', '--jobs', [string]$Jobs)
    if ($PSBoundParameters.ContainsKey('ManifestPath')) {
        if ([string]::IsNullOrWhiteSpace($ManifestPath)) { throw '-ManifestPath requires a path.' }
        $cargo += @('--manifest-path', $ManifestPath)
    } elseif ($Package.StartsWith('zircon_plugin_', [StringComparison]::Ordinal)) {
        $cargo += @('--manifest-path', (Join-Path $repo 'zircon_plugins\Cargo.toml'))
    }
    if ($Features.Count) { $cargo += @('--features', ($Features -join ',')) }
    if ($Offline) { $cargo += '--offline' }
    if ($Release) { $cargo += '--release' }
    if ($LibTests) { $cargo += '--lib' }
    if ($TestTarget) { $cargo += @('--test', $TestTarget) }
    if ($TestFilter) { $cargo += $TestFilter }
    if ($NoRun) { $cargo += '--no-run' }
    if ($Action -eq 'test' -and -not $NoRun) { $cargo += @('--', '--test-threads', [string]$TestThreads) }
    # The common wrapper validates the exact manifest owner even for read-only plans.
    & (Join-Path $PSScriptRoot 'local-cargo.ps1') @arguments @cargo
    exit $LASTEXITCODE
} catch {
    [ordered]@{status='rejected';reasonCode='module_validation_input';message=$_.Exception.Message;
        executionOwner='Agent';commandExecuted=$false;formalAcceptance=$false} | ConvertTo-Json -Compress
    exit 2
}
