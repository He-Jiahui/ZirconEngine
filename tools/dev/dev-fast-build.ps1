[CmdletBinding()]
param(
    [ValidateSet('minimal', 'client2d', 'client3d', 'editor', 'dev', 'server')]
    [string]$Profile = 'client3d',
    [ValidateSet('check', 'build', 'test', 'run')]
    [string]$Action = 'check',
    [string]$Package = 'zircon_app',
    [string]$SourceSnapshot,
    [string]$SourceSnapshotDigest,
    [switch]$Release,
    [switch]$NoLocked,
    [switch]$InstallSccache,
    [string]$SharedTargetRoot = '',
    [string]$FeatureOverride = '',
    [ValidateSet('debug', 'release', 'profiling')]
    [string]$CargoProfile = 'debug',
    [ValidateSet('auto', 'static', 'dev-dynamic')]
    [string]$LinkMode = 'auto',
    [ValidateSet('auto', 'lld', 'system')]
    [string]$Linker = 'auto',
    [ValidateSet('reuse', 'compact', 'diagnostic')]
    [string]$StorageMode = 'reuse',
    [switch]$RuntimeProductDll,
    [switch]$CargoTimings,
    [switch]$LibTests,
    [string]$TestTarget,
    [string]$TestFilter,
    [string]$Bin,
    [string]$ArtifactOutputDirectory,
    [string[]]$PublishArtifact,
    [string]$JenkinsIdentityFile = '',
    [switch]$DryRun,
    [string[]]$ExtraCargoArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$env:PYTHONPATH = if ($env:PYTHONPATH) { "$repoRoot;$env:PYTHONPATH" } else { $repoRoot }
$jenkinsPython = Join-Path $repoRoot '.jenkins\runtime\python\python.exe'
if (-not (Test-Path -LiteralPath $jenkinsPython -PathType Leaf)) { throw "Jenkins runtime Python is missing: $jenkinsPython" }
$jenkinsGate = (& $jenkinsPython -B -m tools.jenkins.frontend gate --repo-root $repoRoot).Trim()
if ($jenkinsGate -eq 'true') {
    if ([string]::IsNullOrWhiteSpace($JenkinsIdentityFile) -or -not (Test-Path -LiteralPath $JenkinsIdentityFile -PathType Leaf)) {
        throw 'Jenkins sole-entry mode requires -JenkinsIdentityFile containing the request/source/coverage identity.'
    }
    $env:ZIRCON_JENKINS_IDENTITY_FILE = (Resolve-Path -LiteralPath $JenkinsIdentityFile).Path
    & $jenkinsPython -B -m tools.jenkins.frontend dispatch --repo-root $repoRoot --job zircon-flow
    exit $LASTEXITCODE
}
if ($Release -and $CargoProfile -ne 'debug') { throw '-Release cannot be combined with -CargoProfile.' }
if ($InstallSccache -and -not (Get-Command sccache -ErrorAction SilentlyContinue)) {
    throw 'Install sccache before invoking the managed build, or use -StorageMode diagnostic.'
}
if ($SharedTargetRoot) {
    throw '-SharedTargetRoot has been replaced by coordinator compatibility pools. Omit it to reuse the managed cache.'
}
if ($ExtraCargoArgs -and $Action -ne 'run') {
    throw 'Use -LibTests, -TestTarget, -TestFilter and -Bin for compilation selectors. ExtraCargoArgs is reserved for application arguments with -Action run.'
}
$features = $FeatureOverride
if (-not $features) {
    $features = & $jenkinsPython -B (Join-Path $repoRoot 'tools/analysis/validation/runtime-profile-feature-presets.py') feature $Profile
    if ($LASTEXITCODE -ne 0 -or -not $features) { throw "Unknown profile feature preset: $Profile" }
    $features = $features.Trim()
}
$parameters = @{
    RepoRoot = $repoRoot
    SourceSnapshot = $SourceSnapshot
    SourceSnapshotDigest = $SourceSnapshotDigest
    Package = $Package
    Features = $features
    NoDefaultFeatures = $true
    NoLocked = $NoLocked
    CargoProfile = $(if ($Release) { 'release' } elseif ($CargoProfile -eq 'debug') { 'development' } else { $CargoProfile })
    LinkMode = $LinkMode
    Linker = $Linker
    StorageMode = $StorageMode
    RuntimeProductDll = $RuntimeProductDll
    CargoTimings = $CargoTimings
    DryRun = $DryRun
}
switch ($Action) {
    'check' { $parameters.CheckOnly = $true }
    'build' { $parameters.SkipTest = $true }
    'test' { $parameters.SkipBuild = $true }
    'run' {
        $parameters.SkipTest = $true
        $parameters.Run = $true
        $parameters.RunArguments = $ExtraCargoArgs
        if (-not $Bin) {
            $Bin = switch ($Profile) {
                { $_ -in 'client2d', 'client3d' } { 'zircon_runtime' }
                { $_ -in 'editor', 'dev' } { 'zircon_editor' }
                default { throw "Profile $Profile has no executable; select -Bin explicitly." }
            }
        }
    }
}
if ($RuntimeProductDll) { $parameters.CheckOnly = $false; $parameters.SkipBuild = $false; $parameters.SkipTest = $true }
if ($LibTests) { $parameters.LibTests = $true }
if ($TestTarget) { $parameters.TestTarget = $TestTarget }
if ($TestFilter) { $parameters.TestFilter = $TestFilter }
if ($Bin) { $parameters.Bin = $Bin }
if ($ArtifactOutputDirectory) { $parameters.ArtifactOutputDirectory = $ArtifactOutputDirectory }
if ($PublishArtifact) { $parameters.PublishArtifact = $PublishArtifact }
& (Join-Path $repoRoot '.codex/skills/zircon-dev/scripts/validate-matrix.ps1') @parameters
exit $LASTEXITCODE
