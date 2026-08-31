---
title: Runtime440 Dependency Manifest Capacity
category: zircon_runtime
report_id: Runtime440-dependency-manifest-capacity-2026-08-31
date: 2026-08-31
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime440 Dependency Manifest Capacity

Compiled UI dependency projection now reserves the maximum possible output length before walking
the ordered import map. The former `filter_map(...).collect()` path had a zero lower size hint and
could repeatedly grow the result allocation even though output cannot exceed the smaller of the
import and fingerprint map lengths.

Dependency ordering, fingerprint filtering, asset metadata, and empty-input behavior remain
unchanged. Regression coverage requires the bounded capacity expression and explicit push path,
and rejects restoration of the unreserved `filter_map` collection.

The ignored Windows Release benchmark emits `RUNTIME440_DEPENDENCY_MANIFEST_CAPACITY_BENCH_V1`
over 17 alternating paired samples. Each sample builds 64 projections of 2,048 128-byte dependency
rows. The legacy model starts with capacity zero; the optimized model reserves all 2,048 rows. The
gate requires `optimized_p95_ns <= legacy_p95_ns * 0.80`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after both gates pass.

## Current batched validation handoff (2026-08-31)

Runtime440 is prepared with Editor368 under request
`runtime440-editor368-performance-batch-20260831ef-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
