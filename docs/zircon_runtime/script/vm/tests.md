---
related_code:
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - zircon_runtime/src/script/vm/tests/support.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs
implementation_files:
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/src/script/vm/tests/plugin_runtime.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - zircon_runtime/src/script/vm/tests/support.rs
  - zircon_runtime/src/script/vm/tests/lifecycle_failures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - runtime_15_script_vm_tests_are_folder_backed
  - runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code
  - rustfmt --edition 2021 --check zircon_runtime/src/script/vm/tests.rs zircon_runtime/src/script/vm/tests/*.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs
  - cargo test -p zircon_runtime --lib runtime_15_script_vm_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Script VM Test Layout

`zircon_runtime/src/script/vm/tests.rs` is now a folder-backed test entry point for the script VM subsystem. The parent file keeps only module mounts and imports; executable tests live in child owners under `zircon_runtime/src/script/vm/tests/`.

## Owners

- `host_exports.rs` owns host handle, capability registry, host export registry, script call table, and real backend call-site source guards.
- `bridge_host.rs` owns bridge host module registration, manifest-backed bridge methods, missing binding rejection, and disabled bridge dispatch behavior.
- `reflection_docs.rs` owns host reflection Markdown rendering, heading clamping, file writing, macro-generated builtin module coverage, and reflection macro descriptor tests. Runtime 15 F12 script reflection macro fixture dead-code cleanup keeps its TestVec3/TestEnum/Point fixtures live through assertions instead of `#[allow(dead_code)]`.
- `plugin_runtime.rs` owns backend family resolution, hot reload slot lifecycle, plugin discovery/loading, package export calls, unavailable backend behavior, host context propagation, and ZR VM project package discovery.
- `module_surface.rs` owns builtin gameplay host module registration, script module runtime/manager wiring, protocol type placement, and script VM folder layout guards.
- `support.rs` owns shared fixtures: recording backend/family, package builders, host context construction, temporary plugin packages, and ZR VM project fixtures.
- `lifecycle_failures.rs` remains the fallback lifecycle failure-path owner.

## Runtime 15 Status

Runtime 15 M3 script VM test folder split status: `runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result`.

The split reduced `script/vm/tests.rs` from 1456 lines to 41 lines. The largest child owner is `script/vm/tests/reflection_docs.rs` at 324 lines, and the 32 script VM tests remain below the Runtime 15 800-line test-owner budget.

`runtime_15_script_vm_tests_are_folder_backed` verifies the parent/child module layout, representative moved-test anchors, preserved test count, owner line budgets, and synchronized status anchors across Runtime 15, the runtime index, the structure convention, review findings, this document, and status-output expectations.

Runtime 15 F12 script reflection macro fixture dead-code cleanup status: `runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred`. `runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code` verifies `reflection_docs.rs` has no `#[allow(dead_code)]`, that TestVec3 fields, TestEnum::A, and the nested Point fixture have real read sites, and that Runtime 15/status mirror anchors stay synchronized.

## Validation

Scoped rustfmt has passed for the split files. The focused guard and core-min Cargo check timed out during cold build without a compile/test result; current status records timeout-no-result rather than acceptance.
