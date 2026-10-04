---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/201-runtime-text-font-document-shaping-layout-raster-atlas-render-authority-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime Text Wrapping Outcome Test Repair

The wrapping implementation now publishes `TextShapingOutcome<Vec<_>>` so
deferred font-generation failures are preserved. Three tests still treated
that outcome as a plain vector and called `len()`/indexing directly. They now
explicitly consume `into_result()` at the test boundary, retaining the same
line-count, source-range, and cache-miss assertions without weakening the
production failure contract.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11B / Runtime201 wrapping regressions | Unwrap the three valid wrapping fixtures before vector assertions. | Outcome-boundary source guard, Rustfmt, and the final Runtime/Editor/Text contract batch pass. | implemented_pending_validation |

## Batched local evidence

- Latest parallel Runtime performance-contract discovery passed `1146/1146`
  in `27.925s`; Editor passed `581/581` in `10.757s`; Runtime Text passed
  `143/143` in `11.506s` (`1870/1870` aggregate).
- The focused wrapping source remains free of the retired growing-prefix
  candidate path; scoped formatting and diff checks pass.

## Managed acceptance gate

This is a Runtime text test-contract repair, not a managed Cargo or product
performance receipt. Admission remains deferred by the external
`E:\\Git\\zr_vm` dirty-worktree gate documented in the asynchronous logs.
Keep the row at `implemented_pending_validation` until owner-attributed
Windows compile/test and Release p50/p95/p99 evidence is available.

Tooling changes remain deferred by request.
