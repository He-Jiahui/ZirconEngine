# AI Workbench HTML/CSS Prototype

This prototype normalizes the AI-generated layout references in `docs/ui-and-layout/ai-workbench-style/` into one responsive, interactive workbench design. It is not a screenshot wrapper and does not import third-party UI libraries.

## Source References

The prototype uses the AI PNGs as visual direction only:

- `ai-workbench-web-framework.png` for the shared shell rhythm.
- `ai-scene-editor-layout.png` for scene viewport, placement, hierarchy, inspector, and console composition.
- `ai-material-editor-layout.png` for graph editor and material details.
- `ai-ui-asset-editor-layout.png` for UI canvas, widget tree, properties, and validation.
- `ai-montage-editor-layout.png` for animation preview and timeline.
- `ai-asset-browser-layout.png` for asset table, filters, metadata, and queue output.
- `ai-runtime-diagnostics-layout.png` for runtime metrics and event output.
- `ai-project-overview-layout.png` for project dashboard structure.

AI-generated inconsistencies are intentionally corrected instead of copied exactly: panel radius, top-bar density, drawer widths, active colors, field sizing, page tabs, and bottom output placement all follow one shared rule set.

The current visual target is `ai-workbench-web-framework.png`: deeper near-black chrome, subtle blue-green panel gradients, low-radius controls, compact text, teal active states, wider scene/inspector drawers, and a thin status bar. Taffy inspection controls are kept as an optional overlay so they do not dominate the main editor composition.

## Prototype Rules

- Responsive layout built with CSS grid and flexbox.
- No full-reference screenshots are embedded into the UI.
- No third-party runtime libraries, icon packs, web fonts, or CSS frameworks are used.
- Viewport imagery, graph nodes, material preview, UI screen preview, timelines, metrics, and thumbnails are CSS-generated placeholders.
- Controls use a compact editor scale: 28-32 px height, 1 px borders, low radius, near-black surfaces, teal active states.
- Page switching, drawer tab switching, tool selection, drawer collapse, output collapse, layout presets, dock-preview targets, selected layout-region highlighting, and overlays are implemented with small plain JavaScript state.

## Rust UI Migration Mapping

- `top-bar` maps to a retained shell command strip.
- `page-tabs` maps to main editor document tabs.
- `tool-rail` maps to a stable vertical command rail.
- `drawer-column`, `dock-panel`, and `panel-body` map to docked pane slots.
- `center-stage` maps to the active editor viewport/document surface.
- `bottom-output` maps to console, timeline, validation, build, and queue panes.
- `layout-tray` maps to a debug-only layout inspector and shows the currently selected Taffy-oriented node contract.
- CSS custom properties under `:root` are the design-token source for dark surfaces, text colors, separators, active teal, control radius, and compact sizing.
- `--left-drawer-width`, `--right-drawer-width`, and `--bottom-output-height` are explicit sizing tokens for Taffy translation. The layout presets change these values through root classes instead of hard-coded pixel edits.
- Page data in `page-data.js` is intentionally declarative so Rust-side view models can provide the same fields without preserving DOM-specific structure.
- `stage-renderers.js` isolates document surface drawing from shell state, matching the future split between workbench chrome and per-editor viewport/content renderers.

## Taffy-Oriented Interaction Notes

- The top `Authoring`, `Review`, `Focus`, and `Debug` buttons are layout presets. They should map to retained layout profiles rather than separate editor windows.
- The `Left drawer`, `Right drawer`, and `Bottom` segmented controls are manual sizing tokens. In Taffy, treat them as constraints applied to the corresponding dock slot node. They open from the `Inspect` status-bar action instead of occupying permanent layout space.
- The `Layout` buttons on the center stage and bottom output select the inspected node. Drawer columns can also be selected through their dock region.
- The floating `Left / Center / Right / Bottom / Clear` dock overlay simulates a future drag/drop preview. It currently highlights a target slot without moving panels, keeping the prototype focused on layout semantics.
- Compact viewports collapse side drawers by default and keep the same main `center-stage` plus `bottom-output` node order. This matches a Taffy tree where side dock nodes can become overlay nodes without changing page ownership.

## Preview

Open `index.html` directly or serve this folder with any static file server. The page does not require a build step. CSS is split into `base.css`, `shell.css`, `panels.css`, `stages.css`, and `responsive.css` so each responsibility can map cleanly to Rust-side layout/style modules later.
