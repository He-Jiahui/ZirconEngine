related_code:
  - zircon_editor/src/core/asset/dirty/registry.rs
  - zircon_editor/src/core/asset/dirty/registry/optimization_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/612/2026-09-01-reset-delta-direct-materialization.md
tests:
  - zircon_editor/src/core/asset/dirty/registry/optimization_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor612 Dirty Reset Delta Materialization

When a dirty-registry cursor requires a reset, the registry now starts with empty incremental
change collections and clones the stable document set once for the reset snapshot. Incremental
replay keeps its existing ordered journal and partition semantics.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor612 | Skip discarded external-change copies on reset while preserving generation retry behavior | implemented_pending_validation | Source regression and static contract checks pass, with scoped Rustfmt and diff checks. The ignored reset-seed benchmark is a helper model; managed Editor Cargo, the real dirty-document projection, and Release p50/p95/p99 evidence remain pending. |
