record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
related_tests:
  - tools/tests/test_runtime_interface03_render_list_direct_append_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/direct_append_tests.rs::runtime_interface03_batch4_render_list_direct_append_release_benchmark
---

# Flat render-list direct paint append

## Scope

`UiRenderList::to_paint_elements_with_metrics` previously allocated a temporary paint-element
vector for every render command and then moved that vector into the final output. Large retained
lists therefore performed command-count-proportional temporary allocations in addition to growth
of the result vector.

`UiRenderCommand` now exposes a crate-internal cached append helper backed by the same paint
projection implementation used by its existing fill APIs. The flat list writes each command
directly into the final output buffer. Multi-element paint order is based on the output length at
entry, so cached metadata, ordering, empty-command fallback, and public APIs are unchanged.

## Verification

- TDD RED: the static contract found the per-command `Vec` plus `elements.append` path and no
  append helper.
- Focused direct-append contracts after implementation: `4/4` passed.
- Rust behavior test compares the complete direct output against the previous per-command
  collection algorithm.
- New tests/benchmarks live in a 183-line child module; `frame_extract.rs` remains 961 lines.
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

The ignored release benchmark projects 8,192 commands over 11 alternating samples, comparing the
previous temporary-vector path with direct final-buffer append. The P95 gate requires at least
10% improvement. Terminal P50/P95 nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
