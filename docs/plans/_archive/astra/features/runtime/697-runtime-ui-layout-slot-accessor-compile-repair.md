---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime UI Layout Slot Accessor Compile Repair

The Runtime UI layout and template tests were still reaching through the
private `UiTree::slots` field after the tree contract converged on accessor
ownership. The tests now use `layout_slots()` for read-only projections and
`push_layout_slot()` for fixture construction. This keeps slot-order and dirty
revision bookkeeping on the tree owner while preserving the existing layout,
overlay, canvas, grid, flow, and asset assertions.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime76 / Runtime11A layout-slot consumers | Replaced direct private-slot reads in block-box, responsive MUI, template slot, template grid/flow, v2 asset, and surface dirty-domain tests with `layout_slots()`; replaced four fixture `Vec::push` calls with `push_layout_slot()`. | No direct `.slots` access remains in the six affected test files; six accessor-based fixture insertions are present. Batched Runtime, Editor, and Runtime Text contract suites passed. | implemented_pending_validation |

## Batched local evidence

- Latest parallel Runtime performance-contract discovery passed `1146/1146`
  in `27.925s`; Editor passed `581/581` in `10.757s`; Runtime Text passed
  `143/143` in `11.506s` (`1870/1870` aggregate).
- Scoped Rustfmt parse/check and `git diff --check` passed; Git emitted only
  existing line-ending notices.

## Managed acceptance gate

The accessor migration is a compile-contract repair and does not by itself
constitute a Windows Release or product-performance receipt. Managed Cargo
admission remains deferred by the external `E:\\Git\\zr_vm` dirty-worktree
gate documented in record 696. The row therefore remains
`implemented_pending_validation` until an owner-attributed multi-task ticket
supplies compile/test and p50/p95/p99 evidence.

Tooling changes remain deferred by request.
