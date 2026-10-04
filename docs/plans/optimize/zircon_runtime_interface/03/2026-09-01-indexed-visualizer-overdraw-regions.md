record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer/overdraw.rs
related_tests:
  - tools/tests/test_runtime_interface03_visualizer_overlay_projection_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/visualizer/overdraw.rs::runtime_interface03_batch6_visualizer_overdraw_frame_index_release_benchmark
---

# Indexed visualizer overdraw regions

## Scope

The visualizer previously projected and sorted the complete node membership for every intersecting
paint pair before checking whether that intersection frame was already represented. Dense overlap
therefore repeated an O(n) member scan and allocation for each duplicate pair, approaching O(n
cubed) diagnostic work.

Overdraw projection now admits each intersection frame through a compact bit-key candidate index
before computing membership. Candidate matches still use complete `UiFrame` equality, and zero
components are canonicalized only for the bucket key so `-0.0` and `0.0` retain their Rust equality
behavior. First-seen region order, sorted/deduplicated node IDs, heat, clipping, and opacity gates
remain unchanged.

The overdraw implementation and its behavior/benchmark coverage moved into the dedicated
`ui/surface/render/visualizer/overdraw.rs` module, reducing responsibility in the parent visualizer.

## Verification

- TDD RED: the focused static contract failed because duplicate detection followed full member
  projection and the overdraw module did not exist.
- Focused overlay projection static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `54/54`
  passed.
- Rust behavior tests compare the complete indexed output with the former linear algorithm and
  cover signed-zero frame equality.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2678`;
- snapshot request: `b7edc0edd6834b888399b00dd59e75d8`;
- attribution request: `2fb64853619144448b4cc5a9293a687b`;
- submit request: `fb236988a1764582b879526f8b3347d2`;
- validation ticket: `3ecadd9b97a545f4bfa21be7e15e9dc3`;
- source manifest: `61d334778522339be798ce67bf65f1713052aecf9fd07ff2dda2fddac5c3085b`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch6 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark projects 192 fully overlapping visible elements over 11 alternating
samples, comparing post-membership linear duplicate detection with pre-membership indexed
admission. The P95 gate requires at least 20% improvement. Terminal P50/P95 nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.
