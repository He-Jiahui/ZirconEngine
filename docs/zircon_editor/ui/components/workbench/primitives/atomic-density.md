---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_field.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_dropdown.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_number_field.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_search_input.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_segmented_control.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_list_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_tree_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_table_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_property_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_component_property_row.zui
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_atomic_density.rs
implementation_files:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_field.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_dropdown.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_number_field.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_search_input.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_segmented_control.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_list_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_tree_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_table_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_property_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_component_property_row.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15b-host-control-metrics-single-source.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - authored_workbench_buttons_match_compact_unreal_density
  - authored_workbench_fields_match_unreal_input_padding_and_height
  - authored_workbench_tabs_use_slate_padding_and_theme_surface
  - authored_workbench_data_rows_share_the_compact_row_height
  - Python TOML parse and exact authored-metric contracts (2026-07-10 passed)
  - focused editor Cargo workbench_atomic_density (2026-07-10: 4 passed, 0 failed)
  - component screenshots under docs/tests/editor (2026-07-10: buttons, fields, rows, atlas passed visual review)
doc_type: module-detail
status: implemented-focused-and-component-visual-passed
---

# Workbench Atomic Density

This module contract standardizes authored Workbench controls before they are assembled into toolbars, tables, inspectors, popups, drawers, or windows. The `.zui` assets provide responsive min/preferred/max constraints; the retained host remains the owner of painting, typography, palette roles, hit testing, and interaction state.

## Control geometry

- Text buttons use the compact 28/30/32 logical-pixel height band, 12-pixel horizontal padding, a 7-pixel icon/text gap, and a 16-pixel icon. Their body text is inherited from the host typography token instead of being repeated in each asset.
- Icon buttons use a 32-pixel preferred square with a 16-pixel glyph and can grow to 40 pixels when their container grants room. This keeps the authored primitive suitable for both dense toolbar and roomier panel contexts without absolute screen coordinates.
- Text fields, dropdowns, number fields, and search inputs use the planned Unreal editable-field vertical padding of 3/4 pixels and the compact 28/30/32 height band. Plain fields use 8-pixel horizontal padding; search fields reserve their leading and trailing icon slots while sharing the same vertical baseline.
- Tabs use the planned 4/10/3/4 padding, a 4-pixel content gap, and a 28-pixel minimum height. The tab strip no longer authors a private background hex; its surface comes from the central theme palette. Tab strips and segmented controls share the 28/30/32 height band.

## Data-row geometry

List, tree, table, property, and component-property rows share the 28-pixel workbench row baseline. The general rows expose 8-pixel horizontal padding and a 4-pixel internal gap. Row width remains stretch-based and tree indentation remains relative to tree depth, so the density correction does not introduce screen-pixel positioning.

## Validation boundary

`workbench_atomic_density.rs` is a dedicated leaf test module because the existing primitive-governance test file is already above the repository's large-file threshold. It loads the actual authored assets through the runtime `.zui` loader and checks exact semantic metrics. All fourteen changed assets also parse with Python TOML and pass the same authored-metric contracts.

The fresh editor test binary executed all four density contracts successfully. It then refreshed the buttons, fields, rows, component atlas, 640/900/1260 acceptance set, and 1672 run-mode image under `docs/tests/editor`; no matching image exists in the repository or external Cargo target used by this session. Manual review accepts the atomic captures, while the 1672 composite drawer still exposes clipped labels and poor responsive distribution. That composite defect remains a separate open S15.4/S15.5 slice and is not hidden by this atomic completion state.
