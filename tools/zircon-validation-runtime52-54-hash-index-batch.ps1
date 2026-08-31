[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime_hash_recovery_batch_"
$tasks = @(
    [pscustomobject]@{
        Name = "runtime52-54-hash-correctness"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--lib",
            "--locked", "--release", "--jobs", "1", $filter,
            "--", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime52-54-hash-performance"
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
        Write-Output "RUNTIME_HASH_RECOVERY_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "RUNTIME_HASH_RECOVERY_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME52_54_HASH_RECOVERY_BATCH_PASS task_count=$($tasks.Count)"
