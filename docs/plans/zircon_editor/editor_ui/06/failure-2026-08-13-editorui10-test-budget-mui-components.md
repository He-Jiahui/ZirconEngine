---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-mui-components
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
related_code:
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets/mod.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets/support.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets/contracts.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/runtime_fixtures.rs
  - tools/tests/test_editor_test_infrastructure_performance_contract.py
  - tools/tests/test_zui_static_suffix_convergence.py
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib ui --locked
---

# EditorUI06: MUI component test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 交接原因：material surface、component lab、metadata contract 与 adapter 回归属于 MUI component library；
  ZUI governance owner 另交 EditorUI11，不能混入本切片。

## 失败现象与复现证据

结构审计当前报告 7 个超限 MUI component owner，test-budget exemptions 为 0：

- `tests/ui/boundary/global_material_surface_assets.rs`：832 行。
- `tests/ui/boundary/material_component_lab/{feedback,inventory,lab_theme,shell}.rs`：803、945、947、907 行。
- `tests/ui/boundary/material_meta_component_contracts.rs`：983 行。
- `tests/ui/component_adapter.rs`：966 行。

## 最低共享层根因

component library 的 material fixture、lab 交互、metadata 与 adapter contract 随功能追加到 flat 测试文件，
没有按 component/contract/fixture 划分 folder-backed owner，导致 800 行结构门禁持续 RED。

## 架构修复验收

- 每项按单一 component/contract 行为迁移为 folder-backed 测试模块，薄 `mod.rs` 只声明子模块；每个测试文件
  小于等于 800 行。
- 保留 MUI material、feedback、inventory、theme、shell、metadata 和 adapter 的完整行为/fixture 覆盖；共享 helper
  必须有唯一 owner。
- 不得留下旧 flat test、`#[path]` mount、compatibility export 或 duplicate test tree。
- 重新审计时不再报告本 7 项；全局所有 owner 清零后才允许受管 structure gate GREEN。

## 禁止临时方案

- 不得提高预算、使用 blanket exemption 或删除/弱化 MUI 行为测试。
- 不得把 ZUI governance 或 workbench/retained-host 测试吸收进此 MUI handoff。

## 修复结果与回传

Open state: `待修复`。本文件仅交接责任和复现证据，未移动 MUI 测试源码，不能作为 fixed return。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 MUI test-budget handoff | `open` | 从准确 48/0 审计中隔离 7 个 MUI component owner。 | 取得源码 exact lease 后按 component/contract folder-backed 拆分，受管回归与结构审计复验。 |

## 2026-09-06 current-source folder-backed migration

The seven original flat paths were rechecked against the current source before editing. The
`feedback`, `inventory`, `lab_theme`, `shell`, `material_meta_component_contracts`, and
`component_adapter` owners already exist as folder-backed modules; their old flat paths remain
in this record as original failure evidence and are not recreated. The remaining oversized
owner, `global_material_surface_assets.rs`, was split into:

- `zircon_editor/src/tests/ui/boundary/global_material_surface_assets/mod.rs` (module wiring);
- `zircon_editor/src/tests/ui/boundary/global_material_surface_assets/support.rs` (shared loader
  and traversal helpers, 622 lines);
- `zircon_editor/src/tests/ui/boundary/global_material_surface_assets/contracts.rs` (five
  conformance tests, 231 lines).

The folder keeps the same module declaration and test behavior. The three static references to
the old path were updated to the folder owner, and the suffix/infrastructure contracts now read
the support and contract files together where they inspect collector markers. Current evidence:

- `python -m unittest tools.tests.test_editor_test_infrastructure_performance_contract` — 11/11
  passed;
- `python -m unittest tools.tests.test_zui_static_suffix_convergence` — 18/18 passed;
- `rustfmt --edition 2021 --check` for both Rust owner files — passed;
- `audit_editor_structure.py` reports 22 oversized test owners, down from 23, and no banned
  module or UI boundary violation.

The original `cargo test -p zircon_editor --lib ui --locked` gate remains required and has not
been claimed by this static evidence. The current source split therefore remains open until the
coordinator-managed Cargo gate and the complete EditorUI06 review/return workflow finish.

### 2026-09-06 current-source Cargo admission

- Snapshot: `2795`
- Request: `failure-roll-01a07160-editorui06-ui-cargo-20260906-r2`
- Command: `cargo test -p zircon_editor --lib ui --locked`
- Source manifest: the two focused Python guards plus the folder-backed `mod.rs`, `support.rs`,
  `contracts.rs`, and `template_assets.rs` files from snapshot 2795.
- Admission: rejected before ticket/job creation with
  `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`.

No Cargo process or selected test ran. This receipt is retained as an external validation blocker;
the failure remains open and no fixed return, commit, or WeCom notification is claimed.

### 2026-09-06 module-wiring correction

The migration review found that declaring `mod contracts` inside `support.rs`
would require a `support/contracts.rs` child. Both existing sibling files are now
declared by `global_material_surface_assets/mod.rs`; the contract imports only its
required helpers through `pub(super)` support visibility. Recursive Rustfmt passed,
and the infrastructure/suffix suites passed again as part of the 31-test local batch.
EditorUI05 moved the direct consumer into `template_assets/runtime_fixtures.rs`
while preserving the prior EditorUI06 path update. Snapshot 2795 predates this
correction and is not current-source acceptance evidence. Managed Cargo remains
pending with the external `zr_vm` blocker skipped as requested.

### 2026-09-06 Editor07 replay-path consumer update

The shared suffix guard now consumes the canonical
`zircon_editor/src/tests/editing/ui_asset_replay/` directory and rejects an empty
editing test-source cohort. Its exact editing-suffix test executed locally with
1 passed, 0 failed in 0.020 seconds after Editor07 preserved all 21 replay test
bodies and nine TOML fixtures. This Python file remains owned by the same UI06
Session; Editor07 does not absorb it into its validation overlay or commit scope.
This is static-source evidence only, with original managed Rust gates pending.

### 2026-09-06 UI11 governance-path consumer update

The infrastructure guard now recursively scans the canonical governance directory,
requires a nonempty Rust cohort, and excludes only its unique root support owner.
It no longer reads the deleted flat governance root or misses nested primitive
tests. The complete local infrastructure suite executed 11 tests with 0 failures
in 0.034 seconds. This Python file remains in the existing UI06 Session; UI11
keeps its Rust migration and Web consumer in its own exact snapshot. Managed
Rust acceptance remains pending, with zr_vm skipped as requested.
