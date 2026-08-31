record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
related_tests:
  - tools/tests/test_runtime_interface03_render_visualizer_batch_index_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs::render_visualizer_batch_index_release_benchmark
---

# Render visualizer batch and cache index reuse

## Scope

`UiRenderVisualizerSnapshot::from_paint_elements_batches_cache` previously searched every
batch for every paint element and repeated cache-entry searches for each paint and batch row.
That multiplied diagnostic projection work by the number of batches and cache entries while
leaving the public visualizer DTO unchanged.

The implementation now builds one source-index-to-batch-index table, one paint cache-status
table, and one batch cache-status/reason table. Duplicate cache entries keep the existing first
match, and out-of-range entries remain ignored. Overlay generation reuses the same borrowed batch
index table for wireframe, clip, baseline, and glyph overlays.

This is a diagnostic snapshot optimization. It does not claim a GPU submission or frame-time
improvement.

## Verification

- TDD RED: the new static guard failed against the repeated batch/cache scans.
- Focused static contract after implementation: `1/1` passed.
- Batched RuntimeInterface03 contracts including accessibility, ECS, render parity, input routing,
  visualizer, and Editor palette: `13/13` passed.
- `python -m compileall` for the new guard: passed.
- Scoped `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`: passed.
- Scoped `git diff --check`: passed.
- Managed Windows Rust 1.94.1 release test and ignored benchmark: pending asynchronous coordinator
  validation; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2648`
- snapshot request: `0422f5b8dc944380a354d81c928ed58c`
- submit request: `9e3ea27d9b61416183218f83e3d3d18e`
- coordinator request: `fd96af865ee84834953996a9a4ba8964`
- validation ticket: `8891e624a82344908e89b51cca241eff`
- source manifest: `fb4b39e10977378241952da272a1a867cf17fa95f991dcb127dd9a46ad6b2a35`
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1 -- --include-ignored --nocapture`
- submitted state: `queued` (asynchronous; intentionally not polled)

## Performance contract

The ignored release benchmark compares the previous per-row linear searches with the shared
indexed tables at 4,096 paint elements and 512 batches across 11 alternating samples. The P95
gate requires the indexed path to be at least 20% faster. The terminal P50/P95 nanosecond values
must come from the managed Windows receipt and will be appended before integration.

No commit, push, or WeCom performance message is authorized by this record alone.
