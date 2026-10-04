---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy/viewport.rs
  - zircon_editor/src/tests/editing/transaction_engine/journal_scene_replay.rs
---

# Editor913 Hierarchy Viewport and Journal Test Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Hierarchy viewport paint metadata | Import the existing `TemplatePaneNodeData` next to `TemplateNodeFrameData` in the test module; preserve candidate geometry, preserved metadata, and viewport-frame assertions. | v27 reported three invalid `super::...::data` paths in this clean owner; production viewport behavior remains unchanged. | implemented_pending_validation |
| Scene journal replay | Explicitly unwrap the existing `Result<bool>` in the deleted-node check and require an actual restored node after decoding each of two `Option<String>` names. Preserve matched-baseline and transaction replay assertions. | v27 reported two string/optional mismatches and one `!` applied to a `Result` in this clean owner. Exact Rustfmt and diff checks pass. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy/viewport.rs` | `856345F6AB573330D675D5F702B1C61AD3392A32DA525D4802E93499F59DDD5E` |
| `zircon_editor/src/tests/editing/transaction_engine/journal_scene_replay.rs` | `E5AFD22DAE30CC14D5FA39D226F8845E89F164A1A6EB9CB7EF855E78C083BDC` |

## Managed gate

These two clean test repairs postdate v28 admission. Group them into a
later managed current-source Editor regression; no ignored Release,
allocator, or product percentile evidence follows from local static checks.
