---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/659/2026-09-02-streamed-source-outline-boundaries.md
related_code:
  - zircon_editor/src/ui/asset_editor/source/source_sync.rs
  - zircon_editor/src/ui/asset_editor/source/source_sync/optimization_batch_jt_editor659_tests.rs
tests:
  - zircon_editor/src/ui/asset_editor/source/source_sync/optimization_batch_jt_editor659_tests.rs
---

# Streamed Source Outline Boundaries

Source-outline construction now sweeps the ordered event iterator directly through a peekable
cursor, removing the copied boundary vector and repeated tree lookups. Nested, same-start, invalid
range, and final-open-range semantics remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor659 | Stream ordered outline boundaries directly through the event iterator | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
