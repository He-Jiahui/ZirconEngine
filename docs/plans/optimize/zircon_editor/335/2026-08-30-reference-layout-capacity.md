---
title: Editor335 Reference Layout Capacity
category: zircon_editor
report_id: Editor335-reference-layout-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor335 Reference Layout Capacity

Asset-reference pointer layout projection now reserves the exact snapshot length before cloning
UUID entries. Reference order, project-asset flags, pane sizing, and empty-layout behavior remain
unchanged while growth reallocations are removed from repeated retained-host projection.

The ignored Windows Release benchmark emits `EDITOR335_REFERENCE_LAYOUT_CAPACITY_BENCH_V1` over 17
alternating paired samples with 512 references per sample, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor335 is submitted with Runtime391 under request
`runtime391-editor335-performance-batch-20260830cp-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
