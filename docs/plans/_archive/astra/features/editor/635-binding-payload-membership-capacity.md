---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/635/2026-09-01-preallocated-binding-payload-membership.md
related_code:
  - zircon_editor/src/ui/asset_editor/binding/schema_projection.rs
tests:
  - zircon_editor/src/ui/asset_editor/binding/schema_projection.rs
---

# Binding Payload Membership Capacity

Binding schema projection now reserves payload-key membership from explicit payload entries and
materialized suggestions. Explicit values, schema defaults, duplicate suppression, diagnostics,
and item order remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Editor635 | Bound payload-key membership before projection loops | implemented_pending_validation | Existing behavior/source regressions and scoped Rustfmt/diff checks pass. The ignored benchmark is a helper workload; managed Editor Cargo and Release p50/p95/p99 remain pending. |
