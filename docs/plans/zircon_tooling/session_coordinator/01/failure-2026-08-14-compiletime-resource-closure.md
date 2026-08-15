---
handoff_kind: failure
status: open
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

- `validation_copies.py` now lexes tracked Rust source while excluding comments,
  quoted strings, raw strings, and character literals; it recognizes real
  `include_bytes!` and `include_str!` invocations only.
- The closure resolver handles direct literal paths and static
  `concat!(env!("CARGO_MANIFEST_DIR"), ...)` prefixes. Dynamic tails include the
  resolved tracked directory, while unresolved, missing, or repository-escaping
  paths fail closed with stable Coordinator errors.
- The planner adds tracked files under each resolved compile-time resource to the
  immutable source manifest. The new regression fixture proves that the external
  `templates/projects/renderable-empty/**` files are included and a lookalike
  macro in a comment/string is ignored.
- An existing metadata fixture now uses valid Rust string-literal syntax so the
  parser does not accept an invalid source representation.
- The production materialization and run IDs below are forward evidence for the
  fixing plan. The handoff remains open only for the Editor01 origin owner to
  accept that evidence and execute the ordinary fixed return.

## 验收状态

The compile-time include closure landed in `93049504c`, and Windows-safe bounded
Git enumeration for thousands of resource roots landed in `9b9e03755`. A real
managed copy `5e67c4a2af86451b828d732b3a116446` subsequently reached
`materialized` with immutable input manifest
`e2e860b6d87f905e72a62eaebc3a99be5cb1f82015b98dda899180fd46136c09`.
Its one validation run `6482e830a92648a8bf9f1a51d90613a3` launched Cargo and
terminated with downstream current-source compile diagnostics rather than a
closure-planning or missing-resource failure. This proves the Coordinator-owned
materialization boundary; it does not claim an Editor01 Cargo pass or perform the
origin-plan lifecycle return.

## 禁止临时方案

- Do not special-case `renderable-empty`, whitelist template folders, or add an
  Editor01-only copy overlay.
- Do not fall back to copying untracked paths, silently skip unresolved includes,
  or permit compile-time resources outside the repository root.
- Do not copy templates into a Cargo package merely to satisfy validation; that
  changes source ownership instead of repairing the generic closure contract.

## 产出记录与时间

- 2026-08-14 18:59:13 +08:00 | status: resolving | 完成：以失败测试确认 Cargo
  package-only closure遗漏编译期模板资源；实现通用 Rust include 解析与 fail-closed
  路径收集；新增回归并将既有 fixture 修正为合法 Rust。验证：受影响 2 项通过，
  `Ran 2 tests in 68.355s`; 完整 `test_validation_copies` 通过，
  `Ran 7 tests in 128.717s`。待完成：服务集成后的 managed validation-copy/Cargo
  复验、独立评审和服务提交。
- 2026-08-14 19:10:41 +08:00 | status: resolving | 完成：词法审查发现并修复
  Rust lifetime 被误作字符字面量的漏扫风险；回归覆盖 `&'static str` 前置、注释、
  普通字符串与 raw string 中的伪 include。验证：目标回归通过，
  `Ran 1 test in 7.128s`; 最终完整 `test_validation_copies` 通过，
  `Ran 7 tests in 86.629s`。待完成：服务集成后的 managed validation-copy/Cargo
  复验、独立评审和服务提交。
- 2026-08-15 13:00:00 +08:00 | status: open | 已提交：compile-time include
  closure `93049504c` 与 bounded Git batching `9b9e03755`。真实 copy
  `5e67c4a2...` materialized 后 run `6482e830...` 已启动 Cargo；剩余仅为
  Editor01 origin lifecycle return，不宣称下游产品编译通过。
