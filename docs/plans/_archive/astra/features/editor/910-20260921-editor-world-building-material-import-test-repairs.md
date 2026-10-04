---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/426/2026-08-31-component-variant-length-prefilter.md
  - docs/plans/optimize/zircon_editor/138-editor-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/shared.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building.rs
  - zircon_editor/src/tests/editing/import.rs
---

# Editor910 World Building, Material, and Import Test Repairs

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Component variant prefilter tests | Import the present `TemplatePaneNodeData` in the private child test module, keeping case-insensitive equality and the ignored Release marker unchanged. | v27 reported four missing-type references and a derivative inference error in this clean owner. Rustfmt and diff checks pass. | implemented_pending_validation |
| World-building extension navigation | Validate the row and command control/action pairs and field namespace still present in `ExtensionNavigationSpec`; stop asserting removed tab fields, retaining each existing action/control consistency check. | v27 reported five references to removed tab fields in this clean test owner; no foreign shared spec implementation was edited. | implemented_pending_validation |
| Imported-mesh undo | Read world counts and imported node records via the authoring-world callback, preserving mesh-kind and undo assertions without trying to call `node_records` on `Result<Option<World>>`. | v27 reported three invalid world-snapshot calls in this clean test owner. Rustfmt and diff checks pass. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/shared.rs` | `3D238D206AE288105B1661A4F480829E2C6C442A4633E752262101F79DE46F2F` |
| `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building.rs` | `5C7716BC373ABAC343F2AD56180FA3F4D4146CA5CEF02827D621933EB0FF10BF` |
| `zircon_editor/src/tests/editing/import.rs` | `5AC6A27E686C37E20557109CEA5972CD54B558E2A096FA5C70E49447936494E5` |

## Managed gate

These test-only repairs postdate v28 admission. One grouped later managed
Runtime/Editor source-bound Rust regression must qualify them, followed by
the exact Editor426 ignored Release marker and relevant product allocation
and p50/p95/p99 gates. No compile or performance acceptance is claimed from
local formatting and diff checks.
