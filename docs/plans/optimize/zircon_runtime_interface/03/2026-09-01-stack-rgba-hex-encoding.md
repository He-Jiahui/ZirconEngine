record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/command.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/render/command/rgba_hex_performance_tests.rs
  - tools/tests/test_runtime_interface03_rgba_hex_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/command/rgba_hex_performance_tests.rs::runtime_interface03_batch55_61_stack_rgba_hex_release_benchmark
---

# Stack RGBA hexadecimal encoding

## Scope

Text-box background and border paint decoration generation previously sent every fixed-width RGBA
value through generic formatting. The implementation now reserves the exact nine-byte result and
appends uppercase hexadecimal nibbles directly. Channel rounding, leading zeroes, uppercase output,
and the mandatory alpha suffix remain unchanged.

## Verification

- TDD RED: the focused contracts found the generic formatting path and missing behavior/benchmark
  module.
- The behavior oracle compares direct encoding with the former formatting implementation across
  zero, maximum, leading-zero, nibble-boundary, and mixed values.
- Complete RuntimeInterface03 static performance-contract discovery: `124/124` passed; scoped Rust
  1.94.1 formatting and diff checks passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.
- Batched managed request `runtime-interface03-batch57-58-20260901-r1` was rejected before ticket
  creation or Cargo execution because the external `E:\Git\zr_vm` worktree was dirty. The request
  produced no compile, behavior, benchmark, integration, push, or performance receipt.
- Current source/test contract hashes: `command.rs` `7DA9F7059427B9CABF2C754D094EABAE3104943225287F4BF6BB13DD50388128`,
  `rgba_hex_performance_tests.rs` `B3EF0BA533AA5935220B543FDEFB92844EB16F323833DA020C538DF6C2C25065`,
  static guard `D1E4E4A2F1405CB7871CF5F053A4E6B2CE0CC4F005F7E819CA2CC539D74F9FD`.
- Ownership lease request: `29b3fe902b8f4d0da605c4826a4a55e0`; baseline attribution: `f8a9071158d74af1823c2613935b4f44`.
- Batched request `runtime-interface03-batch55-59-20260901-r1` was accepted for asynchronous
  reconciliation, but coordinator post-response timed out with request `ed8c6d3ced7d448d9d805f8302709365`;
  no terminal Cargo or benchmark output is available yet.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

## Performance contract

The ignored release benchmark builds 1,000,000 fixed-width RGBA strings over 11 alternating
samples. It compares generic formatting with exact-capacity nibble encoding and requires at least
20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.
