---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-chart-raster-arc-cache.md
related_records:
  - docs/plans/astra/features/editor/30-recent-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives/charts/raster_commands/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives/charts/raster_commands/commands.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives/charts/raster_commands/cache/arc_pixels_tests.rs
---

# Editor973 Editor01 chart raster Arc cache

The chart raster cache now retains immutable pixels as `Arc<[u8]>` and shares
the same allocation with paint commands on a stable hit. The existing ordered
cache, 128-entry limit, resident-byte budget, access promotion, duplicate
replacement, and least-recent eviction semantics remain unchanged.

## Local grouped evidence

The current-source Editor library binary was run once with the shared
`optimization_batch_20260826b` selector that also covered the neighboring
Editor01/Editor07/Editor23 contracts. The three Chart Raster Arc owners in that
invocation all passed: LRU identity, no-copy source contract, and the ignored
marker. The exact marker was:

`EDITOR01_CHART_RASTER_ARC_CACHE_BENCH_V1 entries=128 hits_per_frame=128 payload_bytes=131072 samples=17 sample_order=alternating btree_lookups=128 legacy_pixel_buffers_cloned=128 optimized_pixel_buffers_cloned=0 legacy_pixel_copy_bytes=16777216 optimized_pixel_copy_bytes=0 deterministic_pixel_copy_reduction_percent=100.0000 legacy_p50_ns=4302800 optimized_p50_ns=147300 legacy_p95_ns=7301200 optimized_p95_ns=481700`

The local debug P95 reduction is `93.40%`, clearing the plan's `90%` target.
The surrounding broad selector was not promoted because unrelated owners in
that selector had noisy debug timing/fixture failures; this record claims only
the three Chart Raster owner results above. Managed Release and product-scale
validation remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives/charts/raster_commands/cache.rs` | `FBF642E1E585097A091A9726F4A492BB61C83C48A7EAE2DE15255C38CD75975D` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives/charts/raster_commands/cache/arc_pixels_tests.rs` | `88AD9AB2966D98E8996C20C40D131E3B5E5CC009A5E8D119BF803F64C95AAEE0` |
