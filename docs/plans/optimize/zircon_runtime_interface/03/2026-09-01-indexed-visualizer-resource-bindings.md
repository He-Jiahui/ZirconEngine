record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer/resource_binding_index.rs
related_tests:
  - tools/tests/test_runtime_interface03_visualizer_resource_binding_index_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/visualizer/resource_binding_index.rs::runtime_interface03_batch5_visualizer_resource_binding_index_release_benchmark
---

# Indexed visualizer resource bindings

## Scope

The render visualizer previously searched every accumulated resource binding for every paint and
batch resource. A diagnostic snapshot with many unique resources therefore turned resource
projection into an O(n squared) scan.

`ResourceBindingIndex` now maps each resource ID to a compact list of candidate binding indices.
Candidates still use the complete `UiRenderResourceKey` equality contract, so equal IDs with
different kinds, revisions, atlas pages, UV rectangles, or fallbacks remain separate. Non-reflexive
NaN UV keys also retain the previous non-merging behavior. Binding and association output remains
in first-seen order.

The index, behavior tests, and benchmarks live in the dedicated
`ui/surface/render/visualizer/resource_binding_index.rs` module instead of adding another
responsibility to the visualizer orchestration file.

## Verification

- TDD RED: the focused static contract failed because the index module did not exist and the
  visualizer still used `bindings.iter_mut().find`.
- Focused new and existing visualizer static contracts after implementation: `4/4` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `52/52`
  passed, including the borrowed mount-name occupancy guard.
- Rust behavior tests cover exact-key merging, same-ID revision separation, output order, and NaN
  UV non-merging semantics.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2675`;
- snapshot request: `e3a0fa08b4e74e12810085d7fd5e6cdb`;
- attribution request: `6a60b36cf9364807954168604fa4a7d4`;
- submit request: `4568dff3a00a4f4ab2c67ab047521bed`;
- validation ticket: `3ebadc27f3e74064ab33afd7d83c061c`;
- source manifest: `f6c1015ca72877ed5af038e7c523bbc04c6305da9c3da81e8a5029f7bfe57ba5`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch5 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark admits 4,096 unique image resources over 11 alternating samples and
compares the previous whole-vector lookup with the ID-bucket index. The P95 gate requires at least
20% improvement. Terminal P50/P95 nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
