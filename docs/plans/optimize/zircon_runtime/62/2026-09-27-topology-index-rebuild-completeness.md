---
doc_type: optimization-record
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
implementation_files:
  - zircon_runtime/src/scene/world/hierarchy_topology.rs
tests:
  - zircon_runtime/src/scene/world/hierarchy_topology.rs::tests::missing_parent_projection_row_forces_source_rebuild
---

# Runtime62 hierarchy index rebuild completeness

## Defect and repair

The existing `missing_parent_projection_row_forces_source_rebuild` regression
removes one parent projection row while preserving the indexed entity set.
`is_current_for_entity_count` correctly returned false for that state, but
`needs_source_rebuild` checked only the dirty flag and indexed entity count.
It therefore returned false, contradicting the existing regression and the
complete-index contract used by `ensure_hierarchy_mutation_index_current`.
This is a deterministic source-level counterexample; no failing Cargo result
is claimed.

`needs_source_rebuild` now negates the existing complete-currentness predicate.
Both decisions include the dirty flag, indexed entity count and parent
projection count. The change is one production expression. The existing
regression, all other tests, topology ordering and generation behavior are
preserved byte for byte. No new test duplicates the existing case.

## Preservation and validation

The source was clean against Git before this slice, with SHA-256
`19f9049209e6545d1cc1d6910e400143e34739947c4f62ca5736733e562579e3`.
No live path lease existed; the historical attribution owner was archived.
The source and these two records were claimed as
`16c3ce39866e4a3fb3298fe258a6cfbb`. Exact inverse preservation, scoped formatting
and documentation checks are recorded under
`.codex/state/session-coordinator/async-validation-batches/` with prefix
`2026-09-27-astra-optimize-batch-q-runtime62-topology-index-completeness`.
Independent source review is complete with no findings; grouped managed
execution remains pending.

The predicate adds one constant-time map length comparison on the formerly
incomplete clean path, with no traversal or allocation. This is a code
property, not measured speed or allocation evidence. No new performance
threshold is invented. The existing Runtime62 scale/product gates, including
G15 and G24, remain open pending their complete managed evidence. Q includes
this existing regression in the combined Runtime/Editor library suite;
no standalone Cargo job is requested.
