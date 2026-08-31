---
title: Runtime385 Visibility Projection Capacity
category: zircon_runtime
report_id: Runtime385-visibility-projection-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime385 Visibility Projection Capacity

Main-view visibility projection now reserves culled entity and stable-key outputs from their
source counts. Visible batch projection reserves its batch count and aligned member vectors from
known input bounds, then writes keys and entities directly instead of building a temporary tuple
vector and unzipping it. Batch order, member order, visibility filtering, empty-batch removal, and
the existing `zip` truncation behavior remain unchanged.

The ignored Windows Release benchmark emits `RUNTIME385_VISIBILITY_PROJECTION_CAPACITY_BENCH_V1`
over 17 paired samples with 64 batches and 128 members per batch, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Runtime/Editor Release
compilation, regression tests, ignored benchmarks, record finalization, manifest-only commit/push,
and one-shot WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime385 is submitted with Editor331 under request
`runtime385-editor331-performance-batch-20260830ci-v1`. Receipt, validation ticket, and source
manifest details are recorded in the session submission log after acceptance.
