---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_caption.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_label.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui
implementation_files:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_caption.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_label.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - TOML parse and body-font projection assertions for all four assets (2026-07-10 passed)
  - legacy `12.0|13.0` primitive font-size scan (2026-07-10 zero matches)
  - focused editor typography/token tests (2026-07-10: 19 passed, 0 failed; one screenshot test ignored in the non-ignored preference filter)
  - component captures under docs/tests/editor (2026-07-10: buttons, fields, rows, and component atlas passed visual review)
  - Blend Space caption composition static audit (2026-07-12: 13 caption consumers, zero L4 font-size/font-weight overrides; current binary contract passed, while two-size native screenshots remain visual-failure evidence because the extension subtree content is not yet painted)
doc_type: module-detail
status: implemented-caption-and-section-title-current-source-verified
---

# Workbench Primitive Typography

The low-level Workbench label-bearing assets own typography before they are composed into toolbars, rows, status bars, drawers, or windows. `WorkbenchLabel`, `WorkbenchChip`, `WorkbenchSectionTitle`, and `WorkbenchStatusItem` declare the shared 13.33 logical-pixel body size derived from Unreal Starship's Normal 10-point font at 96 DPI. `WorkbenchCaption` owns the compact 10.67 logical-pixel (8-point) secondary caption used for uppercase group headers and chart-axis annotations; L4 workspaces must consume that primitive instead of writing local `font_size` or `font_weight` props.

## Geometry contract

- `WorkbenchLabel` retains 2 logical pixels of vertical padding. Its 20-pixel minimum height exactly accommodates the current 16-pixel body line plus padding.
- `WorkbenchCaption` uses the shared compact 8-point/10.67-pixel metric, weight 600, secondary tone, 2-pixel vertical padding, and an 18–22 pixel adaptive height.
- `WorkbenchChip` retains 4-pixel vertical and 10-pixel horizontal padding inside a 28–32 pixel adaptive height.
- `WorkbenchSectionTitle` now uses the Runtime body metric at weight 600 inside a compact 28–30 pixel adaptive height. Its semantic `section-title` variant drives the native painter; the themed fallback owns the flat raised surface, square corners, and one-pixel separator.
- `WorkbenchStatusItem` inherits its parent bar height and uses the same body size as the status-bar authored asset.
- `WorkbenchComponentDrawer` button samples no longer override only Primary/Secondary with a local 12.22-pixel font. All button samples inherit the retained-host body metric, and their explicit button/icon-button radii use the shared 4-pixel control baseline.

These sizes are logical units. Display scaling is applied by the text renderer; assets must not multiply them by 96/72 or the monitor scale again.

## Validation boundary

The original four atom assets and the 99-node component drawer parse as TOML. The atoms no longer contain their prior local 12/13-pixel defaults, and the drawer contains no `font_size = 12.22` or `corner_radius = 5.0` sample drift. Fresh focused tests passed from the 2026-07-10 editor test binary, and the buttons/fields/rows/component-atlas captures under `docs/tests/editor` show readable body text without vertical clipping. On 2026-07-12 the caption atom and its Blend Space consumer also parse as TOML: all 13 compact labels use `WorkbenchCaption`, while the L4 asset has zero local font-size/font-weight overrides. The later SectionTitle slice adds source, semantic-identity, Runtime-metric, palette, icon-size, and native-pixel coverage; the final current-source group passes `10/10`. The same current-source binary refreshes 640/900/1260 Blend Space captures under `docs/tests/editor`, with compact 28–30px headers, Runtime text metrics, and no target-directory screenshots. This closes authored Caption/SectionTitle ownership and its current composite use only; complete Blend Space fidelity and the broader Editor Layout goal remain active. The 1672-pixel composite drawer also retains unrelated responsive composition defects.
