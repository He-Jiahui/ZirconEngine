record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/frame_extract.rs
related_tests:
  - tools/tests/test_runtime_interface03_render_list_direct_append_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/direct_append_tests.rs::runtime_interface03_batch4_render_frame_list_direct_append_release_benchmark
---

# Persistent render-frame list direct paint append

## Scope

`UiRenderFrameList::to_paint_elements_with_metrics` repeated the same per-command temporary-vector
allocation used by the flat render list while walking persistent command segments. Published frame
inspection and retained consumers therefore paid one temporary allocation per command.

The persistent list now invokes the crate-internal command append helper and writes directly into
its final paint-element vector. Persistent command iteration, cached metadata, paint order, the
serialized frame contract, and UI12's below-native raster-scale clamp remain unchanged.

## Verification

- TDD RED: the static contract found the per-command `Vec` plus `elements.append` path.
- Focused direct-append contracts after implementation: `4/4` passed.
- Rust behavior test compares the complete persistent-list output against the previous algorithm.
- The dedicated child test module avoids adding benchmark volume to the 961-line main file.
- `python -m compileall`, scoped Rust 1.94.1 `rustfmt --check`, and scoped `git diff --check`:
  passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2670`;
- snapshot request: `6dbaf1e72c844c1cbfff45fb357d4afa`;
- attribution request: `fb4c476f1e384b5dabe553b6bd7c4cff`;
- submit request: `0cfd4f76f39c4a8fa48f280630c6122e`;
- validation ticket: `6d7426ad3bc245418c0da3a3ed139819`;
- source manifest: `ec180fdd1a78340b5d94fb4f1ec05b00682560f6744bb4fea669f47677c8c421`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1
  runtime_interface03_batch4 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark projects 8,192 persistent commands over 11 alternating samples,
comparing the previous temporary-vector path with direct final-buffer append. The P95 gate
requires at least 10% improvement. Terminal P50/P95 nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
