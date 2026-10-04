---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/631/2026-09-01-preallocated-reference-candidates.md
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/reference_resolver/optimization_batch_is_runtime631_tests.rs
tests:
  - zircon_runtime/src/asset/reference_resolver/optimization_batch_is_runtime631_tests.rs
---

# Reference Candidate Capacity

Filesystem reference resolution now reserves locator-to-hint and hint-to-locator candidate vectors
from the project-root bound. Traversal order, ambiguity decisions, and repair payloads are
unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime631 | Bound reference candidate vectors before root traversal | implemented_pending_validation | Source regression and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 remain pending. |
