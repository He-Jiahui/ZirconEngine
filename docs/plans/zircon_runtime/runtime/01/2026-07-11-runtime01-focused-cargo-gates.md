# Runtime 01 focused Cargo gates

Date: 2026-07-11

Status: `runtime_01_focused_cargo_gates_tech_stack_14_text_shaper_7_export_67_physics_43_passed_extensions_pending`

## Scope

This record owns the current-source focused Cargo evidence for Runtime 01. It closes four declared gates and leaves the aggregate `extensions` filter explicit.

## Fresh binary and compiler evidence

- Stable Rust 1.94.1 on WSL ICE'd before test execution; `.codex/tmp/runtime01-tech-stack-wsl-20260711.log` is retained as compiler evidence only.
- A fresh Windows default-feature locked lib-test build completed after the command host timed out. The resulting current binary is `D:\cargo-targets\zircon-runtime01-default-0711\debug\deps\zircon_runtime-4d1af4dc02332e8d.exe`.
- The first current `export_build_plan` execution exposed two stale tests that assumed the Sound timeline feature was owner-local. Current catalog truth declares `provider_package_id = sound_timeline_animation_track`, so provider selection is required for linkage and a missing required provider is reported as a blocked-feature fatal diagnostic. Test names, fixtures, expected path, and expected diagnostic were reconciled; production export code was unchanged.
- Focused reconciliation command `cargo test -p zircon_runtime --lib export_build_plan_feature_provider --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-runtime01-default-0711 -- --nocapture --test-threads=1`: 6 passed / 0 failed.

## Current gate results

- `tech_stack`: 14 passed / 0 failed / 7469 filtered, 0.97s test time.
- `text_shaper`: 7 passed / 0 failed / 7476 filtered, 25.97s test time.
- `export_build_plan`: 67 passed / 0 failed / 7416 filtered, 1.66s test time.
- Physics plugin default-feature workspace gate: unit 10/10 plus integration contract 33/33, doc tests 0, exit 0.

## Remaining gate

Runtime 01 remains `in_progress` until the aggregate locked `extensions` filter runs on this current source and any lowest-layer failures are resolved. No workspace-wide or all-feature completion is claimed here.
