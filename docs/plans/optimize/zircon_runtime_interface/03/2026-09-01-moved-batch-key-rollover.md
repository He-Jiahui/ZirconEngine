record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/batch/plan.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/plan/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_batch_key_construction_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/batch/plan/performance_tests.rs::runtime_interface03_batch7_moved_batch_key_rollover_release_benchmark
---

# Moved batch-key rollover

## Scope

At every batch boundary, `UiBatchPlan::from_paint_elements` previously deep-cloned the active
`UiBatchKey` into the completed batch and then replaced the active key. Resource IDs, fallback
chains, clip state, and draw-effect vectors made that clone proportional to key payload size.

The batch loop now computes the next split reason while both keys are borrowed, then replaces the
active key and moves the previous owned value into the completed batch. Batch order, ranges,
source/node lists, split reasons, and serialized keys are unchanged.

## Verification

- TDD RED: the focused contract found `key: current_key.clone()` and no move-based rollover.
- Focused batch-key construction static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `56/56`
  passed.
- Rust behavior coverage compares complete moved and cloned key sequences.
- The benchmark lives in a dedicated `batch/plan/performance_tests.rs` child module.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2692`;
- snapshot request: `37594b07cd8242f19472d7847ec7fa26`;
- attribution request: `27338cabb8804ceebe5a1bbf714d431e`;
- submit request: `90be35231e3c4165b67fc68cfcacf912`;
- validation ticket: `9c90089241224e84a9b0ce4469bd68d3`;
- source manifest: `89aa2dbdf07a72d635d374d955e70ba34567937fd6f2dfefa8f7709ba5f7ec57`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch7 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark rolls over 4,096 keys with owned resource/fallback strings and draw
effects over 11 alternating samples, comparing deep-cloned completion with moving the previous
active key. The P95 gate requires at least 20% improvement. Terminal P50/P95 nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.
