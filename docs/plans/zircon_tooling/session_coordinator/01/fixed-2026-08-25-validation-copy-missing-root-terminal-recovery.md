---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-25
summary_slug: validation-copy-missing-root-terminal-recovery
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_workspace_copy.WorkspaceCopyTests.test_periodic_recovery_rechecks_root_absence_after_running_lock -v
  - python -B -m unittest tools.session_coordinator.tests.test_workspace_copy -v -k recovery
resolved_at: 2026-08-25
---

# validation-copy-missing-root-terminal-recovery: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：tools/session_coordinator/workspace_copy.py::WorkspaceCopyService.recover_interrupted_jobs
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`tools/session_coordinator/workspace_copy.py::WorkspaceCopyService.recover_interrupted_jobs` — Production copy 11b4cd9ccb8142efa4885142d0de25a6 remains failed after its managed job_root disappeared, so cleanup.validation_copy_overlap_denied repeats for F:\cargo-targets\verify.

## 最低共享层根因

Periodic recovery retries cleanup_pending rows but never converges failed or materialized rows whose direct managed job_root is already absent.

## 架构修复验收

- Missing failed and materialized roots converge to removed with removed_at and durable per-copy evidence.
- Existing, running, planned, cleanup_pending, and locally reserved copies remain protected.
- Filesystem discovery occurs before a bounded CAS write transaction and does not weaken overlap cleanup protection.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Periodic recovery only retried cleanup_pending rows; failed and materialized validation-copy rows whose direct managed job root had already disappeared stayed non-removed forever and permanently protected parent verify roots from artifact cleanup.
- 架构修复：Discover absent direct managed roots outside the write transaction, then acquire the global mutation gate before `_running_lock`, snapshot the fresh local run reservation set, recheck filesystem absence, and finally CAS only idle failed/materialized rows inside the database transaction. The large `WorkspaceCopyService` retains only orchestration; `ValidationCopyTerminalLifecycle` owns this bounded `mutation gate -> running lock -> filesystem recheck -> database` terminal-recovery boundary, durable `removed_at`, and per-copy `validation_copy.missing_root_recovered` event without weakening overlap guards.
- 验证：RED proved missing failed/materialized roots returned (0,0), a reservation acquired after scanning was incorrectly removed, and a root recreated while waiting for `_running_lock` was still marked removed. The first post-extraction review found that acquiring the global mutation gate inside `_running_lock` inverted the foreground maintenance order and reopened the root-recreation window. A new gate-entry regression reproduced that defect with `recovered=1`; GREEN now enforces `mutation gate -> running lock -> database`, rechecks both the local reservation set and root absence under both locks, and includes a direct acquisition-order assertion. The focused recovery suite passes 13/13 in 158.836 seconds. The terminal-lifecycle module's existing status/stream suite passes 8/8 in 100.464 seconds; `py_compile` and `git diff --check` pass. Final independent re-review reported Critical 0, Important 0, Minor 0 and marked the exact worktree ready to merge. The earlier implementation had already passed managed ticket 4af7a6c358e247f0a31eee66faead640 from copy afe4ae75bf86465381baf558c6abc371, manifest 19551f3622cd10fc7f236be6d68c08bf916cf326595f57c783bac82b649f2f6b, exit 0; post-race managed ticket 92ab5bc5c79f4dd4b272739e53f09842 remains queued behind pre-existing FIFO work and is not claimed as passed here. Under schema-67 successor `ef11005c670e455b922037aca5b08dce`, the production recovery path durably converged 9 additional missing roots between 14:45:33Z and 15:06:38Z; current durable status is 1,569 removed, 21 materialized, and one planned copy. Full WorkspaceCopy suite passed 68/69 under load; its only fixed-2-second async ack timing failure passed on immediate isolated replay. Single-file handoff validation passed. Production read-only classification found 642 absent, 21 present, 0 invalid managed roots before recovery.
- 回传：Missing validation-copy roots now converge durably without deleting evidence or racing live/reserved runs; existing copies and all overlap protections remain intact.
