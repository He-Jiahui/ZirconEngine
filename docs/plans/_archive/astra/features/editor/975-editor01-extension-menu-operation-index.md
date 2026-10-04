---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-extension-menu-operation-index.md
related_records:
  - docs/plans/astra/features/editor/973-editor01-chart-raster-arc-cache.md
  - docs/plans/astra/features/editor/974-editor01-circular-progress-hash-arc-cache.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
tests:
  - zircon_editor/src/ui/workbench/model/menu/extension_menu/operation_index_tests.rs
---

# Editor975 Editor01 extension-menu operation index

Extension-view menu publication now builds one `HashSet<EditorOperationPath>`
from the completed nested menu tree and reuses it for contributed menu items
and views. Duplicate suppression, nested operation coverage, command enablement,
ordering, and first-view insertion remain unchanged; the recursive
per-contribution `item_contains_operation` scan is removed.

## Local grouped evidence

The current-source Editor library binary ran the complete
`optimization_batch_20260826ak_` selector in one invocation: `3/3` passed with
no failures. The source/behavior contracts passed together with the ignored
marker:

`EDITOR01_EXTENSION_MENU_OPERATION_INDEX_BENCH_V1 baseline_p95_ns=780726100 optimized_p95_ns=34318800`

The local debug P95 reduction is `95.60%`; the optimized path is `4.40%` of
the recursive-scan baseline and clears the plan's `60%` maximum-ratio gate.
Managed Editor Release and product-scale validation remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/model/menu/extension_menu.rs` | `BC18612E957881725CE808286E67A4406678AF3FFC0C8376C9CB09301FE5F760` |
| `zircon_editor/src/ui/workbench/model/menu/extension_menu/operation_index_tests.rs` | `E05F1982CE876DEAA21B09F0BF4C8944A7B6D7667B092D6F3DD8E7871C489E58` |
