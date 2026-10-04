---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-taffy-child-handle-buffer-reuse.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-runtime-taffy-retained-parent-product.md
related_code:
  - zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs
tests:
  - zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs
  - tools/tests/test_runtime_ui_taffy_parent_product_pressure.py
---

# Taffy Child Handle Buffer Reuse

The retained Taffy parent product now keeps its ordered `NodeId` projection alongside the
retained child records. Topology reconciliation reuses that allocation when calling
`set_children`, removing the short-lived per-update handle vector while preserving the same
stable node identities and error recovery behavior.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M31 | Retain and reuse the Taffy child-handle projection during topology updates | implemented_pending_validation | Product-level capacity/content regression and scoped Rustfmt pass; focused Runtime UI/Taffy/layout contracts `77/77` pass. Managed Cargo and Windows Release p50/p95/p99 allocation/time evidence remain pending. |
