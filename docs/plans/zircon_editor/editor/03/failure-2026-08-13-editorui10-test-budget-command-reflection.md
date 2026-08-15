---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-command-reflection
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_editor/src/tests/editing/reflected_command.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editing --locked
---

# Editor03: reflected-command test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：reflected command contract 属于 Editor03 command/transaction 领域，不能并入 Editor07 UI asset 测试切片。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/tests/editing/reflected_command.rs` 为 870 行。该 command reflection 覆盖属于
Editor03 command/transaction 领域，必须移出 flat owner 以解除 zero-tolerance gate RED。

## 最低共享层根因

reflected command 的 command shape、fixture 与 execution/assertion 场景在一个 flat 测试文件持续增长，
尚未按具体 command contract 建立 folder-backed 边界。

## 架构修复验收

- 依 command reflection 行为拆成 folder-backed modules，薄 `mod.rs` 只挂载，所有测试文件不超过 800 行。
- 保留 command/transaction 语义、fixture 和断言覆盖；共享 helper 不能被复制。
- 不得留下旧 flat file、`#[path]` compatibility mount、exemption 或无关 domain 迁移。
- 重审计不再报告该路径；全局 owner 清零后受管 structure gate 才能 GREEN。

## 禁止临时方案

- 不得提高预算、删除 reflected-command 覆盖或将其转嫁给 Editor07 UI asset tests。

## 修复结果与回传

Open state: `待修复`。本 handoff 仅转移拆分责任，未编辑业务测试。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 command reflection test-budget handoff | `open` | 从准确 48/0 审计隔离 870 行 reflected-command owner。 | 取得源码 lease 后按 command contract folder-backed 拆分，受管 editing 回归和结构审计复验。 |
