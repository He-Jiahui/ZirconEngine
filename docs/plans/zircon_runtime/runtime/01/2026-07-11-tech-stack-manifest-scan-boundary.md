# Runtime 01 tech-stack manifest scan boundary

Date: 2026-07-11

Status: `runtime_01_tech_stack_manifest_scan_boundary_11_passed_cargo_pending`

## Scope

This slice repairs the lowest shared test-support layer exposed by the current Runtime 01 `tech_stack` Cargo attempt. It changes no production runtime behavior, dependency decision, manifest, or feature boundary.

## Failure evidence

- Stable Rust 1.94.1 reached a compiler ICE before the test executable was produced; the attempt is retained in `.codex/tmp/runtime01-tech-stack-wsl-20260711.log` and is not counted as a test failure.
- Nightly Rust 1.99 produced the current lib-test executable, but the filtered run remained inside repeated WSL `/mnt/e` manifest-tree scans. Process inspection showed the active test thread blocked in the Plan 9 filesystem RPC while reading repository directories, not in an engine subsystem or failed assertion.
- Direct execution reproduced the same behavior. The guard advanced one case at a time because each case rebuilt the same manifest inventory.

## Implementation

- `all_manifest_sources()` now uses `OnceLock` so one test process constructs the manifest inventory once.
- `collect_manifest_sources()` starts at the repository root but descends only into current top-level `zircon_*` product trees, explicitly retaining the nested `zircon_plugins` workspace.
- Generated/cache/reference/documentation/test/tool roots and symbolic-link directories are excluded. Direct dependency declarations in all current product manifests remain covered.

## Verification

- `rustfmt +nightly --edition 2021 --check zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`: passed.
- Current-source Windows standalone guard: 11 passed / 0 failed / 0 ignored / 0 filtered in 1.04 seconds (1.53 seconds process wall time).
- The pre-cache bounded WSL scan also completed 11/11, proving the scope did not drop current manifest assertions; caching removes its repeated per-case cost.

## Remaining gate

Runtime 01 remains `in_progress`. A fresh package-level binary must include this support-layer fix before `tech_stack`, `extensions`, `text_shaper`, and `export_build_plan` can be accepted. Physics plugin acceptance remains a separate plugin-workspace gate.

## Follow-up gate result

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-runtime01-physics-0711`: exit 0 after a fresh 15m54s build.
- Physics package unit tests: 10 passed / 0 failed.
- Physics runtime contract integration tests: 33 passed / 0 failed.
- Doc tests: 0 tests, exit 0.

This closes the Runtime 01 default-feature Physics plugin gate. It does not replace the separate feature-on Jolt matrix already owned by the Physics plan, and it does not promote the remaining four runtime-package filters.
