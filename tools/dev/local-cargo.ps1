Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# A PowerShell advanced parameter block consumes Cargo's -p as PipelineVariable.
# Parse only wrapper options before the Cargo verb and forward the rest exactly.
try {
$localArguments = @($args)
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
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
            [string]$taskClientMetadata.clientSources.PSObject.Properties['tools/dev/local-cargo.ps1'].Value -cne $taskClientScriptHash) {
        throw 'Pinned management client root changed'
    }
    $RepoRoot = [IO.Path]::GetFullPath([string]$taskClientMetadata.repositoryRoot)
}
$TargetDir = $null
$BuildRoot = $null
$QueryOnly = $false
$NoLocked = $false
$DryRun = $false
$JenkinsIdentityFile = $null
$PythonInterpreter = $null
$IndependentPreview = $false
$EvidenceDirectory = $null
$EvidenceInputManifest = $null
$EvidenceTimeoutSeconds = $null
$argumentIndex = 0
while ($argumentIndex -lt $localArguments.Count) {
    $option = [string]$localArguments[$argumentIndex]
    if ($option -eq '-QueryOnly') { $QueryOnly = $true; $argumentIndex++; continue }
    if ($option -eq '-NoLocked') { $NoLocked = $true; $argumentIndex++; continue }
    if ($option -eq '-DryRun') { $DryRun = $true; $argumentIndex++; continue }
    if ($option -eq '-IndependentPreview') { $IndependentPreview = $true; $argumentIndex++; continue }
    if ($option -in @('-RepoRoot', '-TargetDir', '-BuildRoot', '-JenkinsIdentityFile', '-PythonInterpreter', '-EvidenceDirectory', '-EvidenceInputManifest', '-EvidenceTimeoutSeconds')) {
        if ($argumentIndex + 1 -ge $localArguments.Count) { throw "$option requires a value" }
        $argumentIndex++
        if ($option -eq '-RepoRoot') { $RepoRoot = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-TargetDir') { $TargetDir = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-BuildRoot') { $BuildRoot = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-PythonInterpreter') { $PythonInterpreter = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-EvidenceDirectory') { $EvidenceDirectory = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-EvidenceInputManifest') { $EvidenceInputManifest = [string]$localArguments[$argumentIndex] }
        elseif ($option -eq '-EvidenceTimeoutSeconds') { $EvidenceTimeoutSeconds = [string]$localArguments[$argumentIndex] }
        else { $JenkinsIdentityFile = [string]$localArguments[$argumentIndex] }
        $argumentIndex++
        continue
    }
    break
}
$CargoArguments = if ($argumentIndex -lt $localArguments.Count) {
    @($localArguments[$argumentIndex..($localArguments.Count - 1)])
} else { @() }
$resolvedRepo = (Resolve-Path -LiteralPath $RepoRoot).Path
$jenkinsPython = if ($PythonInterpreter) { $PythonInterpreter } else { Join-Path $resolvedRepo '.jenkins\runtime\python\python.exe' }
if (-not (Test-Path -LiteralPath $jenkinsPython -PathType Leaf)) { throw "Jenkins runtime Python is missing: $jenkinsPython" }
if ($IndependentPreview -or $null -ne $EvidenceDirectory -or $null -ne $EvidenceInputManifest -or
        $null -ne $EvidenceTimeoutSeconds) { throw 'Independent Cargo execution/evidence options have no ordinary Jenkins recipe' }
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
$taskEntry = & $jenkinsPython -B -X utf8 -c $taskClientSelector $resolvedRepo 'tools/jenkins-management/entry.py'
if ($LASTEXITCODE -ne 0 -or @($taskEntry).Count -ne 1) { throw 'Pinned management entry could not be selected' }
$pythonArguments = @('-B', '-X', 'utf8', [string]$taskEntry, 'cargo', '--repo-root', $resolvedRepo)
if ($JenkinsIdentityFile) { $pythonArguments += @('--identity-file', $JenkinsIdentityFile) }
if (-not [string]::IsNullOrWhiteSpace($TargetDir)) { $pythonArguments += @('--target-dir', $TargetDir) }
if ($BuildRoot) { $pythonArguments += @('--build-root', $BuildRoot) }
if ($QueryOnly) { $pythonArguments += '--query-only' }
if ($NoLocked) { $pythonArguments += '--unlocked' }
if ($DryRun) { $pythonArguments += '--dry-run' }
$pythonArguments += @('--') + @($CargoArguments)
Push-Location -LiteralPath $resolvedRepo
try {
    & $jenkinsPython @pythonArguments
    $cargoExitCode = $LASTEXITCODE
}
finally { Pop-Location }
exit $cargoExitCode

} catch {
    [ordered]@{status='pending';reasonCode='ordinary_entry_pending';message=$_.Exception.Message;
        formalAcceptance=$false;m8Accepted=$false} | ConvertTo-Json -Compress
    exit 2
}
