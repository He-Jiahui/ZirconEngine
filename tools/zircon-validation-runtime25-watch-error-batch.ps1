[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$testPrefix = "asset::pipeline::manager::project_asset_manager::watch_dispatch::tests"
$tasks = @(
    [pscustomobject]@{
        Name = "runtime25-watch-error-correctness"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::activation_error_overflow_discards_oldest_and_preserves_fifo_order",
            "--", "--exact", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime25-watch-error-performance"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::watch_error_tail_queue_release_benchmark_evidence",
            "--", "--exact", "--ignored", "--nocapture", "--test-threads=1"
        )
    }
)

Push-Location $repoRoot
try {
    foreach ($task in $tasks) {
        Write-Output "RUNTIME25_BATCH_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "RUNTIME25_BATCH_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME25_BATCH_PASS task_count=$($tasks.Count)"
