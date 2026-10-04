record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/component/value.rs
related_tests:
  - zircon_runtime_interface/src/ui/component/value/display_text_performance_tests.rs
  - tools/tests/test_runtime_interface03_value_display_text_performance_contract.py
  - zircon_runtime_interface/src/ui/component/value/display_text_performance_tests.rs::runtime_interface03_batch47_48_single_buffer_value_display_release_benchmark
---

# Single-buffer value display text

## Scope

`UiValue::display_text` previously allocated one temporary string per Vec2/Vec3/Vec4 component and
then allocated the final joined string. Vector summaries now reserve one output buffer and stream
each formatted component into it. Scalar float and vector output share one trimming implementation;
the three-decimal rule, trailing-zero behavior, separators, non-finite output, and public API are
unchanged.

## Verification

- TDD RED: the focused contract found per-component `trim_float` allocations followed by `format!`.
- Focused Batch47 value display contract after implementation: `2/2` passed.
- Batched static regression after Batch47-48: `117/117` passed (`105` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares the single-buffer implementation with the former allocating
  oracle for Vec2/Vec3/Vec4, integral and fractional values, negative zero, NaN, and infinities.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: exact-path reclaim/lease request `bfd1600f89a44b45a571dcda3f238df5`; refreshed
Batch47-48 lease request `8659a1a726624917a144423ed0980542`; baseline attribution request
`a44e4cf31b3b438ca07a6f70f66bdff1` (`attributed`);
the previous coordinator owner was archived and no live lease existed. Current bytes were preserved
before the scoped edit.

## Performance contract

The ignored release benchmark formats a four-component vector 500,000 times over 11 alternating
samples. It compares five allocating strings with one shared output buffer and requires at least 20%
P95 improvement. Terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.

Combined Batch47-48 snapshot `2744` was created by request `fbb1c221369a4fb8b6e0009d15390f2d`.
The batched release resubmission request `runtime-interface03-batch47-48-20260901-zrvm-dirty-r1`
was rejected before ticket creation by the managed validation preflight because external worktree
`E:\\Git\\zr_vm` was dirty (`validation_ticket_external_worktree_dirty`). No Cargo run, commit,
push, or terminal performance value exists; the external worktree remains untouched.
