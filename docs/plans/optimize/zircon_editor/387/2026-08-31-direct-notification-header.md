---
title: Editor387 Direct Notification Header
category: zircon_editor
report_id: Editor387-direct-notification-header-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor387 Direct Notification Header

Retained notification-center painting now writes unread and overflow counters directly into one
exact-capacity header string. The former header projection invoked generic formatting on every
paint when either generation counter was non-zero.

Custom/default titles, zero-counter elision, parentheses, plus sign, `omitted` suffix, and the full
`usize` counter range remain byte-for-byte unchanged. Regression coverage compares every counter
combination and boundary values with the former formatter.

The ignored Windows Release benchmark emits `EDITOR387_DIRECT_NOTIFICATION_HEADER_BENCH_V1` over
17 alternating paired samples, each constructing 262,144 headers with both counters populated.
The optimized path converts each counter once and writes both into one exact-capacity allocation.
The gate requires `optimized_p95_ns <= legacy_p95_ns * 0.75` (at least 25% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor387 is prepared with Runtime457 under request
`runtime457-editor387-performance-batch-20260831ey-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
