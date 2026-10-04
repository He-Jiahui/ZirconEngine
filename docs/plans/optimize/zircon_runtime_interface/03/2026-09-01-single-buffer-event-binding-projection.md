record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/event_path.rs
  - zircon_runtime_interface/src/ui/binding/model/event_binding.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_call.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value/projection.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/event_binding/native_projection_performance_tests.rs
  - tools/tests/test_runtime_interface03_event_binding_native_projection_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/event_binding/native_projection_performance_tests.rs::runtime_interface03_batch39_single_buffer_event_binding_release_benchmark
---

# Single-buffer event binding projection

## Scope

`UiEventBinding::native_binding` previously allocated the event-path prefix and full action text
separately, then copied both into a third formatted string. Event paths can now append to a caller
buffer, and event binding projection writes the path, outer parentheses, and action recursively into
one `String`. The public `native_prefix()` and `native_binding()` owned-string contracts and emitted
syntax remain unchanged.

## Verification

- TDD RED: the focused contract found no event-path append helper and a formatted intermediate
  action string in `native_binding`.
- Focused Batch39 static performance contracts after implementation: `2/2` passed.
- Batched static regression: `99/99` passed (`87` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares the single-buffer result with the former allocating path for a
  large nested action and an actionless binding.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Batch37-39 snapshot `2738` was created by request `5f745c2425c2410a9ba1667effaffbf8`.
The batched managed validation request `278d8e8367c242c492dfd8b62c2e3d56` was rejected before
queueing by the external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run,
or terminal benchmark result exists for Batch39. This session will not modify that external
repository.

Ownership receipt: exact-path lease request `a9e06011fa724f0da9fe879025973ba1`; baseline
attribution request `7d86e7ebcb174509a4af7f54b502f6da` (`attributed`).

## Performance contract

The ignored release benchmark projects an event binding containing 256 long string arguments
10,000 times over 11 alternating samples. It compares the former prefix/action intermediates with
one output buffer and requires at least 20% P95 improvement. Terminal nanosecond values must come
from the managed Windows receipt before integration, push, or WeCom reporting.
