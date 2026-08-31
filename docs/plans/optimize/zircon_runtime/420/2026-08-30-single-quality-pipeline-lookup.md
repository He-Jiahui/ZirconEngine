---
title: Runtime420 Single Quality Pipeline Lookup
category: zircon_runtime
report_id: Runtime420-single-quality-pipeline-lookup-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime420 Single Quality Pipeline Lookup

Quality-profile changes now resolve the effective render pipeline with one `HashMap::get`. The
same helper handles an absent pipeline, a registered active or override pipeline, and the existing
`UnknownPipeline` error without changing compilation, capability validation, or lock boundaries.

The previous override path first called `contains_key` and then immediately called `get` for the
same handle. Removing the preflight lookup halves hash-table probes in the override snapshot path
and keeps the pipeline asset clone outside the later validation lock scope.

The ignored Windows Release benchmark emits
`RUNTIME420_SINGLE_QUALITY_PIPELINE_LOOKUP_BENCH_V1` over 17 alternating paired samples, each
performing 262,144 lookups across 4,096 pipeline handles, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime420 is prepared with Editor348 under request
`runtime420-editor348-performance-batch-20260830dl-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
