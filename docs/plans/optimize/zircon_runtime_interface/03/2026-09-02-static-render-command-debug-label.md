record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/render/command/debug_label_performance_tests.rs
  - tools/tests/test_runtime_interface03_render_command_debug_label_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/command/debug_label_performance_tests.rs::runtime_interface03_batch74_75_static_render_command_debug_label_release_benchmark
---

# Static render-command debug labels

## Scope

Cached paint elements previously formatted `UiRenderCommandKind` through the general-purpose Debug
formatter before allocating the label string. The command kind has four fixed variants, so cached
label projection now selects the exact Debug-compatible static name and performs one ordinary string
allocation. Transient elements still omit labels, and the cached label spelling remains exactly
`Group`, `Quad`, `Text`, or `Image`.

## Verification

- TDD RED: the focused contracts failed while cached projection still used `format!("{:?}")` and
  no static-label benchmark existed.
- Focused Batch74-75 static performance contracts after implementation: `6/6` passed.
- Full RuntimeInterface03 static performance contracts: `162/162` passed.
- Python compile checks and scoped `git diff --check`: passed.
- Rust behavior coverage compares all four static labels with the former Debug formatting output.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

The combined Batch74-75 managed request `runtime-interface03-batch74-75-20260902-r1` was rejected
before ticket creation or Cargo launch with `validation_ticket_external_worktree_dirty` for external
repository `E:\Git\zr_vm`. No managed compile, behavior, benchmark, or terminal performance receipt
exists for this batch.

Copy-complete ownership receipt: exact-path lease request
`a5f92153b04b4a2fb9448ae6af0b5781`; baseline attribution request
`8870bb1647d246cbbe14e1ae58ebb8f0` (`attributed`).

The shared `command.rs` source also carries the existing render-list append, frame extraction,
frame-command directory/deserialization, paint-capacity, and RGBA encoding work; those exact
source/test/guard/record paths must travel in the copy-complete validation union.

No commit, push, or WeCom notification is permitted until managed validation is terminal-successful
and the coordinator finalizes the attributed union.

## Performance contract

The ignored release benchmark formats 1,000,000 rotating command kinds over 11 alternating samples.
It compares general Debug formatting with static-name selection and requires at least 20% P95
improvement. Exact terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.
