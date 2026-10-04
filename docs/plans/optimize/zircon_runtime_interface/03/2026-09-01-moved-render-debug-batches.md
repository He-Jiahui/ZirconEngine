record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/debug/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_render_debug_cache_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/debug/performance_tests.rs::runtime_interface03_batch8_moved_render_debug_batches_release_benchmark
---

# Moved render-debug batches

## Scope

`UiRenderDebugSnapshot::from_paint_elements` previously retained the final batch plan only long
enough to deep-clone every batch key, source-index vector, and node-ID vector into the debug
snapshot. The plan is private to this constructor and has no consumer after the cache, visualizer,
parity, and scalar statistics have been projected.

The constructor now completes all borrowed projections first, then consumes `plan.batches` and
moves each owned payload into the debug entry. Debug ordering, ranges, split reasons, cache data,
parity data, visualizer data, and the serialized public contract are unchanged.

## Verification

- TDD RED: the focused contract found the `.iter()` projection and three deep clones.
- Focused render debug/cache static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `44/44`
  passed (`23 + 9 + 12`).
- Rust behavior coverage compares the complete moved projection with the prior cloned projection.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2702`;
- snapshot request: `b1062211b55a4e0ba69dc283c9a505b8`;
- attribution request: `04d054b019854a2f8edcbadaf13cd23f`;
- submit request: `e97c7f16f6c2457092a36b5b6e9789b6`;
- coordinator request: `8cc11b631c96486abd1b03e05812ba7f`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch8 -- --include-ignored --nocapture`;
- submitted state: accepted; the response timed out after submission and ticket reconciliation is
  intentionally deferred while independent work continues.

## Performance contract

The ignored release benchmark projects 4,096 batches carrying owned resource IDs, draw effects,
source indices, and node IDs over 11 alternating samples. It compares the prior deep-cloned
projection with ownership transfer and requires at least 20% P95 improvement. Terminal P50/P95
nanosecond values must come from the managed Windows receipt before integration, push, or WeCom
reporting.
