---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/639/2026-09-01-preallocated-dependency-ready-jobs.md
  - docs/plans/optimize/zircon_editor/639/2026-09-01-preallocated-menu-operation-index.md
related_code:
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
  - zircon_editor/src/core/jobs/system/pending/tests/fairness.rs
  - zircon_editor/src/ui/workbench/model/menu/optimization_batch_iz_editor639_tests.rs
tests:
  - zircon_editor/src/core/jobs/system/pending/tests/fairness.rs
  - zircon_editor/src/ui/workbench/model/menu/optimization_batch_iz_editor639_tests.rs
---

# Ready-Job And Menu Index Capacity

Dependency promotion reserves ready jobs from the indexed dependent count, while contributed-menu
projection reserves operation membership from the recursive item bound. Ordering, fairness,
duplicate semantics, and missing-entry handling remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor639 | Bound ready-job and menu-operation indexes before insertion | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmarks are helper workloads; managed Editor Cargo and Release p50/p95/p99 remain pending. |
