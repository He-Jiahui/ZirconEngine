---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-node-pool-owned-key-lookup.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/736-virtual-list-reconciliation-scratch-reuse.md
implementation_files:
  - zircon_runtime/src/ui/surface/node_pool.rs
tests:
  - zircon_runtime/src/ui/surface/node_pool.rs
  - tools/tests/test_runtime_ui_node_pool_lookup_ownership_performance_contract.py
---

# Runtime node-pool owned key lookup reuse

`insert_or_reuse_pooled_child` now transfers the desired node into an owned lookup helper.
The helper moves component, control-ID, and node-path strings into the temporary pool key,
probes the bounded bucket, and restores them before merging or inserting the desired node. The
warm reuse path therefore avoids cloning all three key fields while preserving desired node
identity, pooled-node selection, bucket removal, capacity limits, and public borrowed lookup
compatibility. Residency reports also aggregate node and bucket counts in one map walk before
publication.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A P1-18 | Reuse owned node-pool key fields during retained-child lookup and aggregate residency reporting in one bucket walk. | RED/GREEN source guard and lower pointer/bucket/residency regressions pass; focused contracts pass `27/27`; the UI loader covers 173 modules and passes `776/776` in `1.962s`, while the broader non-tooling Runtime/Editor performance-plus-pressure batch covers 548 modules and passes `2057/2057` in `15.174s`; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Acceptance boundary

Keep this record pending until the owner-attributed Windows Release batch verifies compilation,
pool allocation behavior, and node-pool/navigation p50/p95/p99. The broader P1-18 provider,
template-handle, state-reset, and resource-generation work remains open; tooling is deferred.
No coordinator status was queried for this slice.

Current source hashes:

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/node_pool.rs` | `4278AC49103E86877D6D4556918909B0640686C899EF80A2F5DE5E764A59AD3E` |
| `tools/tests/test_runtime_ui_node_pool_lookup_ownership_performance_contract.py` | `B6860AA8545A7B20AFCA7F14AD59C95B0CB35C822F5BC2429A0F0C447098A947` |
