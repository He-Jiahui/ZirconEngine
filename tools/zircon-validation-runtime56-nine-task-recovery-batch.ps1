[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime56_recovery_batch_"
$tasks = @(
    [pscustomobject]@{
        Name = "runtime56-nine-task-correctness"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--lib",
            "--locked", "--release", "--jobs", "1", $filter,
            "--", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime56-nine-task-performance"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--lib",
            "--locked", "--release", "--jobs", "1", $filter,
            "--", "--ignored", "--nocapture", "--test-threads=1"
        )
    }
)

Push-Location $repoRoot
try {
    foreach ($task in $tasks) {
        Write-Output "RUNTIME56_RECOVERY_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "RUNTIME56_RECOVERY_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME56_NINE_TASK_RECOVERY_BATCH_PASS task_count=$($tasks.Count)"
