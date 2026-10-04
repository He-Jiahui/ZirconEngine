---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-circular-progress-hash-arc-cache.md
related_records:
  - docs/plans/astra/features/editor/30-recent-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/entry.rs
tests:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/cache/hash_arc_tests.rs
---

# Editor974 Editor01 circular-progress HashMap + Arc cache

The circular-progress raster cache now uses a `HashMap` for direct key lookup,
an access-generation field for LRU promotion, and `Arc<[u8]>` for immutable
pixel sharing. The 128-entry and 16 MiB limits, duplicate replacement,
generation rebasing, and least-recent eviction behavior remain unchanged.

## Local grouped evidence

The current-source Editor library binary was run once with the shared
`optimization_batch_20260826b` selector. The three Circular Progress owners in
that invocation all passed: LRU/generation behavior, no-copy/hash source
contract, and the ignored marker. The exact marker was:

`EDITOR01_CIRCULAR_PROGRESS_HASH_ARC_CACHE_BENCH_V1 entries=128 hits_per_frame=128 payload_bytes=65536 samples=17 sample_order=alternating legacy_key_comparisons=16384 optimized_hash_lookups=128 legacy_pixel_copy_bytes=8388608 optimized_pixel_copy_bytes=0 deterministic_lookup_work_reduction_percent=99.2188 legacy_p50_ns=2025700 optimized_p50_ns=155100 legacy_p95_ns=2695900 optimized_p95_ns=176900`

The local debug P95 reduction is `93.44%`, clearing the plan's `80%` target;
deterministic lookup work is reduced by `99.2188%` and hit-path pixel copying
by `100%`. The surrounding broad selector was not promoted because unrelated
owners had noisy debug timing/fixture failures; this record claims only the
three Circular Progress owner results above. Managed Release and product-scale
validation remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/cache.rs` | `4764FEC5242A154476D0FC8BE611D918E2AF6CEDB36B51B15CEA35DD3BC988A0` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback/circular_progress/cache/hash_arc_tests.rs` | `D8637E38559D64B8861618DE51769C9DEE0526D15867DC44A54D3C1CB03352EC` |
