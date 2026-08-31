---
title: Editor383 Single-Pass Table Offsets
category: zircon_editor
report_id: Editor383-single-pass-table-offsets-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor383 Single-Pass Table Offsets

Retained Editor table-row geometry now advances one horizontal cursor while producing the four
cell rectangles. The former bulk path asked every cell for its prefix offset independently, causing
the column widths before that cell to be traversed and summed again. The row already shares one
column allocation snapshot; this change also makes offset projection a single pass.

Individual geometry queries retain the same prefix-offset behavior used by tests, while the actual
row paint path preserves all cell frames, content offsets, action-column reservation, and narrow
layout behavior. Regression coverage compares the bulk snapshot against every individual cell and
guards the running-offset production path.

The ignored Windows Release benchmark emits `EDITOR383_SINGLE_PASS_TABLE_OFFSETS_BENCH_V1` over 17
alternating paired samples, each projecting 524,288 four-column rows. The legacy path performs a
fresh prefix sum per cell; the optimized path advances one cursor per column. The gate requires
`optimized_p95_ns <= legacy_p95_ns * 0.85` (at least 15% lower P95).

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Editor383 is prepared with Runtime453 under request
`runtime453-editor383-performance-batch-20260831eu-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
