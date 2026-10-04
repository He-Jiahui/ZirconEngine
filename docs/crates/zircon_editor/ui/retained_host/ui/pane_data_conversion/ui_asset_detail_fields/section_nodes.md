---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/row_model.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/section_nodes.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/section_nodes.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - detail_rows_use_workbench_body_typography_with_sufficient_height
  - scoped rustfmt and legacy numeric-font static scan (2026-07-10 passed)
  - focused editor Cargo detail_rows_use_workbench_body_typography_with_sufficient_height (2026-07-10: 1 passed)
  - refreshed component and Workbench screenshots under docs/tests/editor (2026-07-10)
doc_type: module-detail
status: implemented-focused-and-visual-passed
---

# UI Asset Detail Section Nodes

`section_nodes.rs` expands an authored UI-asset detail section into paired label and editable-value rows. The owner grows the section when rows do not fit, shifts following nodes by the same growth, computes a relative label/value split from the available width, and emits commit/draft binding identifiers for enabled fields.

## Typography contract

Both the label and input-field node project `EditorTypographyTokens::WORKBENCH_BODY_SIZE`. Unreal Starship's Normal 10-point baseline is converted once by the shared token owner to 13.33 logical pixels at 96 DPI; the detail projection must not reintroduce local `10/11px` defaults or apply the point conversion a second time.

The row projects `EditorDensityTokens::WORKBENCH_ROW_HEIGHT` (28 logical pixels), matching the shared dense workbench row instead of the previous local 22-pixel value. This leaves 12 pixels around the current 16-pixel body line and matches the roughly 28–30 pixel detail fields in the AI workbench reference. `detail_rows_use_workbench_body_typography_with_sufficient_height` guards the semantic size, token-owned row height, and minimum vertical room so a later token change cannot silently clip inspector text.

## Boundaries

- This module owns row projection and relative label/value geometry only.
- Runtime shaping, glyph measurement, rasterization, and DPI scaling remain in the runtime text subsystem.
- The caller owns section models and authored base nodes; this module does not introduce a second preferences store or a compatibility typography path.

## Current validation state

Scoped formatting, diff checks, the selected chrome/detail legacy-number scan, and `detail_rows_use_workbench_body_typography_with_sufficient_height` pass. The refreshed component and Workbench captures were written under `docs/tests/editor`, and the target scan found no matching validation PNG. This closes the detail-row typography and minimum-height contract; full inspector composition remains part of later composite/window work.
