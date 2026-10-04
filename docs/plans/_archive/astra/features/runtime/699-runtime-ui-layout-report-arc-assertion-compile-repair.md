---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/697-runtime-ui-layout-slot-accessor-compile-repair.md
---

# Runtime UI Layout Report Arc Assertion Compile Repair

The incremental-layout regression helper compared the published
`UiSurfaceFrame::layout_engine_report` `Arc` directly with its inner report.
The assertion now compares `Arc::as_ref()` to the expected value, matching the
immutable frame publication contract while leaving the debug-snapshot value
assertions unchanged. This is a test-boundary repair only; layout selection,
report generation, and publication behavior are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime76 incremental layout diagnostics | Dereferenced the published frame report at the assertion boundary in `surface_dirty_domains`. | Scoped source guard, Rustfmt, and the final batched Runtime/Editor/Text contract suites pass. | implemented_pending_validation |

## Batched local evidence

- Latest parallel Runtime performance-contract discovery passed `1146/1146`
  in `27.925s`; Editor passed `581/581` in `10.757s`; Runtime Text passed
  `143/143` in `11.506s` (`1870/1870` aggregate).
- Scoped Rustfmt parse/check and `git diff --check` passed; Git emitted only
  existing line-ending notices.

## Managed acceptance gate

This assertion repair is not a managed Cargo compile or product-performance
receipt. Managed admission remains deferred by the external
`E:\\Git\\zr_vm` dirty-worktree gate recorded in 696. Keep the row at
`implemented_pending_validation` until an owner-attributed Windows batch
supplies compile/test and p50/p95/p99 evidence.

Tooling changes remain deferred by request.
