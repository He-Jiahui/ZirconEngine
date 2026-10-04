---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/640/2026-09-01-preallocated-build-export-target-indexes.md
related_code:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows/optimization_batch_ja_editor640_tests.rs
tests:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows/optimization_batch_ja_editor640_tests.rs
---

# Build Export Target Index Capacity

Build-export target projection now reserves platform and target-ID indexes from the materialized
target count. Duplicate occurrence handling, row order, stable IDs, and suffixing remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor640 | Reserve bounded build-export target indexes | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
