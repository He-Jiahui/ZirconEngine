[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$filter = "runtime200_"

Push-Location $repoRoot
try {
    cargo +1.94.1 test -p zircon_runtime --lib --locked --release --jobs 1 $filter -- `
        --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    cargo +1.94.1 test -p zircon_runtime --lib --locked --release --jobs 1 $filter -- `
        --ignored --nocapture --test-threads=1
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    python -m unittest `
        tools.tests.test_runtime_navigation_dispatch_ownership_performance_contract `
        tools.tests.test_runtime_pointer_dispatch_input_ownership_performance_contract `
        tools.tests.test_runtime_pointer_route_trace_ownership_performance_contract `
        tools.tests.test_runtime_ui_dispatch_route_sharing_performance_contract `
        tools.tests.test_runtime_ui_dispatch_route_sharing_pressure `
        tools.tests.test_runtime_pointer_hover_diff_performance_contract `
        tools.tests.test_runtime_pointer_hover_path_performance_contract `
        tools.tests.test_runtime_pointer_hover_hot_paths_pressure -v
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & rustfmt +1.94.1 --edition 2021 --check `
        zircon_runtime/src/ui/dispatch/visited_node_set.rs `
        zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs `
        zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs `
        zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs `
        zircon_runtime/src/ui/dispatch/input_manager/pointer_table/hovered_path_tests.rs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    git diff --check -- `
        docs/plans/optimize/zircon_runtime/200/2026-08-31-borrowed-dispatch-route-sharing.md `
        docs/plans/optimize/zircon_runtime/200/2026-08-31-pointer-hover-hot-paths.md `
        tools/zircon-validation-runtime200-ui-route-performance-batch.ps1 `
        tools/runtime_ui_dispatch_route_sharing_pressure.py `
        tools/runtime_pointer_hover_hot_paths_pressure.py `
        tools/tests/test_runtime_navigation_dispatch_ownership_performance_contract.py `
        tools/tests/test_runtime_pointer_dispatch_input_ownership_performance_contract.py `
        tools/tests/test_runtime_pointer_route_trace_ownership_performance_contract.py `
        tools/tests/test_runtime_ui_dispatch_route_sharing_performance_contract.py `
        tools/tests/test_runtime_ui_dispatch_route_sharing_pressure.py `
        tools/tests/test_runtime_pointer_hover_diff_performance_contract.py `
        tools/tests/test_runtime_pointer_hover_path_performance_contract.py `
        tools/tests/test_runtime_pointer_hover_hot_paths_pressure.py `
        zircon_runtime/src/ui/dispatch/visited_node_set.rs `
        zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs `
        zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs `
        zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs `
        zircon_runtime/src/ui/dispatch/input_manager/pointer_table/hovered_path_tests.rs `
        zircon_runtime/src/ui/surface/surface/event_routing.rs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
finally {
    Pop-Location
}

Write-Output "RUNTIME200_UI_ROUTE_PERFORMANCE_BATCH_PASS rust_behavior_tests=6 ignored_release_benchmarks=1"
