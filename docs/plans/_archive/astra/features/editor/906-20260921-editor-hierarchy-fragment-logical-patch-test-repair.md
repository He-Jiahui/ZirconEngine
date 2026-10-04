---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_fragment.rs
---

# Editor906 Hierarchy Fragment Logical Patch Test Repair

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Large and unmaterialized rename patches | Inspect entity, name, and depth on the typed `replacement()` content, preserving row-index, no-reflow, and unchanged-control checks. | v27 Editor check reported four missing field accessors on `SceneHierarchyLogicalRowPatch`; the current foreign-modified projection owner exposes the content only on a replacement patch. | implemented_pending_validation |
| Selection-only patch | Locate the unmaterialized row by its stable row index, assert `replacement().is_none()`, and retain its selected-state assertion. | v27 reported one further missing entity accessor; the present projection owner constructs selection-only patches with no replacement payload. Local Rustfmt and scoped diff checks pass. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_fragment.rs` | `F8B618EB22BE5D98A015B692508529693482A31803A7EEF403FE6271C1D25945` |

## Managed gate

This clean test-only repair postdates terminal v27. The shared projection
owner remains foreign modified and was left untouched. A grouped current-
source managed Editor regression run must still establish correctness; the
ten-thousand-row reflow, allocator, and product p50/p95/p99 performance gates
remain pending.
