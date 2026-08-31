---
title: Runtime421 Cached Canvas Placement Fields
category: zircon_runtime
report_id: Runtime421-cached-canvas-placement-fields-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime421 Cached Canvas Placement Fields

Both template and v2 surface-tree canvas-slot parsers now load the six optional placement fields
once, use those borrowed values to decide whether placement exists, and pass the same values into
the established parsers. Empty layouts still produce no placement and any single field, including
`auto_size`, still creates the default-backed placement contract.

The previous paths probed the TOML map with up to six `contains_key` calls and then repeated all six
lookups when a placement was present. Auto-size-only layouts therefore performed twelve tree
lookups; the new path performs six while preserving validation and defaulting behavior.

The ignored Windows Release benchmark emits
`RUNTIME421_CACHED_CANVAS_PLACEMENT_FIELDS_BENCH_V1` over 17 alternating paired samples, each
performing 131,072 auto-size-only parses against a 65-entry map, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime421 is prepared with Editor349 under request
`runtime421-editor349-performance-batch-20260830dm-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
