---
title: Runtime404 Visibility Index In-Place Sort
category: zircon_runtime
report_id: Runtime404-visibility-index-unstable-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime404 Visibility Index In-Place Sort

Visibility mesh projection now sorts its compact index vector in place with an unstable total key of
stable instance key and original mesh index. The index lane reproduces the previous stable tie order
for duplicate compatibility data while avoiding the stable sorter scratch allocation in large render
extracts.

The ignored Windows Release benchmark emits `RUNTIME404_VISIBILITY_INDEX_UNSTABLE_SORT_BENCH_V1`
over 17 alternating paired samples with 65,536 uniquely permuted keys. Input copying happens outside
the timed region, and the gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime404 is submitted with Runtime405 under request
`runtime404-runtime405-performance-batch-20260830da-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
