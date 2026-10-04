related_code:
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/optimization_batch_is_runtime629_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/629/2026-09-01-preallocated-gltf-mesh-dependency-index.md
tests:
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets/optimization_batch_is_runtime629_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Runtime629 Dependency Membership Capacity

The glTF mesh-subasset importer now reserves its dependency membership set from the primitive
count, with saturating arithmetic. Traversal order, duplicate handling, and published asset identity
are unchanged. The separate manifest-membership slice is recorded under Runtime02's asset-root
completion list.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime629 | Reserve glTF mesh dependency membership from the primitive bound | implemented_pending_validation | Runtime629 source regression and the shared Runtime/Editor static contract batches pass; scoped Rustfmt and diff checks pass. The ignored 31-sample helper is a microbenchmark only; managed Cargo, real import-caller measurement, and Release p50/p95/p99 evidence remain pending. |
