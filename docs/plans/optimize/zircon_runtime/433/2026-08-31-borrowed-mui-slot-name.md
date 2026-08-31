---
title: Runtime433 Borrowed MUI Slot Name
category: zircon_runtime
report_id: Runtime433-borrowed-mui-slot-name-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime433 Borrowed MUI Slot Name

MUI template slot-name lookup now returns a trimmed string slice borrowed from the node's TOML
value. Read-only layout and style classifiers no longer allocate and copy the same slot name each
time they inspect a child node.

Slot-attribute precedence, whitespace trimming, and empty-value rejection are unchanged. The
mutating slot-props path materializes an owned name only while it holds a mutable node borrow.
Regression tests verify value precedence and pointer identity with the stored TOML text.

The ignored Windows Release benchmark emits `RUNTIME433_BORROWED_MUI_SLOT_NAME_BENCH_V1` over 17
alternating paired samples, each performing 16 passes over 2,048 long slot names, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime433 is prepared with Editor361 under request
`runtime433-editor361-performance-batch-20260831dy-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
