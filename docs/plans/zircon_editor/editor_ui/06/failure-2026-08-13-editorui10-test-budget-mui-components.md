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
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/feedback.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/inventory.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/lab_theme.rs
  - zircon_editor/src/tests/ui/boundary/material_component_lab/shell.rs
  - zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
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
