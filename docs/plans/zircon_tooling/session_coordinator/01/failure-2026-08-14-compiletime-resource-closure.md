---
handoff_kind: failure
status: resolving
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
- 触发切片：Editor01 gateway current-source validation.
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

## 修复结果

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

## 验收状态

Implementation and Python regression are complete. Coordinator service-source
integration and a newly materialized managed Cargo copy remain pending; the
currently running service predates this source change, and shared validation
window coordination forbids launching a replacement Cargo request at this time.
No Editor01 Cargo pass is claimed by this record.

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
