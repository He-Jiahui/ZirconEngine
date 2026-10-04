---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/23/2026-08-26-preview-suggestion-borrowed-root.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_lab_field_edit.rs
  - zircon_editor/src/ui/asset_editor/preview/mock_suggestions/borrowed_root_tests.rs
---

# Editor916 Component Lab and Preview Test Fixture Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Component lab search field edit | Name the authored `WorkbenchInputSearch` control ID once inside the test module so both field mutation and query retrieval address the same declared search control. | v27 reported two missing `SEARCH_CONTROL` uses; read-only inspection of the authored workbench component drawer confirms this ID owns the `ComponentLab/InputSearchEdit` route. | implemented_pending_validation |
| Preview mock suggestion borrowed-root fixture | Import the existing private mock-kind enum from the nested test's grandparent owner to construct object/text fixtures; preserve the 4,096-branch ignored P95 benchmark and nested-key assertions. | v27 reported two missing enum uses in the clean test owner. Local Rustfmt and scoped diff checks pass; no product mock implementation was changed. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_lab_field_edit.rs` | `571CCC57578FFE25FA84921B41DB8B10A17719ADE28E83778FD67739A47C0369` |
| `zircon_editor/src/ui/asset_editor/preview/mock_suggestions/borrowed_root_tests.rs` | `70FF93D85CA9B23A202A96D91474BC2C34BE3A4E73E48E98FC7C0EEE84B4CE02` |

## Managed gate

Both clean test changes postdate v28 admission. Later grouped current-
source managed Editor regressions and the exact ignored Release filter are
required to establish correctness and the claimed 90%-lower borrowed-root
P95; allocation and product p50/p95/p99 remain pending.
