---
title: Runtime630 Preallocated Keyed I/O Shutdown Notifications
category: zircon_runtime
report_id: Runtime630-preallocated-keyed-io-shutdown-notifications-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime630 Preallocated Keyed I/O Shutdown Notifications

Bounded keyed I/O shutdown now reserves its terminal-notification batch from the suspended and
queued entry counts already held under the lane lock. The previous vector grew from zero while
shutdown retired eligible entries from both collections.

The sum is a safe upper bound because each input entry can emit at most one terminal notification;
fences and fence-pinned work may be retained without using their reserved slot. Admission closure,
ordered queue merging, terminal selection, observer invocation order, and pump scheduling remain
unchanged.

The ignored Windows Release benchmark emits `RUNTIME630_PREALLOCATED_KEYED_IO_SHUTDOWN_BENCH_V1`
over 17 alternating sample pairs with 65,536 notifications. The gate requires preallocated P95 to
be at most 80% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime630/Editor630 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime630 is prepared with Editor630 under request
`runtime630-editor630-task-shutdown-catalog-membership-performance-20260901it-v1`. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
