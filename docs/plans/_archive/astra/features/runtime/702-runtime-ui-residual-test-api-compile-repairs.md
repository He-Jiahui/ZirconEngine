---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/701-runtime-text-wrapping-outcome-test-repair.md
---

# Runtime UI Residual Test-API Compile Repairs

This batch closes four residual test-boundary mismatches exposed by the
current Runtime UI contracts:

- incremental arrangement tests now destructure the four-field finish receipt;
- projected frame-hit tests use the indexed grid query helper;
- repeated navigation-index rebuilds qualify the shared arranged-node-index
  constructor instead of calling a shadowed local map;
- ellipsis tests inspect the retained typed virtual-source receipts directly.

The repairs preserve the production ownership and failure semantics while
making the tests consume the APIs that the optimized Runtime paths publish.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime76 arrangement and hit testing | Align finish receipts and projected-grid test queries with current APIs. | Focused arrangement, navigation-index, and arranged-visibility source contracts pass. | implemented_pending_validation |
| Runtime11B typed virtual fragments | Replace removed receipt convenience calls with direct typed receipt lookup. | Text wrapping/ellipsis source checks and the final Runtime/Editor/Text contract batch pass. | implemented_pending_validation |

## Batched local evidence

- Arrangement input patch contracts: `2/2`.
- Navigation-index contracts: `6/6`.
- Arranged-visibility contracts: `9/9`.
- Runtime route/hover and text contracts remain green in the aggregate batch.
- Latest parallel Runtime/Editor/Text performance-contract batch: `1870/1870`
  (Runtime `1146/1146` in `27.925s`, Editor `581/581` in `10.757s`, Runtime
  Text `143/143` in `11.506s`). The focused arrangement/navigation/visibility
  set also passed `58/58` in `4.360s`.
- Rustfmt parse/check and scoped whitespace checks pass for the directly edited
  boundaries; existing line-ending notices are non-semantic.

## Managed acceptance gate

These are compile-contract repairs and source evidence, not managed Cargo or
product-performance receipts. The external `E:\\Git\\zr_vm` dirty-worktree
gate still prevents owner-attributed Windows Release admission; retain
`implemented_pending_validation` until compile/test plus p50/p95/p99 evidence
is published asynchronously.

Tooling changes remain deferred by request.
