---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/161/2026-08-26-alert-message-deferred-clone.md
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/127-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-28-asset-pointer-item-generation-ownership.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/surface/row/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/alert/message/capacity_tests.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/support.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/tests/host/retained_window/shell_window/template_paint.rs
---

# Editor920 Popup, Welcome, Layout and Asset Fixture Contract Repairs

## 计划完成列表

| Scope | Completed source change | Evidence | Acceptance |
| --- | --- | --- | --- |
| Popup-row surface | Use the current default `UiPainterResolvedState::Normal` in the transparent-row test fixture; preserve fill/outline assertions. | One v27 retired `Idle` variant; the runtime-interface state enum explicitly defaults to `Normal`. | implemented_pending_validation |
| Borrowed alert message | Construct the options fallback as the current `Rc<String>` type, without altering the borrowed-message production function or ignored Release benchmark. | One v27 unsupported `&str.into()` conversion. | implemented_pending_validation |
| Export-wizard borrowed model | Hold the empty pane fixture for the lifetime of its borrowed `Cow` result, preserving the owned-fallback assertion. | One v27 temporary-lifetime error. | implemented_pending_validation |
| Host layout routing | Compare the shared frame's `Arc` report through its referenced value against the surface report, preserving selection-count and JSON export assertions. | One v27 `Arc<Report>` versus `Report` mismatch. | implemented_pending_validation |
| Welcome pane/shell painting | Initialize the welcome fixture's new layout field before the existing post-construction node-layout capture; assign the expected concrete `PaneData` without wrapping it in `Arc`. | Two v27 initializer/type diagnostics; same fixture already calls `compile_welcome_pane_layout` after construction. | implemented_pending_validation |
| Asset pointer content | Import the already-exposed `AssetPointerContentRoute` in the nested test module, keeping the UUID-preserving route assertion. | One v27 unresolved enum; bridge return type is `Option<super::AssetPointerContentRoute>`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows/surface/row/style.rs` | `458A3E3692962335A2E67FEB57C1B7C9426E661D6E3B2E16F94CE9198C636E4A` |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_primitives/alert/message/capacity_tests.rs` | `989658C064F291D27B28DCDC0895E021B3E8ECD5722412BDBF557CEE7461CA07` |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export_wizard_panel.rs` | `399CD4FC9BD53CE8C6B0A268D7381F9785F146F8DA36DEAB6D4772F04AF626BA` |
| `zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs` | `C982B97FAE83C27FE3B285C4A9C096E5BEF7290ACB9709CDDBB7CC3FB5EB2225` |
| `zircon_editor/src/tests/host/retained_window/shell_window/support.rs` | `FAFD84F350B768BC8B4815092FE2936327A57D8E4259B431EFC94D30D3A51C74` |
| `zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs` | `34C72DFAB4061F43B8B1E04A49B8665F79D1DF49EA9797A2CAE4695740899A3C` |
| `zircon_editor/src/tests/host/retained_window/shell_window/template_paint.rs` | `9C6DC21A026BB58EC062C6D40048F0BB23DACF84A6FD87B15E71E957B39D1773` |

## Managed gate

All seven post-v28 clean files require grouped managed Rust compilation
and tests. The preceding batch did not reach Cargo, and a one-time admission
check encountered a separate active blocking lease. No extra batch was
queued, monitored, or incorrectly counted as Rust/Release/product evidence.
