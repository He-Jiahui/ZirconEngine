---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/glyph.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/toolbar_commands.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/glyph.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
plan_sources:
  - user: 2026-07-14 repair Editor Layout from the M1 failure handoff and improve shared primitives before larger composition
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp
tests:
  - workbench_toolbar_icon_and_label_keep_starship_gap_without_overlap
  - cargo test -p zircon_editor --lib --locked paint_template_nodes::template_buttons::tests --jobs 1 -- --nocapture --test-threads=1
doc_type: module-detail
---

# Workbench Toolbar Button Content

## Purpose

The retained-host Button content painter is the shared native path for workbench toolbar commands such as Save, Browse, and Compile. It places the leading SVG asset and the Runtime Text label inside the final relative button frame; the L4 toolbar remains responsible only for composition, available width, and the authored spacing value.

## Contract

The component projection carries `layout_spacing` into `TemplatePaneNodeData.layout_content_offset_x`. When that authored value is finite and positive, the painter uses it as the exact icon-to-label gap. Nodes without an authored spacing retain the host theme's `button_icon_gap` fallback, so generic and legacy callers keep the centralized design-token behavior.

Label width continues to come from Runtime Text measurement and its raster clip guard. The icon is painted through the existing SVG asset path. This change therefore does not introduce a second text renderer, toolbar-specific control-id branch, fixed window coordinate, local color, or local font override.

## Reference Alignment

Unreal Starship's `SlimToolbar` keeps compact horizontal content while separating the icon and label through shared style padding rather than feature-local geometry. Zircon follows that ownership model: `WorkbenchButton` authors its normal 7-unit atom spacing, while the top-toolbar composite intentionally authors a compact 4-unit value. The native painter now respects either value instead of silently replacing both with the host fallback.

## Validation

The focused native regression constructs a 104×30 Compile toolbar command, paints its real SVG icon and Runtime Text label, and measures their final command frames. Before the fix the authored gap was 4 but the painter emitted 7; after the fix the exact 4-unit gap is retained, the complete `Compile` ink width fits the text frame, and the ink stays inside the shared 8-unit right inset.

The complete retained Button painter group passes `53 passed / 0 failed`. The broader Editor Layout visual-refinement goal remains active; this primitive fix does not claim whole-window Unreal equivalence.

## Constraints

- Keep spacing authored by shared atoms/composites; feature windows must not add native painter exceptions.
- Keep labels on Runtime Text measurement and rendering interfaces.
- Keep all future visual captures under `docs/tests/editor`, never under a Cargo target directory.
- If a new Button layout role needs distinct padding or glyph placement, extend the shared typed projection instead of overloading action ids.
