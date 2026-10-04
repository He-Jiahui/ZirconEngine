---
title: Editor637 Preallocated Pending Edit Successes
category: zircon_editor
report_id: Editor637-preallocated-pending-edit-successes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor637 Preallocated Pending Edit Successes

Pending-edit application now reserves the success vector from the smaller of the current retry plus
pending queue snapshot and the entry budget. This bounds `unlimited()` safely by actual queued work,
while the failure vector remains demand-grown for the uncommon error path.

Retry-first ordering, elapsed-time checks, panic requeue behavior, failure retention, and budget
reporting remain unchanged. The real queue regression verifies a fully successful batch uses the
bounded initial capacity.

The ignored Windows Release benchmark emits `EDITOR637_PREALLOCATED_PENDING_EDIT_SUCCESSES_BENCH_V1`
over 17 alternating sample pairs and 65,536 success projections. The gate requires preallocated P95
to be at most 85% of unreserved P95.

No direct Cargo validation was run. The coordinator owns the aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
