---
title: Runtime411 Font Variation Vector Sort
category: zircon_runtime
report_id: Runtime411-font-variation-vector-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime411 Font Variation Vector Sort

Font variation canonicalization now reserves a compact vector, stable-sorts it by OpenType tag, and
deduplicates equal tags in place. This removes one `BTreeMap` node allocation per distinct axis while
preserving tag order, last-duplicate-wins behavior, non-finite rejection, and negative-zero
normalization.

The ignored Windows Release benchmark emits `RUNTIME411_FONT_VARIATION_VECTOR_SORT_BENCH_V1` over 17
alternating paired samples with 16,384 axes, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime411 is submitted with Runtime410 under request
`runtime410-runtime411-performance-batch-20260830dd-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
