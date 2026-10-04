---
title: Runtime636 Preallocated Material Selection Results
category: zircon_runtime
report_id: Runtime636-preallocated-material-selection-results-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime636 Preallocated Material Selection Results

Material management selection now reserves both result vectors from the unique request count before
partitioning records into selected and missing IDs. The previous vectors grew from zero even though
the request projection already provided a strict upper bound for both outputs.

Request order, duplicate collapse, record ownership, missing-ID reporting, and summary/index
construction remain unchanged. Saturating behavior is not needed because the capacity derives from a
materialized vector length.

The ignored Windows Release benchmark emits
`RUNTIME636_PREALLOCATED_MATERIAL_SELECTION_RESULT_BENCH_V1` over 17 alternating sample pairs with
65,536 requests. The gate requires preallocated P95 to be at most 80% of the unreserved result
partition P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime636/Editor636 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime636 is prepared with Editor636 under the shared `optimization_batch_iy_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
