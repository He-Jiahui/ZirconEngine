---
record_kind: failure_return_status
status: fixed
resolved_at: 2026-08-29
summary_slug: health-validation-copy-blocker-index-scan
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
plan_link_mode: child_record_only
source_artifact: docs/plans/zircon_tooling/session_coordinator/01/failure-2026-08-29-health-validation-copy-blocker-index-scan.md
---

# health-validation-copy-blocker-index-scan 回传摘要

- 状态：`fixed`
- 回传工件：[fixed-2026-08-29-health-validation-copy-blocker-index-scan.md](fixed-2026-08-29-health-validation-copy-blocker-index-scan.md)
- 摘要：Commit `ddba694ad635f50b3298f3511f2fca735dcb0191` adds the schema69 partial covering index for active validation-copy blockers. Successor `9f8e1b1258ab47e8bb94bd08426e90a1` serves `/health` in 0.122 seconds instead of 8.889 seconds while retaining terminal copy history.
