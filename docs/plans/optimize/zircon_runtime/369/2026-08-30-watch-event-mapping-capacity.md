---
title: Runtime369 Watch Event Mapping Capacity
category: zircon_runtime
report_id: Runtime369-watch-event-mapping-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime369 Watch Event Mapping Capacity

## Scope

`map_notify_event` now routes create, modify, and remove path batches through one bounded mapper.
The mapper reserves the input path count before filtering sidecars and invalid paths, preserving
event order, rename handling, URI validation, and the existing empty-result behavior.

## Tests And Performance Gate

The source file owns two non-ignored behavior/source-contract tests and one ignored Release
benchmark under the `optimization_batch_20260830bq_` prefix. The benchmark emits
`RUNTIME369_WATCH_EVENT_MAPPING_CAPACITY_BENCH_V1`, maps 4,096 paths across 17 interleaved samples,
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

The baseline retains the prior `filter_map` collection path. The optimized path reserves the path
upper bound and pushes only validated URIs, so filtered entries do not cause repeated vector
growth. The benchmark reports exact raw samples and P95 values; it does not claim filesystem or
watcher latency outside this mapping stage.

No direct Cargo command was run. The coordinator owns the combined Runtime/Editor Release
compile, batched behavior tests, ignored benchmarks, exact P95 evidence, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom publication with measured data.

## Current batched validation handoff (2026-08-30)

Runtime369 is included in ticket `bb793f894807473ea8c78a90c6fc2d35` for request
`runtime-editor-369-371-315-317-20260830-v2`, with source manifest hash
`391c0060104af61c0806431d76bebbaf6f1d74c41c216b63aa899577269baf4c`. The batch also binds
`external_image_copy.rs` at `a1102110c7daee234ea89c1f19491a267e64f7a0e4a2882fa61e1c1c47920606`.
Cargo, performance, review, commit, push, and WeCom remain coordinator-owned and pending.

## Validation attempt (2026-08-30)

Corrected batch ticket `bb793f894807473ea8c78a90c6fc2d35` ended `failed`. The coordinator
provided no valid Cargo, performance, or commit evidence; the external validation resource was
left unchanged and no successful WeCom notification was sent.
