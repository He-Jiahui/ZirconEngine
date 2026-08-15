---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-ui-asset-domain
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/07
related_code:
  - zircon_editor/src/tests/editing/ui_asset/inspector.rs
  - zircon_editor/src/tests/editing/ui_asset/tree_and_undo.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/editing/ui_asset_replay.rs
  - zircon_editor/src/tests/editing/ui_asset_theme_authoring.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editing::ui_asset --locked
---

# Editor07: UI asset domain test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 交接原因：这些覆盖 UI asset inspector、tree/undo、palette drop、preview binding、replay 与 theme authoring；
  它们属于 Editor07 domain editor 语义，不属于 EditorUI10 的通用审计。asset reference/promotion 已另交 Editor10。

## 失败现象与复现证据

当前结构审计以 800 行为界、0 个 test-budget exemption，报告以下 6 个 Editor07 owner：

- `tests/editing/ui_asset/inspector.rs`：997 行。
- `tests/editing/ui_asset/tree_and_undo.rs`：829 行。
- `tests/editing/ui_asset_palette_drop.rs`：1029 行。
- `tests/editing/ui_asset_preview_binding_authoring.rs`：1798 行。
- `tests/editing/ui_asset_replay.rs`：1910 行。
- `tests/editing/ui_asset_theme_authoring.rs`：1381 行。

它们混合多个 UI asset 行为，导致通用 zero-tolerance structure gate 保持 RED。

## 最低共享层根因

domain editor 的新增回归集中附加到 flat 测试 owner，尚未将 inspector、tree/undo、palette、preview、replay
与 theme 的 fixture/operation/test suite 按行为域拆成 folder-backed 子模块。

## 架构修复验收

- 每个列举 owner 必须依行为迁移到 folder-backed 目录并留薄 `mod.rs`，每个 Rust 测试文件不超过 800 行。
- 保留 UI asset 文档、mock preview host、undo/replay 和 authoring 的原有测试语义；共享 fixture 只可进入其唯一
  domain helper，不得复制测试树。
- 不得保留旧 flat 文件、`#[path]` shim 或 compatibility re-export。
- 重跑本 domain 的受管回归后，结构审计不得再报告这 6 条；全局 48 项归零后才允许
  `structure_convention` gate GREEN。

## 禁止临时方案

- 不得提高预算、添加目录/glob/blanket exemption、删除测试或弱化 EditorUI10 zero-tolerance gate。
- 不得把 UI asset reference/promotion 责任从 Editor10 吸回，或把 UI component/MUI 覆盖混入本切片。

## 修复结果与回传

Open state: `待修复`。本记录完成前向责任交接，未修改任何 Editor07 测试源码，不能作为 fixed return。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 UI asset domain test-budget handoff | `open` | 从准确的 48/0 审计中隔离 6 个 UI asset domain owner，最大 replay owner 为 1910 行。 | 取得源码 exact lease 后按行为 folder-backed 拆分，受管 domain 测试和结构审计均需复验。 |
