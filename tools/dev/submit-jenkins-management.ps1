[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][ValidateSet('create-branch','create-worktree','prepare-merge','apply-patch','commit-worktree','release-worktree','validate','generate','submit-flow')][string]$Action,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$')][string]$SessionId,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$')][string]$RequestId,
    [Parameter(Mandatory=$true)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$')][string]$OperationId,
    [Parameter(Mandatory=$true)][string]$InputFile,
    [string]$RepoRoot = 'E:/Git/ZirconEngine',
    [switch]$QueryOnly
)
$ErrorActionPreference = 'Stop'
try {
    $taskRepo = [IO.Path]::GetFullPath($RepoRoot)
    $taskPython = Join-Path $taskRepo '.jenkins\runtime\python\python.exe'
    $taskEntry = Join-Path $taskRepo 'tools\jenkins-management\entry.py'
    foreach ($taskPath in @($taskPython, $taskEntry, $InputFile)) {
        if (-not (Test-Path -LiteralPath $taskPath -PathType Leaf)) { throw "Required Jenkins input is missing: $taskPath" }
    }
    $taskArguments = @('-B', '-X', 'utf8', $taskEntry, 'submit', '--repo-root', $taskRepo,
        '--input-file', $InputFile, '--session-id', $SessionId, '--request-id', $RequestId,
        '--operation-id', $OperationId, '--action', $Action)
    if ($QueryOnly) { $taskArguments += '--query-only' }
    & $taskPython @taskArguments
    exit $LASTEXITCODE
} catch {
    [ordered]@{status='pending';reasonCode='ordinary_entry_pending';message=$_.Exception.Message;
        formalAcceptance=$false;m8Accepted=$false} | ConvertTo-Json -Compress
    exit 2
}
