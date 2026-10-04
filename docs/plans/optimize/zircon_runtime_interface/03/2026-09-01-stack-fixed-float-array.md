record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/component/value.rs
related_tests:
  - zircon_runtime_interface/src/ui/component/value/fixed_float_array_performance_tests.rs
  - tools/tests/test_runtime_interface03_fixed_float_array_performance_contract.py
  - zircon_runtime_interface/src/ui/component/value/fixed_float_array_performance_tests.rs::runtime_interface03_batch47_48_stack_fixed_float_array_release_benchmark
---

# Stack fixed float array

## Scope

TOML conversion for Vec2/Vec3/Vec4 previously collected every numeric component into a heap `Vec`
and then converted that vector into the required fixed array. The converter now validates the fixed
length and fills `[f64; N]` directly on the stack. Integer widening, float preservation, invalid
type rejection, length rejection, and public value shapes are unchanged.

## Verification

- TDD RED: the focused contract found `collect::<Option<Vec<_>>>()` followed by `try_into()`.
- Focused Batch48 fixed-float-array contract after implementation: `2/2` passed.
- Batched static regression after Batch47-48: `117/117` passed (`105` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares stack and allocating implementations for Vec2/Vec3/Vec4,
  mixed integer/float input, short/long arrays, invalid scalar types, and non-array input.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: exact-path lease request `92886e67d04b462fae336699534beba2`; refreshed Batch47-48
lease request `8659a1a726624917a144423ed0980542`; baseline attribution request
`a44e4cf31b3b438ca07a6f70f66bdff1` (`attributed`).

## Performance contract

The ignored release benchmark converts a four-component TOML array 1,000,000 times over 11
alternating samples. It compares the former heap-vector conversion with direct stack-array filling
and requires at least 20% P95 improvement. Terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.

Combined Batch47-48 snapshot `2744` was created by request `fbb1c221369a4fb8b6e0009d15390f2d`.
The batched release resubmission request `runtime-interface03-batch47-48-20260901-zrvm-dirty-r1`
was rejected before ticket creation by the managed validation preflight because external worktree
`E:\\Git\\zr_vm` was dirty (`validation_ticket_external_worktree_dirty`). No Cargo run, commit,
push, or terminal performance value exists; the external worktree remains untouched.
