record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/batch/key.rs
related_tests:
  - tools/tests/test_runtime_interface03_batch_key_construction_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/batch/key.rs::runtime_interface03_batch7_standalone_batch_key_clip_release_benchmark
---

# Direct standalone batch-key clip projection

## Scope

`UiBatchKey::from_paint_element` previously created a new clip-state hash index for one paint
element, interned at most one clip, and then discarded the index. Visualizer projection invokes
this standalone constructor for every element, so clipped diagnostics paid an allocation and hash
table setup per row.

Standalone construction now clones the element's clip directly into the key. The shared
`from_paint_element_with_clip_states` path used by `UiBatchPlan` still interns clips across the
whole plan, preserving canonical shared clip state where reuse is meaningful. Key fields and the
public serialized representation are unchanged.

## Verification

- TDD RED: the focused contract found `UiBatchClipStates::default()` in the standalone
  constructor.
- Focused batch-key construction static contracts after implementation: `2/2` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `56/56`
  passed.
- Rust behavior coverage compares direct construction with the shared interning path.
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

The ignored release benchmark constructs 200,000 clipped standalone keys over 11 alternating
samples, comparing one-entry interner creation with direct clip projection. The P95 gate requires
at least 20% improvement. Terminal P50/P95 nanosecond values must come from the managed Windows
receipt before integration, push, or WeCom reporting.
