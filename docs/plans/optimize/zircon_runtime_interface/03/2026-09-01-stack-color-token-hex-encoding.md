record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs
related_tests:
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry/color_hex_performance_tests.rs
  - tools/tests/test_runtime_interface03_color_token_hex_performance_contract.py
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry/color_hex_performance_tests.rs::runtime_interface03_batch55_61_stack_color_token_hex_release_benchmark
---

# Stack color-token hexadecimal encoding

## Scope

Design-token color publication previously formatted each RGB or RGBA value through generic
formatting. The implementation now reserves the exact seven- or nine-byte result and appends
lowercase hexadecimal nibbles directly. Opaque colors still omit alpha; translucent colors retain
it, with the same rounding, leading zeroes, and lowercase output.

## Verification

- TDD RED: the focused contracts found no dedicated encoder and no behavior/benchmark module.
- The behavior oracle compares direct encoding with the former formatting implementation across
  opaque, translucent, zero, maximum, leading-zero, and nibble-boundary values.
- Complete RuntimeInterface03 static performance-contract discovery: `124/124` passed; scoped Rust
  1.94.1 formatting and diff checks passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.
- Batched managed request `runtime-interface03-batch57-58-20260901-r1` was rejected before ticket
  creation or Cargo execution because the external `E:\Git\zr_vm` worktree was dirty. The request
  produced no compile, behavior, benchmark, integration, push, or performance receipt.
- Current source/test contract hashes: `cascade_registry.rs` `0541565B03B95328961C5BBE820F14C95D7823D94E7A62BA7262C879CB9257A4`,
  `color_hex_performance_tests.rs` `0BE1567FDEDF1D472AE6F8CF20382A3353713A9722A64A1375BA141475B51658`,
  static guard `E1EE547FC4365FAD3BFB1D9096A32A43E54ABFEE0626260E82BD8C3EE2A296DC`.
- Ownership lease request: `29b3fe902b8f4d0da605c4826a4a55e0`; baseline attribution: `f8a9071158d74af1823c2613935b4f44`.
- Batched request `runtime-interface03-batch55-59-20260901-r1` was accepted for asynchronous
  reconciliation, but coordinator post-response timed out with request `ed8c6d3ced7d448d9d805f8302709365`;
  no terminal Cargo or benchmark output is available yet.
- A post-submit source review changed the loop binding from an implicitly dereferenced `&u8` to
  explicit by-value `&channel`; current `cascade_registry.rs` SHA-256 is
  `B3E04DD0FEE62825E6D8BE4C1B5F3E52D262C4CA752655041E6FDFBF2B1E27AE`. The accepted request's
  prior manifest is therefore stale and must not be used as current acceptance evidence.
- Current Batch55-61 submission `runtime-interface03-batch55-61-20260901-r1` was rejected before
  ticket creation by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; no Cargo, terminal performance, commit, or push evidence exists.

## Performance contract

The ignored release benchmark alternates opaque and translucent colors across 1,000,000 builds and
11 samples. It compares generic formatting with exact-capacity nibble encoding and requires at
least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
