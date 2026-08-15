---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-ui-asset-reference
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/10
related_code:
  - zircon_editor/src/tests/editing/ui_asset/reference_and_promotion.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editing --locked
---

# Editor10: UI asset reference and promotion test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：UI asset reference/promotion 是 Editor10 project and asset reference 管理责任，应与 Editor07 authoring/replay 分离。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/tests/editing/ui_asset/reference_and_promotion.rs` 为 806 行。它覆盖 UI asset reference/promotion，
属于 Editor10 project/asset reference 管理，必须拆分以解除全局 structure gate RED。

## 最低共享层根因

reference resolution、promotion 和相关 fixture/assertion 被累积到 flat 测试 owner，没有按 reference/promotion
行为建立 folder-backed 边界。

## 架构修复验收

- 按 reference 与 promotion 行为拆为 folder-backed tests，薄 `mod.rs` 挂载，测试文件不超过 800 行。
- 保留 project/asset reference 解析、promotion 和错误路径语义；共享 fixture 唯一归属。
- 不得保留 old flat test、`#[path]` shim、duplicate tree 或 budget exemption。
- 重审计不再报告该路径；全局 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 reference/promotion 覆盖，或将其回归混入 Editor07 的 authoring/replay 责任。

## 修复结果与回传

Open state: `待修复`。本 handoff 只转移拆分责任，未修改业务测试。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 UI asset reference test-budget handoff | `open` | 从准确 48/0 审计隔离 806 行 reference/promotion owner。 | 取得源码 lease 后按 reference/promotion folder-backed 拆分，受管 editing 回归和结构审计复验。 |
