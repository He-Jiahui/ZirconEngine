[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$testPrefix = "ui::component::descriptor::validation::hash_membership_tests"
$tasks = @(
    [pscustomobject]@{
        Name = "runtime75-descriptor-hash-correctness"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::optimization_batch_20260826ag_runtime75_hash_validation_preserves_first_duplicate_errors",
            "--", "--exact", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime75-descriptor-hash-source-contract"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::optimization_batch_20260826ag_runtime75_descriptor_validation_uses_borrowed_hash_membership",
            "--", "--exact", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime75-descriptor-hash-performance"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::optimization_batch_20260826ag_runtime75_descriptor_hash_validation_performance_evidence",
            "--", "--exact", "--ignored", "--nocapture", "--test-threads=1"
        )
    }
)

Push-Location $repoRoot
try {
    foreach ($task in $tasks) {
        Write-Output "RUNTIME75_DESCRIPTOR_HASH_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "RUNTIME75_DESCRIPTOR_HASH_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME75_DESCRIPTOR_HASH_BATCH_PASS task_count=$($tasks.Count)"
