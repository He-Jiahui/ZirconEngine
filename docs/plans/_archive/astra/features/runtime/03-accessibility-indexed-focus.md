related_code:
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/diagnostics/indexed_focus_tests.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/resolution.rs
related_records:
  - docs/plans/astra/features/runtime/706-accessibility-visibility-detached-scratch-reuse.md
  - docs/plans/astra/features/runtime/708-accessibility-relation-target-retain.md
  - docs/plans/astra/features/runtime/709-accessibility-resolution-key-scratch-reuse.md
  - docs/plans/astra/features/runtime/710-accessibility-description-reference-borrow.md
  - docs/plans/astra/features/runtime/711-accessibility-filter-map-membership.md
  - docs/plans/astra/features/runtime/712-ui-tree-dirty-index-ownership.md
  - docs/plans/astra/features/runtime/819-accessibility-diagnostic-index-dedup.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-08-26-accessibility-indexed-focus.md
tests:
  - zircon_runtime/src/ui/accessibility/diagnostics/indexed_focus_tests.rs
  - zircon_runtime/src/ui/accessibility/diagnostics/index_dedup_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Runtime03 Accessibility Indexed Focus

Accessibility focus validation now retains the existing node index through focus clearing and root
fallback, removing post-index linear node scans while preserving duplicate handling, diagnostics,
and fallback order.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime03 | Mutate validated focus targets through the existing node index | implemented_pending_validation | Indexed-focus source/behavior contracts pass with scoped Rustfmt/diff checks. The ignored benchmark is a helper workload; managed Runtime Cargo and Release p50/p95/p99 evidence remain pending. |
| Runtime11A/78 | Reuse accessibility extraction scratch, retain relation/description payloads only when resolution requires ownership, and use the published map for child membership | implemented_pending_validation | Accessibility extraction and rich-text contracts pass in the batched source suite; managed Runtime Cargo, allocation, and product latency evidence remain pending. |
| Runtime819 | Use one entry-based node index for duplicate diagnostics and retained first-node lookup | implemented_pending_validation | TDD source/model contract `3/3`; lower duplicate/order regression and ignored `RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1` marker are wired; the 4,096-node deterministic model changes auxiliary index allocations `2→1`; managed Cargo/Release and accessibility p50/p95/p99 evidence remain pending. |
