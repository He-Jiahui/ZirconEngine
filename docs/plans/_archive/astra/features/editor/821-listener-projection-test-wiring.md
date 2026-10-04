---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/313/2026-09-19-listener-projection-test-wiring-repair.md
related_code:
  - zircon_editor/src/core/editor_event/listener/projection.rs
  - zircon_editor/src/core/editor_event/listener/projection/capacity_tests.rs
tests:
  - zircon_editor/src/core/editor_event/listener/projection/capacity_tests.rs
  - tools/tests/test_editor_listener_projection_capacity_performance_contract.py
---

# Editor821 Listener Projection Test Wiring

The Editor313 lower Rust regression and ignored Release marker are now wired
into the `projection.rs` test module instead of remaining in a detached file.
This closes the test-harness reachability gap without changing production
projection semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor821 | Wire the existing listener projection lower regression and Release marker | implemented_pending_validation | Intentional RED then GREEN source contract `3/3`; exact-file Rustfmt passes; managed Cargo/Release and Editor Event p50/p95/p99 evidence remain pending. |

## Managed gate

No standalone Cargo command was started. The repaired lower test joins the
batched Runtime/Editor handoff; managed Windows Release execution and product
percentile evidence remain pending behind the external dirty `E:\Git\zr_vm`
worktree gate.
