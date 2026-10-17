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
