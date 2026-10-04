---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime UI Submission Segment Test Compile Repair

The public Runtime UI frame test still treated `UiRenderSubmission` as the
retired flat extract DTO. The assertion now follows the current immutable
segment contract: it obtains the first submission segment, compares its
`UiRenderFrameExtract` with the surface frame, projects the compatibility
extract explicitly, and reads `tree_id`/commands from that extract. No render
ordering or submission behavior changes; the repair keeps the test aligned
with the segmented submit path described by Runtime11C.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11C public UI submit contract | Updated `runtime_ui_layout_routes` to inspect `UiRenderSubmission::segments()` and the segment extract instead of comparing an `Arc<UiRenderSubmission>` to a flat `UiRenderExtract`. | Runtime, Editor, and Runtime Text contract suites passed in one batched run; source assertions and whitespace checks passed. | implemented_pending_validation |

## Batched local evidence

- Latest parallel Runtime performance-contract discovery passed `1146/1146`
  in `27.925s`; Editor passed `581/581` in `10.757s`; Runtime Text passed
  `143/143` in `11.506s` (`1870/1870` aggregate).
- Scoped Rustfmt parse/check and `git diff --check` passed; Git emitted only
  existing line-ending notices.

## Managed acceptance gate

This is a test-contract repair, not a managed Rust compile or product
performance receipt. The combined managed Release requests remain blocked
before ticket creation by the external `E:\\Git\\zr_vm` dirty-worktree gate
recorded in 696. Keep this row at `implemented_pending_validation` until an
owner-attributed batch reports Windows compile/test results and the required
p50/p95/p99 performance measurements.

Tooling changes remain deferred by request.
