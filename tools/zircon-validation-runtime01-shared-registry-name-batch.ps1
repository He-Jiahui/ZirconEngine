[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$testPrefix = "core::runtime::descriptors::registry_name::tests"
$tasks = @(
    [pscustomobject]@{
        Name = "runtime01-registry-name-correctness"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::registry_name_clones_share_value_storage",
            "--", "--exact", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime01-registry-name-performance"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--locked",
            "--release", "--jobs", "1",
            "$testPrefix::registry_name_clone_release_benchmark_evidence",
            "--", "--exact", "--ignored", "--nocapture", "--test-threads=1"
        )
    }
)

Push-Location $repoRoot
try {
    & python -m unittest `
        tools.tests.test_runtime01_shared_registry_name_performance_contract -v
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    foreach ($task in $tasks) {
        Write-Output "RUNTIME01_BATCH_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "RUNTIME01_BATCH_TASK_PASS name=$($task.Name)"
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME01_BATCH_PASS task_count=$($tasks.Count)"
