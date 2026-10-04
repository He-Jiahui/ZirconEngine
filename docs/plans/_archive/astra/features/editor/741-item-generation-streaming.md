---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/226-editor-asset-workspace-content-browser-current-source-review.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-item-generation-streaming.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
tests:
  - zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs
  - tools/tests/test_editor_asset_item_generation_streaming_performance_contract.py
---

# Editor asset item-generation streaming construction

`AssetWorkspaceItemGeneration` now builds UUID/locator indexes, selected
indices, and 64-item chunks directly from its input iterator. The old
`FromIterator` temporary item `Vec` is gone; `From<Vec<_>>` and
`FromIterator<_>` share the same single-pass builder. Row order, chunk
boundaries, lookup behavior, and duplicate-key diagnostics remain unchanged.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 / Asset Browser generation | Remove the intermediate item vector while preserving chunk/index/selection products. | TDD RED/GREEN source-pressure contract `3/3`; lower Rust iterator/index regression; scoped Rustfmt and Python compilation. Managed Cargo/Release allocation and Asset Browser latency evidence remain pending. | implemented_pending_validation |

The one-process non-tooling Runtime/Editor performance-plus-pressure loader
covers `554` modules and passes `2062/2062` tests in `16.986s`, including this
contract and the preceding Editor740/Runtime206 slices. This is shared
source/model evidence, not managed Cargo or product timing.

## 性能边界

For `N` rows the builder remains `O(N)`. A deterministic 1,000,000-row model
reduces owned item slots from `2N` to `N` by eliminating the temporary
collector; hash-map key ownership and chunk payload ownership are unchanged.
This model is not a product CPU, RSS, or p50/p95/p99 measurement.

## 受管验证

This feature joins the existing batched Runtime/Editor Windows Release lane;
no per-task Cargo run or coordinator status query was made. Keep the feature
at `implemented_pending_validation` until compilation, parity, allocation,
and latency gates are recorded. Tooling production changes remain deferred.
