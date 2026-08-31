[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime85_project_dedup_recovery_batch_"
$modelSource = Join-Path $repoRoot "tools/runtime85_project_root_dedup_model.rs"
$modelBinary = Join-Path $repoRoot "target/runtime85-project-root-dedup-model.exe"
$tasks = @(
    [pscustomobject]@{
        Name = "runtime85-project-dedup-correctness"
        Arguments = @(
            "+1.94.1", "test", "-p", "zircon_runtime", "--lib",
            "--locked", "--release", "--jobs", "1", $filter,
            "--", "--nocapture", "--test-threads=1"
        )
    },
    [pscustomobject]@{
        Name = "runtime85-project-dependency-performance"
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
        Write-Output "RUNTIME85_RECOVERY_TASK_START name=$($task.Name)"
        $cargoArguments = $task.Arguments
        & cargo @cargoArguments
        if ($LASTEXITCODE -ne 0) {
            exit $LASTEXITCODE
        }
        Write-Output "RUNTIME85_RECOVERY_TASK_PASS name=$($task.Name)"
    }

    python -m unittest tools.tests.test_runtime85_project_root_dedup_performance_contract -v
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $modelBinary) | Out-Null
    & rustc +1.94.1 --edition=2024 -C opt-level=3 $modelSource -o $modelBinary
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
    & $modelBinary
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    & rustfmt +1.94.1 --edition 2024 --check `
        zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs `
        zircon_runtime/src/asset/project/package_asset_registry.rs `
        tools/runtime85_project_root_dedup_model.rs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    git diff --check -- `
        docs/plans/optimize/zircon_runtime/85/2026-08-24-project-import-dependency-dedup.md `
        docs/plans/optimize/zircon_runtime/85/2026-08-26-project-root-dedup-index.md `
        tools/runtime85_project_root_dedup_model.rs `
        tools/tests/test_runtime85_project_root_dedup_performance_contract.py `
        tools/zircon-validation-runtime85-project-dedup-recovery-batch.ps1 `
        zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs `
        zircon_runtime/src/asset/project/package_asset_registry.rs `
        zircon_runtime/src/asset/tests/project/package_assets.rs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME85_PROJECT_DEDUP_RECOVERY_BATCH_PASS cargo_task_count=$($tasks.Count) model_count=1 python_contract_count=1"
