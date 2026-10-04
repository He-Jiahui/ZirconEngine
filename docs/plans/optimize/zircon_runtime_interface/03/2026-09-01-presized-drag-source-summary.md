---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/component/drag.rs
related_tests:
  - zircon_runtime_interface/src/ui/component/drag/summary_performance_tests.rs
  - tools/tests/test_runtime_interface03_drag_summary_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/component/drag/summary_performance_tests.rs::runtime_interface03_batch51_52_presized_drag_summary_release_benchmark
---

# Presized drag source summary

## Scope

The asset-kind and display-name summary branch previously used `format!`, allowing the required
owned string to grow from a small formatting estimate. It now reserves `kind + separator + name`
once and appends directly. Empty kind/name fallback, display-name cloning, locator fallback, absent
metadata, separators, and the public summary API are unchanged.

## Verification

- TDD RED: the focused contract found the growing `format!` path and no release benchmark child.
- Focused Batch51-52 performance contracts after implementation: `4/4` passed.
- Batched static regression after Batch51-52: `124/124` passed (`112` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares the presized implementation with the former formatting oracle for
  full metadata, empty kind, empty name, locator fallback, and absent metadata.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: initial exact-path lease request `235e24885213443bb878db1f16cf795b`; refreshed
batch lease request `0834608c8fcb433399a23eb7221fb3ce`; baseline attribution request
`60c68e0f2a594ae9a4a4a5fb44045f70` (`attributed`).

## Performance contract

The ignored release benchmark formats 200,000 long asset summaries over 11 alternating samples. It
compares the former formatting growth path with an exact-capacity output buffer and requires at
least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.

Combined Batch51-52 snapshot `2746` was created by request
`e770dff93eeb455693db534f540a2720`. Batched release request
`runtime-interface03-batch51-52-20260901-r1` was rejected before ticket creation by
`validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`. No Cargo run,
commit, push, or terminal performance value exists; the external worktree remains untouched.
