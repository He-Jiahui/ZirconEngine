[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime88_borrowed_event_fold_batch_"
$modelSource = Join-Path $repoRoot "tools/runtime88_borrowed_event_fold_model.rs"
$modelBinary = Join-Path $repoRoot "target/runtime88-borrowed-event-fold-model.exe"

Push-Location $repoRoot
try {
    cargo +1.94.1 test -p zircon_runtime --lib --locked --release --jobs 1 $filter -- `
        --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    python -m unittest tools.tests.test_runtime88_borrowed_event_fold_performance_contract -v
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
        zircon_runtime/src/asset/watch/fold_events.rs `
        zircon_runtime/src/asset/watch/fold_events/tests.rs `
        tools/runtime88_borrowed_event_fold_model.rs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    git diff --check -- `
        docs/plans/optimize/zircon_runtime/88/2026-08-26-borrowed-event-fold.md `
        tools/runtime88_borrowed_event_fold_model.rs `
        tools/tests/test_runtime88_borrowed_event_fold_performance_contract.py `
        tools/zircon-validation-runtime88-borrowed-event-fold-batch.ps1 `
        zircon_runtime/src/asset/watch/fold_events.rs `
        zircon_runtime/src/asset/watch/fold_events/tests.rs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME88_BORROWED_EVENT_FOLD_BATCH_PASS rust_tests=2 python_contracts=6 performance_models=1"
