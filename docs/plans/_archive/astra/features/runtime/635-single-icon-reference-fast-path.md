---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/635/2026-09-01-single-icon-reference-without-hash-set.md
related_code:
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/ui/optimization_batch_ix_runtime635_tests.rs
tests:
  - zircon_runtime/src/asset/assets/ui/optimization_batch_ix_runtime635_tests.rs
---

# Single Icon Reference Fast Path

An external UI icon now projects zero or one direct reference without creating a temporary vector
and hash set for an impossible duplicate. Multi-reference UI documents retain hash membership and
fragment normalization semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime635 | Use the direct single-reference path for external icons | implemented_pending_validation | Source/behavior regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
