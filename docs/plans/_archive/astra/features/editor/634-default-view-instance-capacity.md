---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/634/2026-09-01-preallocated-default-view-instances.md
related_code:
  - zircon_editor/src/ui/workbench/preset/default_registry.rs
  - zircon_editor/src/ui/workbench/preset/default_registry/optimization_batch_ix_editor634_tests.rs
tests:
  - zircon_editor/src/ui/workbench/preset/default_registry/optimization_batch_ix_editor634_tests.rs
---

# Default View Instance Capacity

Default workbench view projection now reserves unique-ID membership and instance outputs from the
bounded primary and drawer view count. Traversal order, duplicate handling, titles, and host
assignment remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor634 | Reserve bounded default view-instance projections | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
