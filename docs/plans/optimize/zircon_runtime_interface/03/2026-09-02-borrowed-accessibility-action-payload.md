record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
related_tests:
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/accessibility_payload_performance_tests.rs
  - tools/tests/test_runtime_interface03_accessibility_payload_performance_contract.py
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter/accessibility_payload_performance_tests.rs::runtime_interface03_batch68_69_borrowed_accessibility_payload_release_benchmark
---

# Borrowed accessibility action payload

## Scope

Accessibility action adaptation previously copied the entire validated ABI payload into a temporary
`Vec<u8>` before read-only JSON deserialization. The implementation now validates the same carrier
and 256 KiB limit, borrows the payload only for the synchronous parse, and removes that staging
allocation and byte copy. Text and IME paths still materialize owned strings, so no payload lifetime
escapes the adapter call.

## Verification

- TDD RED: the focused contract found `payload_bytes` in `accessibility_event` and no borrowed
  payload behavior or release benchmark.
- Focused logical-key/accessibility and adjacent event-adapter static contracts after implementation:
  `8/8` passed.
- The Rust behavior oracle compares copied bytes with the bounded borrowed view and verifies that
  the production adapter emits the same accessibility request for a 16 KiB value payload.
- Existing runtime-event accessibility behavior coverage remains part of the managed crate test
  selection.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number, integration, push, or WeCom claim exists yet.
- Batched request `runtime-interface03-batch68-69-20260902-r1` was rejected before ticket creation
  or Cargo execution by `validation_ticket_external_worktree_dirty` for external worktree
  `E:\\Git\\zr_vm`; it produced no compile, behavior, benchmark, integration, push, or performance
  receipt.

## Performance contract

The ignored release benchmark admits a 16 KiB payload 20,000 times over 11 alternating samples. It
compares the former owned staging copy with the bounded borrowed view and requires at least 75% P95
improvement. The structural contract also requires zero staging `Vec` allocation in
`accessibility_event`. Terminal nanosecond values must come from the managed Windows receipt.
