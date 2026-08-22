---
handoff_kind: fixed
status: fixed
created_at: 2026-08-14
summary_slug: compiletime-resource-closure
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/tests/test_validation_copies.py
  - zircon_runtime_interface/src/project/template_pack/embedded.rs
tests:
  - python -m unittest tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_cargo_metadata_closure_includes_local_packages_and_requires_external_descriptor tools.session_coordinator.tests.test_validation_copies.ValidationCopySourceTests.test_cargo_closure_includes_compile_time_resources_outside_package_root
  - python -m unittest tools.session_coordinator.tests.test_validation_copies
resolved_at: 2026-08-23
---


# Coordinator01: validation copy omits Rust compile-time resources outside Cargo packages

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor01 gateway current-source validation.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Editor01 can submit a source manifest, but only Coordinator01 owns the
  immutable validation-copy input closure. The failure reproduces before Rust can
  compile, so no editor-side compatibility or source workaround is permitted.

## 失败现象与复现证据

The managed `cargo check -p zircon_editor --lib` validation copy failed before
executing the editor crate because
`zircon_runtime_interface/src/project/template_pack/embedded.rs` expands
`include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),
"/../templates/projects/renderable-empty/", $path))`. The copy contained the
Cargo package root but not the tracked template directory one level outside it.

The original closure planner discovered local Cargo package roots and collected
their tracked files only. It did not inspect Rust compile-time include macros,
therefore valid repository resources referenced by `include_bytes!` or
`include_str!` could be absent from a supposedly immutable build input.

## 最低共享层根因

`CargoInputClosurePlanner.plan` defined the closure solely from Cargo metadata
and package-root Git paths. Compile-time resources are compiler inputs rather
than Cargo metadata, and can legitimately resolve outside a package root through
`env!("CARGO_MANIFEST_DIR")` plus `concat!`. This made every dependent editor or
runtime validation non-self-contained.

## 架构修复验收

- Discover real `include_bytes!` and `include_str!` compiler inputs from tracked
  Rust sources without accepting lookalikes in comments or string literals.
- Resolve literal and static `concat!(env!("CARGO_MANIFEST_DIR"), ...)` paths
  inside the repository, and fail closed for missing, dynamic, or escaping inputs.
- Enumerate thousands of resource roots through bounded deterministic Git calls
  so Windows command length cannot invalidate an otherwise complete closure.
- Materialize a real immutable validation copy and prove Cargo starts from it;
  downstream product compilation may fail, but closure planning must not.

## 修复结果与回传

- 根因：The validation-copy closure treated Cargo package files as complete compiler input and enumerated discovered compile-time resource roots in one unbounded Windows Git invocation.
- 架构修复：Generic Rust include discovery now resolves repository-bound tracked resources, fails closed for invalid inputs, and batches deterministic Git path enumeration below the Windows command budget.
- 验证：Current source test_validation_copies passed 9/9 in 44.825s; copy 5e67c4a2 materialized manifest e2e860b6 and run 6482e830 launched Cargo, failing only on downstream source debt.
- 回传：Editor01 may resume immutable-copy validation; no product source change or replay is required for the Coordinator closure.

## 禁止临时方案

- Do not special-case `renderable-empty`, whitelist template folders, or add an
  Editor01-only copy overlay.
- Do not fall back to copying untracked paths, silently skip unresolved includes,
  or permit compile-time resources outside the repository root.
- Do not copy templates into a Cargo package merely to satisfy validation; that
  changes source ownership instead of repairing the generic closure contract.
