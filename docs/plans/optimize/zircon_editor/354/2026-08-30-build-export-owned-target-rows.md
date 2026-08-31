---
title: Editor354 Build Export Owned Target Rows
category: zircon_editor
report_id: Editor354-build-export-owned-target-rows-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor354 Build Export Owned Target Rows

Build-export pane projection now adopts the owned target-row vector when template projection is
empty and otherwise appends it directly into the final node buffer. Wizard behavior, row ordering,
and the target row projection contract remain unchanged.

The previous path extended the final node vector through an iterator over the temporary target-row
buffer. The new path transfers the target-row allocation outright on the empty-template fallback
and uses `Vec::append` when template nodes exist.

The ignored Windows Release benchmark emits
`EDITOR354_BUILD_EXPORT_OWNED_TARGET_ROWS_BENCH_V1` over 17 alternating paired samples, each
merging 8,192 batches of 1,024 owned rows, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor354 is prepared with Runtime426 under request
`runtime426-editor354-performance-batch-20260830dr-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
