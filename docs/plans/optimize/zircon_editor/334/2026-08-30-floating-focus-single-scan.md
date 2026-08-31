---
title: Editor334 Floating Focus Single Scan
category: zircon_editor
report_id: Editor334-floating-focus-single-scan-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor334 Floating Focus Single Scan

Floating-window focus resolution now scans tabs once: it returns the focused instance
immediately, remembers the first active tab while searching, and falls back to the first tab only
when neither exists. The existing focused > active > first priority and first-active stability are
preserved while the missing-focused worst case drops from two full scans to one.

The ignored Windows Release benchmark emits `EDITOR334_FLOATING_FOCUS_SINGLE_SCAN_BENCH_V1` over
17 paired samples with 1,024 tabs, a missing focused target, and the active target last, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor334 is submitted with Runtime388 under request
`runtime388-editor334-performance-batch-20260830cl-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
