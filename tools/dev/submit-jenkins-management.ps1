[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][ValidateSet('create-branch','create-worktree','prepare-merge','apply-patch','commit-checkout','commit-worktree','release-worktree','recover-worktree','validate','generate','prepare-flow','submit-flow','prepare-worktree','inspect-worktree','retire-worktree-reader','preview-current-editor-r34')][string]$Action,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$')][string]$SessionId,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$')][string]$RequestId,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$')][string]$OperationId,
    [Parameter(Mandatory=$true)][string]$InputFile,
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [switch]$QueryOnly,
    [switch]$DeferCommit
)
$ErrorActionPreference = 'Stop'
try {
    $taskRepo = [IO.Path]::GetFullPath($RepoRoot)
    $taskPython = Join-Path $taskRepo '.jenkins\runtime\python\python.exe'
    $taskEntry = Join-Path $taskRepo 'tools\jenkins-management\entry.py'
    if ($Action -eq 'commit-checkout') {
        $taskEntry = Join-Path $taskRepo 'tools\dev\jenkins_commit.py'
    } else {
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
    }
    foreach ($taskPath in @($taskPython, $taskEntry, $InputFile)) {
        if (-not (Test-Path -LiteralPath $taskPath -PathType Leaf)) { throw "Required Jenkins input is missing: $taskPath" }
    }
    $taskArguments = @('-B', '-X', 'utf8', $taskEntry, 'submit', '--repo-root', $taskRepo,
        '--input-file', $InputFile, '--session-id', $SessionId, '--request-id', $RequestId,
        '--operation-id', $OperationId)
    if ($Action -ne 'commit-checkout') { $taskArguments += @('--action', $Action) }
    if ($DeferCommit) {
        if ($Action -ne 'commit-checkout') { throw 'DeferCommit requires Action=commit-checkout' }
        $taskArguments += '--defer'
    }
    if ($QueryOnly) { $taskArguments += '--query-only' }
    & $taskPython @taskArguments
    exit $LASTEXITCODE
} catch {
    [ordered]@{status='pending';reasonCode='ordinary_entry_pending';message=$_.Exception.Message;
        formalAcceptance=$false;m8Accepted=$false} | ConvertTo-Json -Compress
    exit 2
}
