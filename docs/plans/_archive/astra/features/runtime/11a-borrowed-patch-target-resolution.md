related_code:
  - zircon_runtime/src/ui/event_ui/manager/reflection_store.rs
  - zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a/2026-08-26-borrowed-patch-target-resolution.md
tests:
  - zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Runtime11A Borrowed Patch Target Resolution

Reflection-patch validation now keeps the indexed tree identity borrowed and copies only compact
node/patch data until the changed-tree map needs an owned key. Validate-before-mutate,
deduplication, ordering, and notification semantics are unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime11A | Remove per-patch tree-identity cloning from target resolution | implemented_pending_validation | Reflection-store source and behavior contracts pass with scoped Rustfmt/diff checks. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 evidence remain pending. |
