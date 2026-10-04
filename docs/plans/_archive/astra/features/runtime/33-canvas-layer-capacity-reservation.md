---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-canvas-layer-capacity-reservation.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/tests/canvas_slot_layout.rs
tests:
  - zircon_runtime/src/ui/tests/canvas_slot_layout.rs
  - tools/tests/test_runtime_ui_taffy_parent_product_pressure.py
  - tools/tests/test_runtime_ui_arranged_visibility_index_performance_contract.py
---

# Canvas Layer Capacity Reservation

The arranged Canvas projection now reserves the known slot budget for its flattened layers and
each parent's filtered child list. Canvas grouping, hidden-child filtering, z-order, and draw
semantics are unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A/M33 | Reserve bounded Canvas layer and per-parent child capacities during arranged-tree projection | implemented_pending_validation | Existing Canvas behavior tests plus a source capacity guard are present; combined Runtime/Editor hotpath contract batch `135/135`, Python syntax, Rustfmt, and scoped diff checks pass. Managed Runtime Cargo and Windows Release allocation/time p50/p95/p99 remain pending. |
