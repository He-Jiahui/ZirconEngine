---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: goal-closeout-counts-terminal-failed-intents
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/tests/test_workflow_commit.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit -v
  - powershell -File tools/zircon-session.ps1 -Json milestone close-goal --session-id runtime-text01-fr-m2-closeout-20260714 --run-id b5fedc3825764dc79b3c785291a40910
---

# Tooling01: Goal closeout counts terminal failed commit intents as unreconciled

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 FR-M3 post-commit Goal closeout
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Text01 M3 已由协调器成功提交，最低共享根因位于通用 workflow Goal closeout 的 commit-intent reconciliation 判定。

## 失败现象与复现证据

Text01 M3 commit `1c3305b172763e56a81aac886fe8c860d28d20ec` succeeded with an exact 25-file manifest and successful WeCom notification. The subsequent `milestone close-goal` failed with `workflow_goal_commit_reconciliation_pending` and `count: 7`.

All seven counted rows are terminal `workflow_commit_intents.status = 'failed'` records from earlier pre-ref-update attempts. Each has `commit_sha = NULL` and `error_text = 'finalize failed before ref update'`. The accepted M2 and M3 intents are already `reconciled` with real commit SHAs.

## 最低共享层根因

`MilestoneWorkflowService.close_goal()` counts every intent whose status is not `reconciled`:

```sql
SELECT COUNT(*) FROM workflow_commit_intents
WHERE run_id=? AND status <> 'reconciled'
```

This conflates terminal failed attempts, which provably never updated the ref, with genuinely ambiguous `prepared` or post-ref `committed` intents that still require recovery reconciliation.

## 架构修复验收

- Goal closeout treats terminal `failed` intents as closed history and continues to block genuinely nonterminal or post-ref unreconciled intents.
- Add focused coverage with multiple failed historical intents plus one reconciled successful milestone commit; closeout must not require destructive database cleanup.
- Preserve recovery behavior for `prepared` and `committed` intents whose ref/baseline outcome may be ambiguous.
- Rerun the original Text01 `milestone close-goal`; it must complete without rewriting Git history or deleting audit rows.

## 禁止临时方案

- Do not delete or update historical failed intent rows by hand.
- Do not mark failed intents `reconciled` without a commit SHA.
- Do not bypass Goal closeout, weaken milestone completion checks, or special-case Text01.

## 修复结果与回传

Open state: `待来源 Session 收敛 owned scope`；不得将它误报为 Goal 已关闭。

2026-07-15 的受控验证已确认低层 intent 判定已修复：

- `test_goal_closeout_ignores_terminal_failed_commit_intents` 与
  `test_goal_closeout_keeps_prepared_commit_intent_pending` 均通过。前者保留
  `failed`/无 SHA 的审计行并允许 closeout；后者仍拒绝 `prepared` intent。
- 原始 Text01 命令现在不再返回
  `workflow_goal_commit_reconciliation_pending`，而返回
  `workflow_goal_owned_scope_dirty`，其中列出来源 Session 仍属自己的未提交
  文本、源码、测试、文档和截图路径。

因此历史 terminal failed intents 已不再是当前阻塞根因，但该 artifact 不能在
来源 owned scope 仍 dirty 时回传为 `fixed-*`：来源执行者必须先通过受管精确
提交收敛这些路径，再重跑同一 `milestone close-goal` 命令。不得以忽略
owned-scope gate、删除历史 intent 或手工修改数据库替代该收敛。
