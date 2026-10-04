---
related_code:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/query_scratch.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
implementation_files:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/query_scratch.rs
plan_sources:
  - docs/plans/astra/performance/01-bounded-hotpaths.md
tests:
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - tools/tests/test_runtime_ui_surface_hit_query_scratch_contract.py
doc_type: milestone-detail
status: implemented_pending_validation
---

# UI Radius Query Scratch Reuse

Nearby cursor-radius fallback now writes its distance-sorted candidates into the retained
`UiHitQueryScratch` owned by the hit index. The scratch clears and bounds this buffer with the
existing generation, candidate, and mark storage, eliminating the former per-query temporary
vector while retaining the same candidate order and result projection.

This does not cap, truncate, or otherwise alter positive-radius query semantics. The separate
large-radius cell/candidate budget decision remains open.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M29 | Reuse the nearby fallback sort buffer across admissible radius queries | implemented_pending_validation | Runtime UI source-contract batch `40/40`; Python syntax, scoped Rustfmt, and scoped diff checks pass. The Rust regression verifies repeated nearby fallback results and stable retained capacity. Managed Runtime Cargo and release p50/p95/p99 evidence remain pending. |
