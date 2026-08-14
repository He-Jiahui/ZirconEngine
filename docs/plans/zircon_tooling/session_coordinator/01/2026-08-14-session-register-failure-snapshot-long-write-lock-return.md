---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-14
summary_slug: session-register-failure-snapshot-long-write-lock
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-14-session-register-failure-snapshot-long-write-lock.md
---

# session-register-failure-snapshot-long-write-lock 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-14-session-register-failure-snapshot-long-write-lock.md](fixed-2026-08-14-session-register-failure-snapshot-long-write-lock.md)
- 摘要：Session registration no longer holds the database writer during failure snapshot parsing; immutable drift fails closed, duplicate requests skip preparation/import, and maintenance DB-busy diagnostics no longer kill recovery.
