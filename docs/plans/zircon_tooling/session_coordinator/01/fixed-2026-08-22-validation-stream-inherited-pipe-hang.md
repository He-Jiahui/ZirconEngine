---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-19
summary_slug: validation-stream-inherited-pipe-hang
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/tests/test_workspace_copy_terminal_status.py
  - tools/session_coordinator/workspace_copy_terminal.py
resolved_at: 2026-08-22
---


# validation-stream-inherited-pipe-hang: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Validation-copy terminal stream collector
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Validation-copy terminal stream collector` — Use a validation root process that exits while a descendant keeps inherited stdout or stderr open; ValidationCopyTerminalLifecycle.collect blocks forever in reader.join().

## 最低共享层根因

Terminal collection waits for unbounded pipe EOF after the root process exits and has no reader deadline or typed truncated-capture terminal path.

## 架构修复验收

- Terminal output is drained without waiting for inherited pipe EOF after root exit and `after_root_exit` cleanup.
- Windows blocking-I/O cancellation is a bounded fallback; closing a buffered `TextIOWrapper` from another thread must not reintroduce an unbounded lock wait.
- Normal large dual-stream and invalid-UTF8 terminal evidence remains bounded and durable.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Terminal collection waited for inherited stdout/stderr EOF after the validation root exited, so a surviving descendant could hold the lane indefinitely.
- 架构修复：Use nonblocking pipe reads that drain available kernel bytes and terminate after root cleanup without waiting for descendant EOF; retain bounded Windows cancellation only as fallback and preserve cleanup errors.
- 验证：Immutable managed ticket f219149610914159a16dcbaf74486ae2 passed 8 terminal-lifecycle and 61 workspace-copy tests; its manifest 923d246ca3dab29f3864c9247743411c9cf9fac7a367bec053394c293fb2ce9b still exactly matches HEAD commit 08094b9b9e17f6c80372e15c17b01204038b305b.
- 回传：Inherited-pipe terminal collection is bounded, durable, and integrated at current HEAD; exact managed evidence remains byte-identical.
