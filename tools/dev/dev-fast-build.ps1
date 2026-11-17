[CmdletBinding()]
param(
    [ValidateSet('minimal', 'client2d', 'client3d', 'editor', 'dev', 'server')]
    [string]$Profile = 'client3d',
    [ValidateSet('check', 'build', 'test', 'run')][string]$Action = 'check',
    [ValidatePattern('^[A-Za-z0-9_][A-Za-z0-9_.-]*$')][string]$Package = 'zircon_app',
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
    [string]$JenkinsIdentityFile,
    [string]$PythonInterpreter,
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
try {
    if ($PSBoundParameters.ContainsKey('JenkinsIdentityFile')) {
        throw 'Jenkins workflow is retired; omit -JenkinsIdentityFile for Agent-owned execution.'
    }
    foreach ($unsupported in @('SourceSnapshot', 'SourceSnapshotDigest', 'InstallSccache',
            'RuntimeProductDll', 'CargoTimings', 'Bin', 'ArtifactOutputDirectory',
            'PublishArtifact', 'ExtraCargoArgs')) {
        if ($PSBoundParameters.ContainsKey($unsupported)) {
            throw "-$unsupported has no supported direct executor; no source copies, installs or publication were performed."
        }
    }
    if ($Action -eq 'run' -or $LinkMode -ne 'auto' -or $Linker -ne 'auto' -or
            $StorageMode -ne 'reuse' -or $CargoProfile -eq 'profiling') {
        throw 'This bounded entry supports check/build/test, default linking and reuse storage only; use an explicitly scoped owning product tool for other operations.'
    }
    if ($Release -and $CargoProfile -ne 'debug') { throw '-Release cannot be combined with -CargoProfile.' }
    if ($SharedTargetRoot -and $TargetDir) { throw 'Supply only one explicit requested target directory.' }
    if ($SharedTargetRoot) { $TargetDir = $SharedTargetRoot }
    if ($Action -ne 'test' -and ($LibTests -or $TestTarget -or $TestFilter -or $NoRun -or
            $PSBoundParameters.ContainsKey('TestThreads'))) { throw 'Test selectors require -Action test.' }
    if ($Action -eq 'test' -and (-not ($LibTests -or $TestTarget) -or ($LibTests -and $TestTarget))) {
        throw 'Select exactly one of -LibTests or -TestTarget for test validation.'
    }
    if ($NoRun -and $PSBoundParameters.ContainsKey('TestThreads')) { throw '-NoRun cannot use test harness options.' }
    $repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
    . (Join-Path $PSScriptRoot 'agent-cargo-common.ps1')
    $python = if ($PSBoundParameters.ContainsKey('PythonInterpreter')) {
        Resolve-AgentPython -PythonInterpreter $PythonInterpreter
    } else { Resolve-AgentPython }
    if ($PSBoundParameters.ContainsKey('FeatureOverride')) {
        if ([string]::IsNullOrWhiteSpace($FeatureOverride)) { throw '-FeatureOverride requires explicit nonempty features.' }
        $features = $FeatureOverride
    } else {
        if ($Package -notin @('zircon_app', 'zircon_runtime')) {
            throw 'Runtime presets belong to zircon_app/zircon_runtime; other packages need explicit -FeatureOverride or dev-module-build.ps1.'
        }
        # Read the canonical preset rather than copying its feature matrix into this client.
        $presetReader = @'
import json,sys,tomllib
from pathlib import Path
source=Path(sys.argv[1])/'zircon_runtime/runtime-feature-presets.toml'
data=tomllib.loads(source.read_text(encoding='utf-8'))
if data.get('schema_version') != 2: raise ValueError('Unsupported runtime preset schema')
choices=[p for p in data['profiles'] if p['id']==sys.argv[2]]
if len(choices)!=1: raise ValueError('Missing or duplicate runtime preset')
feature=choices[0]['cargo_feature']
if not isinstance(feature,str) or not feature: raise ValueError('Invalid preset Cargo feature')
# app_features/runtime_features are Cargo manifest expansion syntax (dep:/?/),
# not literal CLI features. Enable their owning alias instead.
manifest=tomllib.loads((Path(sys.argv[1])/sys.argv[3]/'Cargo.toml').read_text(encoding='utf-8'))
if feature not in manifest.get('features',{}): raise ValueError('Preset Cargo feature is not owned by the selected package')
print(feature)
'@
        $features = & $python -B -X utf8 -c $presetReader $repo $Profile $Package
        if ($LASTEXITCODE -ne 0 -or @($features).Count -ne 1) { throw 'Canonical runtime preset could not be read.' }
        $features = [string]$features
    }
    foreach ($feature in ($features -split '[,\s]+')) {
        if ($feature -notmatch '^[A-Za-z0-9_][A-Za-z0-9_./+-]*$') { throw 'Select explicit Cargo feature names.' }
    }
    $arguments = @('-RepoRoot', $repo, '-PythonInterpreter', $python)
    if ($BuildRoot) { $arguments += @('-BuildRoot', $BuildRoot) }
    if ($TargetDir) { $arguments += @('-TargetDir', $TargetDir) }
    if ($NoLocked) { $arguments += '-NoLocked' }
    if ($DryRun) { $arguments += '-DryRun' }
    if ($QueryOnly) { $arguments += '-QueryOnly' }
    $cargo = @($Action, '--package', $Package, '--no-default-features', '--features', $features,
        '--jobs', [string]$Jobs)
    if (-not $NoLocked) { $cargo += '--locked' }
    if ($Package.StartsWith('zircon_plugin_', [StringComparison]::Ordinal)) {
        $cargo += @('--manifest-path', (Join-Path $repo 'zircon_plugins\Cargo.toml'))
    }
    if ($Release -or $CargoProfile -eq 'release') { $cargo += '--release' }
    if ($Offline) { $cargo += '--offline' }
    if ($LibTests) { $cargo += '--lib' }
    if ($TestTarget) { $cargo += @('--test', $TestTarget) }
    if ($NoRun) { $cargo += '--no-run' }
    if ($TestFilter) { $cargo += $TestFilter }
    if ($Action -eq 'test' -and -not $NoRun) { $cargo += @('--', '--test-threads', [string]$TestThreads) }
    & (Join-Path $PSScriptRoot 'local-cargo.ps1') @arguments @cargo
    exit $LASTEXITCODE
} catch {
    [ordered]@{status='rejected';reasonCode='fast_build_input';message=$_.Exception.Message;
        executionOwner='Agent';commandExecuted=$false;formalAcceptance=$false} | ConvertTo-Json -Compress
    exit 2
}
