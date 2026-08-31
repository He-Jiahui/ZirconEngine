---
title: Editor361 Borrowed View Counter Key
category: zircon_editor
report_id: Editor361-borrowed-view-counter-key-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor361 Borrowed View Counter Key

View instance counter restoration now probes the descriptor map with the borrowed descriptor ID.
Repeated instances of an existing view descriptor therefore update the maximum numeric suffix
without cloning and allocating the descriptor key on every call.

The first descriptor occurrence still inserts one owned key, and malformed or missing numeric
suffixes remain ignored. Regression tests cover maximum-suffix behavior and the borrow-before-own
source contract.

The ignored Windows Release benchmark emits `EDITOR361_EXISTING_VIEW_COUNTER_UPDATE_BENCH_V1`
over 17 alternating paired samples, each rebuilding 16 counters from 1,024 instances sharing a
long descriptor ID, requiring `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor361 is prepared with Runtime433 under request
`runtime433-editor361-performance-batch-20260831dy-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
