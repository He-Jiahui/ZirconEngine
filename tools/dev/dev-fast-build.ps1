[CmdletBinding()]
param(
    [ValidateSet('minimal', 'client2d', 'client3d', 'editor', 'dev', 'server')]
    [string]$Profile = 'client3d',
    [ValidateSet('check', 'build', 'test', 'run')][string]$Action = 'check',
    [string]$Package = 'zircon_app',
    [string]$SourceSnapshot,
    [string]$SourceSnapshotDigest,
    [switch]$Release,
    [switch]$NoLocked,
    [switch]$InstallSccache,
    [string]$SharedTargetRoot = '',
    [string]$FeatureOverride = '',
    [ValidateSet('debug', 'release', 'profiling')][string]$CargoProfile = 'debug',
    [ValidateSet('auto', 'static', 'dev-dynamic')][string]$LinkMode = 'auto',
    [ValidateSet('auto', 'lld', 'system')][string]$Linker = 'auto',
    [ValidateSet('reuse', 'compact', 'diagnostic')][string]$StorageMode = 'reuse',
    [switch]$RuntimeProductDll,
    [switch]$CargoTimings,
    [switch]$LibTests,
    [string]$TestTarget,
    [string]$TestFilter,
    [string]$Bin,
    [string]$ArtifactOutputDirectory,
    [string[]]$PublishArtifact,
    [string]$JenkinsIdentityFile = '',
    [string]$BuildRoot = '',
    [string]$TargetDir = '',
    [ValidateRange(1,8)][int]$Jobs = 1,
    [ValidateRange(1,8)][int]$TestThreads = 1,
    [switch]$Offline,
    [switch]$NoRun,
    [switch]$QueryOnly,
    [switch]$DryRun,
    [string[]]$ExtraCargoArgs
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$taskRepo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$taskPython = Join-Path $taskRepo '.jenkins\runtime\python\python.exe'
$taskEntry = Join-Path $taskRepo 'tools\jenkins-management\entry.py'
try {
    if ($Action -eq 'run' -or $SourceSnapshot -or $SourceSnapshotDigest -or $InstallSccache -or
        $LinkMode -ne 'auto' -or $Linker -ne 'auto' -or $StorageMode -ne 'reuse' -or
        $RuntimeProductDll -or $CargoTimings -or $Bin -or $ArtifactOutputDirectory -or
        $PublishArtifact -or $ExtraCargoArgs -or $CargoProfile -eq 'profiling') {
        throw 'A requested option has no ordinary Jenkins validate recipe; no local fallback is available.'
    }
    if ($Release -and $CargoProfile -ne 'debug') { throw '-Release cannot be combined with -CargoProfile.' }
    if ($SharedTargetRoot -and $TargetDir) { throw 'Supply only one explicit requested target directory.' }
    if ($SharedTargetRoot) { $TargetDir = $SharedTargetRoot }
    if ($Action -ne 'test' -and ($LibTests -or $TestTarget -or $TestFilter -or $NoRun -or $PSBoundParameters.ContainsKey('TestThreads'))) {
        throw 'Test selectors require -Action test.'
    }
    if ($NoRun -and $PSBoundParameters.ContainsKey('TestThreads')) { throw '-NoRun cannot use test harness options.' }
    foreach ($taskPath in @($taskPython, $taskEntry)) {
        if (-not (Test-Path -LiteralPath $taskPath -PathType Leaf)) { throw "Required Jenkins entry file is missing: $taskPath" }
    }
    $taskArguments = @('-B', '-X', 'utf8', $taskEntry, 'cargo', '--repo-root', $taskRepo)
    if ($JenkinsIdentityFile) { $taskArguments += @('--identity-file', $JenkinsIdentityFile) }
    if ($BuildRoot) { $taskArguments += @('--build-root', $BuildRoot) }
    if ($TargetDir) { $taskArguments += @('--target-dir', $TargetDir) }
    if (-not $FeatureOverride) { $taskArguments += @('--runtime-profile', $Profile) }
    if ($DryRun) { $taskArguments += '--dry-run' }
    if ($QueryOnly) { $taskArguments += '--query-only' }
    $taskCargo = @($Action, '--package', $Package, '--jobs', [string]$Jobs)
    if ($FeatureOverride) { $taskCargo += @('--no-default-features', '--features', $FeatureOverride) }
    if ($Release -or $CargoProfile -eq 'release') { $taskCargo += '--release' }
    if ($NoLocked) {
        # This choice belongs to the immutable input. The client rejects a
        # differing locked value rather than dropping the caller's option.
        $taskArguments += '--unlocked'
    } else { $taskCargo += '--locked' }
    if ($Offline) { $taskCargo += '--offline' }
    if ($LibTests) { $taskCargo += '--lib' }
    if ($TestTarget) { $taskCargo += @('--test', $TestTarget) }
    if ($NoRun) { $taskCargo += '--no-run' }
    if ($TestFilter) { $taskCargo += $TestFilter }
    if ($Action -eq 'test' -and -not $NoRun) { $taskCargo += @('--', '--test-threads', [string]$TestThreads) }
    $taskArguments += @('--') + $taskCargo
    & $taskPython @taskArguments
    exit $LASTEXITCODE
} catch {
    [ordered]@{status='pending';reasonCode='ordinary_entry_pending';message=$_.Exception.Message;
        formalAcceptance=$false;m8Accepted=$false} | ConvertTo-Json -Compress
    exit 2
}
