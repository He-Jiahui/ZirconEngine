---
handoff_kind: fixed
status: fixed
created_at: 2026-08-29
summary_slug: offline-queue-process-identity-type-drift
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/offline_queue.py
  - tools/session_coordinator/processes.py
  - tools/session_coordinator/tests/test_deferred_action_client.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_deferred_action_client.DeferredActionClientTests.test_offline_spool_reuses_shared_windows_process_identity_contract -v
  - python -m unittest tools.session_coordinator.tests.test_deferred_action_client -v
resolved_at: 2026-08-30
---

# Coordinator01: offline queue process identity ctypes contract drifts

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：unoccupied failure discovery through deferred action client regression
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns both the shared Windows process identity authority and the descriptor-gap offline queue.

## 失败现象与复现证据

On Windows with Python 3.14, a process that calls `processes.process_creation_time()` before enqueueing an offline command fails in `GetProcessTimes` with `ctypes.ArgumentError`. The shared helper binds the process-wide Win32 function prototype to its private `_FileTime` pointer type, while `offline_queue.py` later passes distinct `wintypes.FILETIME` pointers. Descriptor-absent session heartbeat cannot be persisted, and a healthy successor cannot replay the command.

## 最低共享层根因

`offline_queue.py` duplicates Win32 process identity probing instead of consuming the single process identity authority in `processes.py`. Mutable ctypes function metadata therefore makes otherwise valid behavior depend on import and call order.

## 架构修复验收

- Offline queue lock descriptors obtain process creation identity only through the shared process helper.
- Unavailable or inaccessible process identity retains the existing `None`/dead-owner behavior without masking unrelated programming errors.
- The order-dependent focused regression and the complete deferred action client suite pass on Windows Python 3.14.

## 禁止临时方案

- Do not clear or weaken Win32 ctypes argument types.
- Do not isolate test processes to hide the order dependency.
- Do not suppress `ctypes.ArgumentError`, forge a creation identity, or disable stale-lock recovery.

## 修复结果与回传

- 根因：Offline queue duplicated the shared Windows GetProcessTimes probe with an incompatible ctypes FILETIME pointer type.
- 架构修复：Route offline lock identity through processes.process_creation_time and preserve unavailable-process fallback and canonical 16-digit identity formatting.
- 验证：test_deferred_action_client 15/15 including Windows identity-order regression; py_compile and scoped diff check passed.
- 回传：Coordinator offline queue process identity drift fixed and returned; replay remains independent.
