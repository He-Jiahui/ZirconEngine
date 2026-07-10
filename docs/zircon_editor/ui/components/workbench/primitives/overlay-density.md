---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_context_menu.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_dropdown_popup.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_popup_menu.zui
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_overlay_density.rs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_context_menu.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_dropdown_popup.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_popup_menu.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - authored_workbench_menus_share_slate_popup_density_and_flat_surface
  - Python TOML parse and exact popup-metric contract (2026-07-10 passed)
  - focused Cargo authored overlay-density contract (2026-07-10 1/1 passed)
  - component popup-selection capture (2026-07-10 passed)
doc_type: module-detail
status: implemented-focused-and-component-visual-passed
---

# Workbench Overlay Density

Workbench context menus, dropdown popups, and action menus share one authored popup density before domain-specific menu contents are projected. Each root now declares 8-pixel horizontal and 3-pixel vertical padding, a 4-pixel item gap, and a 28-pixel minimum row height. The popup surface uses a 4-pixel corner radius, a 1-pixel border, and zero elevation so it follows the planned Unreal/Slate flat-menu hierarchy instead of Material shadow elevation.

Popup placement remains relative to its anchor metadata. These authored density values do not replace `template_popup_layout` or native edge-flip/clamp behavior; they only standardize the content box and surface presentation consumed by those owners.

The focused contract lives in the dedicated 75-line `workbench_overlay_density.rs` leaf rather than extending the already oversized primitive-governance file. Focused Cargo passed 1/1. The retained component route refreshed `docs/tests/editor/editor-components-popup-selection-list-900x360.png` (48,815 bytes, SHA256 `6A21EA440D4197875EF993607D4D74B20E1FB2B5C59B8BE0258BE015FC9EBB99`), and a known-output scan found no matching PNG in the external Cargo target.

The 900x620 window-menu artifact was also rerun and retained SHA256 `82A38BE46DD1CE1746AEBD45899F68BEB433FC8414E61D33543A7824AFC159B7`. That route is rendered by the native `HostMenuStateData` menu path rather than these authored `.zui` popup primitives. Its content-measured width, viewport-constrained height, and scroll behavior therefore remain a separate responsive-menu slice; this atomic overlay-density closeout does not claim that native window menu is visually complete.
