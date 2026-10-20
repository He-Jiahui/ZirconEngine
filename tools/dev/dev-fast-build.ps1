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
$taskClientCapsule = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
if ((Split-Path -Leaf $taskClientCapsule) -cmatch '^[0-9a-f]{64}$' -and
        (Test-Path -LiteralPath (Join-Path $taskClientCapsule 'capsule.json') -PathType Leaf)) {
    $taskClientRaw = [IO.File]::ReadAllBytes((Join-Path $taskClientCapsule 'capsule.json'))
    $taskClientMetadata = [Text.Encoding]::UTF8.GetString($taskClientRaw) | ConvertFrom-Json
    $taskClientSha = [Security.Cryptography.SHA256]::Create()
    try {
        $taskClientManifestHash = ([BitConverter]::ToString($taskClientSha.ComputeHash($taskClientRaw))).Replace('-','').ToLowerInvariant()
        $taskClientScriptHash = ([BitConverter]::ToString($taskClientSha.ComputeHash([IO.File]::ReadAllBytes($PSCommandPath)))).Replace('-','').ToLowerInvariant()
    } finally { $taskClientSha.Dispose() }
    if ($taskClientManifestHash -cne (Split-Path -Leaf $taskClientCapsule) -or
            $taskClientMetadata.clientRoot -cne 'clients' -or
            [string]$taskClientMetadata.clientSources.PSObject.Properties['tools/dev/dev-fast-build.ps1'].Value -cne $taskClientScriptHash) {
        throw 'Pinned management client root changed'
    }
    $taskRepo = [IO.Path]::GetFullPath([string]$taskClientMetadata.repositoryRoot)
}
$taskPython = Join-Path $taskRepo '.jenkins\runtime\python\python.exe'
$taskEntry = Join-Path $taskRepo 'tools\jenkins-management\entry.py'
try {
    $taskClientSelector = @'
import hashlib,importlib.util,json,sys
from pathlib import Path
repo=Path(sys.argv[1]).absolute();name=sys.argv[2]
installed_path=repo/'.jenkins/state/management/installed.json'
installed_raw=installed_path.read_bytes();installed=json.loads(installed_raw)
checksum=installed['capsuleSha256']
if not isinstance(checksum,str) or len(checksum)!=64 or any(c not in '0123456789abcdef' for c in checksum): raise ValueError('Invalid management capsule')
capsule=repo/'.jenkins/runtime/management'/checksum
raw=(capsule/'capsule.json').read_bytes();config=json.loads(raw)
if (Path(installed['capsule'])!=capsule or hashlib.sha256(raw).hexdigest()!=checksum or config['repositoryRoot']!=str(repo) or any(installed.get(k)!=config.get(k) for k in ('driverDigest','runtimeOperationId'))): raise ValueError('Installed management binding changed')
if 'clientRoot' in config:
    runner_path=capsule/'runner.py'
    if runner_path.resolve()!=runner_path.absolute() or hashlib.sha256(runner_path.read_bytes()).hexdigest()!=config['files']['runner.py']: raise ValueError('Pinned management runner changed')
    spec=importlib.util.spec_from_file_location('public_management_client_selector',runner_path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    path=module.client_path(config,repo,name,capsule=capsule)
else:
    path=repo/name
if path.resolve()!=path.absolute() or hashlib.sha256(path.read_bytes()).hexdigest()!=config['clientSources'][name] or installed_path.read_bytes()!=installed_raw: raise ValueError('Pinned management client changed')
print(str(path))
'@
    $taskEntry = & $taskPython -B -X utf8 -c $taskClientSelector $taskRepo 'tools/jenkins-management/entry.py'
    if ($LASTEXITCODE -ne 0 -or @($taskEntry).Count -ne 1) { throw 'Pinned management entry could not be selected' }
    $taskEntry = [string]$taskEntry
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
