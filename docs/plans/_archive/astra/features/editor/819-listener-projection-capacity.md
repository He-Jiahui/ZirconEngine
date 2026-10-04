---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/313/2026-08-30-listener-projection-capacity-v2.md
  - docs/plans/optimize/zircon_editor/313/2026-09-19-listener-projection-capacity-repair.md
  - docs/plans/optimize/zircon_editor/313/2026-09-19-listener-projection-test-wiring-repair.md
related_code:
  - zircon_editor/src/core/editor_event/listener/projection.rs
  - zircon_editor/src/core/editor_event/listener/projection/capacity_tests.rs
tests:
  - zircon_editor/src/core/editor_event/listener/projection/capacity_tests.rs
  - tools/tests/test_editor_listener_projection_capacity_performance_contract.py
related_records:
  - docs/plans/astra/features/editor/821-listener-projection-test-wiring.md
---

# Editor819 Listener Projection Capacity

Editor listener descriptor and delivery JSON projections now reserve their exact source lengths
before mapping. The repair restores the intended Editor313 capacity contract in the current source
without changing response order, field shape, or empty-input semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor819 | Restore exact-capacity listener descriptor and delivery projections | implemented_pending_validation | TDD source/model contract `3/3`; existing lower Editor313 source/count regression and ignored `EDITOR313_LISTENER_PROJECTION_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-entry model changes `11→0` growth events for each projection; managed Cargo/Release and Editor event p50/p95/p99 evidence remain pending. |

The earlier combined current-source Runtime/Editor focused batch passed `41/41`;
a refreshed ten-contract Runtime/Editor source/model recheck passes `42/42`
with zero failures, errors, or skips. This is local source/model evidence;
managed Windows Cargo/Release and product percentile gates remain pending.

## Complexity boundary

Only the local output-vector allocation shape changes. JSON construction, field ordering, source
ownership, delivery ordering, and public control-response contracts remain unchanged. Empty input
still produces an empty zero-capacity vector.

## Managed gate

No standalone Cargo command was started. The slice joins the batched Runtime/Editor validation
handoff; managed Windows Release compilation and product percentile evidence remain pending behind
the external dirty `E:\Git\zr_vm` admission gate.
