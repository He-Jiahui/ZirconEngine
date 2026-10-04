related_code:
  - zircon_runtime/src/ui/event_ui/manager/reflection_store.rs
  - zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a/2026-08-26-direct-property-query-projection.md
tests:
  - zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Runtime11A Direct Property Query Projection

Property queries now resolve the indexed node through borrowed lookups and clone only the selected
property at the API boundary instead of cloning the complete node descriptor. Missing-node,
missing-property, and owned response semantics remain unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A | Replace whole-node reflection projection with direct borrowed lookup | implemented_pending_validation | Reflection-store source and behavior contracts pass with scoped Rustfmt/diff checks. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 evidence remain pending. |
