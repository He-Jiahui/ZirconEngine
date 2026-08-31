[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime_hotpath_batch_"

Push-Location $repoRoot
try {
    cargo +1.94.1 test -p zircon_runtime --lib --locked --release --jobs 1 $filter -- `
        --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo +1.94.1 test -p zircon_runtime --lib --locked --release --jobs 1 $filter -- `
        --ignored --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    python -m unittest `
        tools.tests.test_runtime_input_manager_metadata_capacity_performance_contract `
        tools.tests.test_runtime_input_manager_metadata_capacity_pressure `
        tools.tests.test_runtime_keyboard_admission_performance_contract `
        tools.tests.test_runtime_keyboard_admission_pressure `
        tools.tests.test_runtime_path_module_validation_performance_contract `
        tools.tests.test_runtime_path_module_validation_pressure -v
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & rustfmt +1.94.1 --edition 2021 --check `
        zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs `
        zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests/reserve_capacity_tests.rs `
        zircon_runtime/src/ui/dispatch/input_manager/outcome.rs `
        zircon_runtime/src/ui/dispatch/input_manager/outcome/single_pass_metadata_tests.rs `
        zircon_runtime/src/ui/surface/input/keyboard_action.rs `
        zircon_runtime/src/ui/surface/input/keyboard_action/single_scan_text_tests.rs `
        zircon_runtime/src/ui/surface/input/keyboard_navigation.rs `
        zircon_runtime/src/ui/surface/input/keyboard_navigation/single_normalize_tests.rs `
        zircon_runtime/src/core/framework/scene/entity_path.rs `
        zircon_runtime/src/core/framework/scene/entity_path/single_scan_parse_tests.rs `
        zircon_runtime/src/plugin/extension_registry/validation/runtime_core.rs `
        zircon_runtime/src/plugin/extension_registry/validation/runtime_core/single_trim_tests.rs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    git diff --check -- `
        docs/plans/optimize/zircon_runtime/177/2026-08-26-dispatch-outcome-single-pass.md `
        docs/plans/optimize/zircon_runtime/182/2026-08-26-ime-host-request-capacity.md `
        docs/plans/optimize/zircon_runtime/337/2026-08-29-single-scan-keyboard-text.md `
        docs/plans/optimize/zircon_runtime/338/2026-08-29-single-normalize-direction-key.md `
        docs/plans/optimize/zircon_runtime/340/2026-08-29-preallocated-entity-path.md `
        docs/plans/optimize/zircon_runtime/340/2026-08-29-single-scan-entity-path.md `
        docs/plans/optimize/zircon_runtime/341/2026-08-29-single-trim-module-field.md `
        tools/zircon-validation-runtime-six-hotpath-performance-batch.ps1 `
        tools/runtime_input_manager_metadata_capacity_pressure.py `
        tools/runtime_keyboard_admission_pressure.py `
        tools/runtime_path_module_validation_pressure.py `
        tools/tests/test_runtime_input_manager_metadata_capacity_performance_contract.py `
        tools/tests/test_runtime_input_manager_metadata_capacity_pressure.py `
        tools/tests/test_runtime_keyboard_admission_performance_contract.py `
        tools/tests/test_runtime_keyboard_admission_pressure.py `
        tools/tests/test_runtime_path_module_validation_performance_contract.py `
        tools/tests/test_runtime_path_module_validation_pressure.py `
        zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs `
        zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests/reserve_capacity_tests.rs `
        zircon_runtime/src/ui/dispatch/input_manager/outcome.rs `
        zircon_runtime/src/ui/dispatch/input_manager/outcome/single_pass_metadata_tests.rs `
        zircon_runtime/src/ui/surface/input/keyboard_action.rs `
        zircon_runtime/src/ui/surface/input/keyboard_action/single_scan_text_tests.rs `
        zircon_runtime/src/ui/surface/input/keyboard_navigation.rs `
        zircon_runtime/src/ui/surface/input/keyboard_navigation/single_normalize_tests.rs `
        zircon_runtime/src/core/framework/scene/entity_path.rs `
        zircon_runtime/src/core/framework/scene/entity_path/single_scan_parse_tests.rs `
        zircon_runtime/src/plugin/extension_registry/validation/runtime_core.rs `
        zircon_runtime/src/plugin/extension_registry/validation/runtime_core/single_trim_tests.rs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME_SIX_HOTPATH_PERFORMANCE_BATCH_PASS rust_behavior_tests=12 ignored_release_benchmarks=6 python_contracts=27"
