---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-manifest-membership.md
related_code:
  - zircon_runtime/src/asset/project/manifest/validation.rs
tests:
  - zircon_runtime/src/asset/project/manifest/validation/optimization_batch_ir_runtime629_tests.rs
---

# Runtime629 Manifest Membership Completion

This record closes the Astra ledger gap for Runtime629. Project-manifest validation reserves the
asset-root map and UI-root set from their admitted input lengths while preserving duplicate
ordering, overlap detection, UI scheme validation, and existing error payloads.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime629 | Reserve project-manifest asset/UI root membership | implemented_pending_validation | The source contract and focused regression exist. The ignored helper reports nearest-rank p50/p95/p99 over `MAX_PROJECT_ASSET_ROOTS` and requires reserved P95 to be at most 85% of the unreserved path. Managed Windows Runtime caller tests and Release product-workload evidence remain pending. |

The helper microbenchmark does not call the complete `ProjectManifest::validate` product path and
therefore cannot promote this item to accepted. Tooling remains outside this record.
