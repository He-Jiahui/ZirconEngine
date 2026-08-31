# Runtime74 UI Resource Reference Streaming Visitor Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime74-three-task-hash-streaming-batch-20260830`
- Source plans: Runtime74 P1-45 / G047 and the Runtime64 dependency extraction boundary
- Status: implementation and release gate authored; batched managed validation pending

## Problem

UI asset dependency extraction recursively collected every matching `res://`,
`asset://`, and `project://` URI into a temporary `Vec<&str>`. The caller then
consumed that vector exactly once to normalize locators and build the unique
`AssetReference` result. A resource-heavy document therefore allocated and grew
an intermediate pointer array proportional to the total URI count.

## Change

- Replace `collect_resource_uris` with a borrowed `visit_resource_uris`
  callback that streams each matching URI directly to the existing reference
  normalizer and deduplicator.
- Thread the same callback through imports, token values, root nodes, component
  roots, child mounts, style declarations, TOML arrays, and TOML tables.
- Preserve the established traversal order, URI schemes, locator normalization,
  first-seen deduplication, final result type, and zero URI string clones.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| One document containing 4,096 resource URIs | one temporary URI pointer vector | no temporary URI pointer vector | 100% of collection storage removed |
| URI string clones during traversal | 0 | 0 | unchanged |
| Traversal complexity | O(U) plus temporary-vector growth | O(U) streaming callbacks | same linear bound, lower allocation traffic |

The ignored release gate runs 17 alternating legacy/visitor sample pairs over
4,096 resource URIs. Acceptance requires visitor nearest-rank P95 to be at most
80% of legacy P95, a minimum 20% reduction. Exact Windows timing values remain
pending the batched coordinator run.

## Acceptance

- `runtime74_batch_ui_resource_reference_visitor_preserves_order`
  compares visitor output with the legacy recursive collector.
- `runtime74_batch_ui_resource_reference_visitor_has_no_temporary_uri_vector`
  locks the production callback shape.
- `optimization_batch_20260826b_runtime74_ui_resource_reference_visitor_borrows_document_uris`
  locks borrowed URI ownership at the asset integration boundary.
- `runtime74_batch_ui_resource_reference_visitor_performance_evidence`
  emits `RUNTIME74_UI_RESOURCE_REFERENCE_VISITOR_BENCH_V1`, all raw samples,
  the temporary-vector counts, and the 20% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt, source contracts, and scoped diff checks must
  pass before managed validation submission.

The managed `runtime74_batch_` release gate seals this source-local behavior/structure/performance
trio together with the hot-reload and watch-invalidation slices in one Cargo invocation: three
source contracts, nine Rust tests, and three performance rows. The existing asset integration
regression above remains unchanged outside this exact owned filter. Dynamic Windows marker values,
commit attribution, and WeCom publication remain pending the coordinator result.

## Remaining Plan Work

This slice does not close Runtime74 or Runtime64. Incremental dependency graph
publication, generation-aware reload, cancellation, complete UI hot reload,
and product-scale resource performance gates remain open.
