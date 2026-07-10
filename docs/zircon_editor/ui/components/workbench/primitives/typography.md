---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_label.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui
implementation_files:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_label.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - TOML parse and body-font projection assertions for all four assets (2026-07-10 passed)
  - legacy `12.0|13.0` primitive font-size scan (2026-07-10 zero matches)
  - focused editor typography/token tests (2026-07-10: 19 passed, 0 failed; one screenshot test ignored in the non-ignored preference filter)
  - component captures under docs/tests/editor (2026-07-10: buttons, fields, rows, and component atlas passed visual review)
doc_type: module-detail
status: implemented-focused-and-component-visual-passed
---

# Workbench Primitive Typography

The low-level Workbench label-bearing assets use one body typography baseline before they are composed into toolbars, rows, status bars, drawers, or windows. `WorkbenchLabel`, `WorkbenchChip`, `WorkbenchSectionTitle`, and `WorkbenchStatusItem` therefore declare the same 13.33 logical-pixel body size derived from Unreal Starship's Normal 10-point font at 96 DPI.

## Geometry contract

- `WorkbenchLabel` retains 2 logical pixels of vertical padding. Its 20-pixel minimum height exactly accommodates the current 16-pixel body line plus padding.
- `WorkbenchChip` retains 4-pixel vertical and 10-pixel horizontal padding inside a 28–32 pixel adaptive height.
- `WorkbenchSectionTitle` keeps the stronger weight and 24–32 pixel adaptive height; only its body-size source is normalized.
- `WorkbenchStatusItem` inherits its parent bar height and uses the same body size as the status-bar authored asset.
- `WorkbenchComponentDrawer` button samples no longer override only Primary/Secondary with a local 12.22-pixel font. All button samples inherit the retained-host body metric, and their explicit button/icon-button radii use the shared 4-pixel control baseline.

These sizes are logical units. Display scaling is applied by the text renderer; assets must not multiply them by 96/72 or the monitor scale again.

## Validation boundary

The four atom assets and the 99-node component drawer parse as TOML. The atoms no longer contain their prior local 12/13-pixel defaults, and the drawer contains no `font_size = 12.22` or `corner_radius = 5.0` sample drift. Fresh focused tests passed from the 2026-07-10 editor test binary, and the buttons/fields/rows/component-atlas captures under `docs/tests/editor` show readable body text without vertical clipping. The 1672-pixel composite drawer still contains unrelated responsive composition defects, so this closes the atomic typography route only, not the complete window.
