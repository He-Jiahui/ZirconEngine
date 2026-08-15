---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-ui-assets
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
related_code:
  - zircon_editor/src/tests/ui/assets_activity/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib ui --locked
---

# EditorUI05: UI asset management test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：assets activity/bootstrap 和 template assets 是 EditorUI05 UI asset management 责任，不属于 asset-browser、reference 或 MUI component 计划。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`tests/ui/assets_activity/bootstrap_assets.rs` 为 821 行，`tests/ui/boundary/template_assets.rs` 为 1560 行。
它们是 UI asset bootstrap/template management 行为 owner，未拆分使全局 structure gate 继续 RED。

## 最低共享层根因

asset bootstrap/activity 和 template asset contract 场景累积在 flat 测试文件，未将 bootstrap、asset lifecycle、
template fixture 和 contract behavior 建立为 folder-backed 单一 owner。

## 架构修复验收

- 将两项按 UI asset management 行为迁移到 folder-backed tests，薄 `mod.rs` 挂载，测试文件不超过 800 行。
- 保留 bootstrap、activity、template asset 与 fixture/assertion 语义；共享 helper 唯一归属。
- 不得留下 flat compatibility test、`#[path]` mount、duplicate tree 或 budget exemption。
- 重审计不再报告这两项；全局 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 UI asset 覆盖，或把 asset-browser/reference/MUI 责任混入此切片。

## 修复结果与回传

Open state: `待修复`。本 handoff 只转移拆分责任，未修改 UI asset 测试。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 UI asset management test-budget handoff | `open` | 从准确 48/0 审计隔离 821 行 bootstrap 与 1560 行 template asset owner。 | 取得源码 lease 后按 asset management 行为 folder-backed 拆分，受管 UI 回归和结构审计复验。 |
