---
title: Runtime405 Material Record Indirect Sort
category: zircon_runtime
report_id: Runtime405-material-record-indirect-sort-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime405 Material Record Indirect Sort

Material management record sets now sort compact `(ResourceId, source index)` entries and apply the
result with an in-place permutation. This avoids repeatedly moving records that contain names,
references, and overview fields during comparison sorting. The source-index lane preserves the old
stable ordering even if malformed input repeats a resource ID, while each full record moves only as
required by the final permutation.

The ignored Windows Release benchmark emits `RUNTIME405_MATERIAL_RECORD_INDIRECT_SORT_BENCH_V1`
over 17 alternating paired samples with 65,536 populated records. Template cloning happens outside
the timed region, and the gate requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime405 is submitted with Runtime404 under request
`runtime404-runtime405-performance-batch-20260830da-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
