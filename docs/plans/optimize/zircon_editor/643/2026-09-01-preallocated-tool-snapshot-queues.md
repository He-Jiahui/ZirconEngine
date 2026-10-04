---
title: Editor643 Preallocated Tool Snapshot Queues
category: zircon_editor
report_id: Editor643-preallocated-tool-snapshot-queues-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor643 Preallocated Tool Snapshot Queues

Tool scheduler snapshot construction now reserves each resource's queued-request projection from
the resource queue depth before filtering canonical request handles. The queue already supplies a
hard upper bound, so the change removes geometric growth without guessing or changing holder,
resource, or request ordering. Missing request IDs remain filtered exactly as before.

The source regression requires the bounded reservation and boxed-slice handoff. The ignored
Windows Release benchmark emits `EDITOR643_PREALLOCATED_TOOL_SNAPSHOT_QUEUE_BENCH_V1` over 17
alternating sample pairs with 65,536 queued requests. The gate requires preallocated P95 to be at
most 85% of the unreserved projection P95.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, optimization closeout, and WeCom outcome are
recorded only after coordinator completion.
