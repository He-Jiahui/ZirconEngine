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
    [string]$JenkinsIdentityFile = '',
    [switch]$DryRun,
    [switch]$QueryOnly
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
try {
    if ($Action -ne 'test' -and ($LibTests -or $TestTarget -or $TestFilter -or $NoRun -or
            $PSBoundParameters.ContainsKey('TestThreads'))) { throw 'Test selectors require -Action test.' }
    if ($Action -eq 'test' -and (-not ($LibTests -or $TestTarget) -or ($LibTests -and $TestTarget))) {
        throw 'Select exactly one of -LibTests or -TestTarget for module validation.'
    }
    if ($NoRun -and $PSBoundParameters.ContainsKey('TestThreads')) { throw '-NoRun cannot use test harness options.' }
    if ($Package -eq 'zircon_runtime' -and -not $PSBoundParameters.ContainsKey('Features')) {
        $Features = @('core-min')
    }
    foreach ($taskFeature in $Features) {
        if ($taskFeature -notmatch '^[A-Za-z0-9_][A-Za-z0-9_./+-]*$') { throw 'Select explicit Cargo feature names.' }
    }
    $taskPalette = $Package -ceq 'zircon_plugin_animation_graph_editor'
    if ($taskPalette -and ($Action -ne 'test' -or -not $LibTests -or $TestTarget -or
            $TestFilter -cne 'tests::graph_palette_matches_creatable_editor_capabilities' -or
            $Features.Count -ne 0 -or $Jobs -ne 1 -or $TestThreads -ne 1 -or
            $NoRun -or $Release -or $Offline)) {
        throw 'Animation palette validation requires the single bounded plugin lib test recipe.'
    }
    $taskRepo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
    $taskPython = Join-Path $taskRepo '.jenkins\runtime\python\python.exe'
    $taskEntry = Join-Path $taskRepo 'tools\jenkins-management\entry.py'
    $taskArguments = @('-B', '-X', 'utf8', $taskEntry, 'cargo', '--repo-root', $taskRepo)
    if ($JenkinsIdentityFile) { $taskArguments += @('--identity-file', $JenkinsIdentityFile) }
    if ($DryRun) { $taskArguments += '--dry-run' }
    if ($QueryOnly) { $taskArguments += '--query-only' }
    $taskCargo = @($Action, '--package', $Package, '--no-default-features', '--locked', '--jobs', [string]$Jobs)
    if ($taskPalette) {
        $taskCargo += @('--manifest-path', (Join-Path $taskRepo 'zircon_plugins\Cargo.toml'), '--profile', 'test')
    }
    if ($Features) { $taskCargo += @('--features', ($Features -join ',')) }
    if ($Offline) { $taskCargo += '--offline' }
    if ($Release) { $taskCargo += '--release' }
    if ($LibTests) { $taskCargo += '--lib' }
    if ($TestTarget) { $taskCargo += @('--test', $TestTarget) }
    if ($TestFilter) { $taskCargo += $TestFilter }
    if ($NoRun) { $taskCargo += '--no-run' }
    if ($Action -eq 'test' -and -not $NoRun) { $taskCargo += @('--', '--test-threads', [string]$TestThreads) }
    if ($DryRun) {
        [ordered]@{status='planned';package=$Package;cargoArguments=$taskCargo;
            executionOwner='Jenkins';commandExecuted=$false;formalAcceptance=$false} | ConvertTo-Json -Depth 5 -Compress
        exit 0
    }
    $taskArguments += @('--') + $taskCargo
    & $taskPython @taskArguments
    exit $LASTEXITCODE
} catch {
    [ordered]@{status='pending';reasonCode='module_validation_input';message=$_.Exception.Message;
        formalAcceptance=$false} | ConvertTo-Json -Compress
    exit 2
}
