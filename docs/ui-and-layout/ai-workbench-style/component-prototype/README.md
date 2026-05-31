# Workbench Component Prototype

This prototype rebuilds `E:/Git/ZirconEngine/docs/ui-and-layout/workbench.png` as a component stack inside the normalized `ai-workbench-style` design workspace instead of wrapping the reference PNG.

## Build Order

1. `tokens.css` extracts the fixed dark editor palette, 1672x941 reference dimensions, compact 30 px controls, 1 px separators, low radii, and teal active states.
2. `icons.js` defines local inline SVG icons, so buttons and tree/list rows are not dependent on an icon package.
3. `atoms.js` and `atoms.css` provide buttons, icon buttons, inputs, checkboxes, radio buttons, switches, tabs, segmented controls, selects, number fields, and sliders.
4. `collections.js` and `collections.css` compose tree view, table view, list view, popup menu, alerts, tooltip, and toast.
5. `surfaces.js` and `surfaces.css` compose the top toolbar, left rail, scene drawer, CSS scene viewport, inspector window, UI component drawer, and status bar through explicit `workbenchWindow`, drawer/window surface, and panel-view helpers.
6. `app.js` mounts the window helper and wires small interactions for tab switching, panel view switching, tree/list/table selection, toggles, and dropdown popup placement.

## Fidelity Rules

- UI chrome and controls are HTML/CSS/SVG components.
- The whole `workbench.png` is not embedded as a screenshot.
- The central scene uses `assets/workbench-viewport-reference.png`, a crop of only the Scene viewport from the reference. UI chrome, drawers, controls, text, and overlays remain HTML/CSS/SVG components.
- The 1672x941 viewport is the strict reference size for visual QA. Smaller viewports use responsive media queries while preserving the same component hierarchy.

## Preview

Serve this directory with a static server and open `index.html`. No build step, CDN, web font, or third-party runtime is required.

The default view matches the reference workbench. The secondary panel tabs are also wired for interactive review: `Layers` swaps the left drawer into a layer list, `History` swaps the inspector into an action history, and `Console` swaps the component drawer into a log panel. These alternate views are hidden by default so they do not affect the pixel audit baseline.

## Pixel Audit

After capturing a `1672x941` candidate screenshot, run `verify-pixel-audit.ps1` from this folder. The audit compares the candidate against `../../workbench.png` by major workbench region and reports sampled RGB deltas. The viewport is the only raster-backed region; remaining differences should come from HTML/CSS component geometry, icons, text metrics, and panel styling.

Use `.\verify-pixel-audit.ps1 -Step 4` for the normal iteration pass. Use `.\verify-pixel-audit.ps1 -Only inspector-transform -Step 2` for denser checks on stubborn regions where text anti-aliasing and icon strokes dominate the score.

Use `.\verify-component-audit.ps1 -Step 2` for fine-grained component iteration. It breaks the stubborn workbench regions into transform rows, control columns, table rows, renderer subgroups, status bar sections, alerts, tooltip, and toast so style changes can be measured against the exact component they are meant to improve.

Current acceptance target is region-by-region convergence: the raster-backed viewport should stay below `12` average RGB delta, while pure HTML/CSS regions should keep moving toward `25` or below before claiming pixel-level completion.

Run `node verify-interaction-contract.mjs` to check that the composed component tree exposes the expected tab targets, popup layer, selection hooks, toggles, dropdown placement hook, and `aria-selected` updates.

Run `node validate-responsive.mjs` to check the same component stack at `1672x941`, `1440x900`, `1200x820`, `1040x760`, `720x760`, and `640x720`, plus a live resize sequence. The validator rejects horizontal overflow, missing core workbench regions, missing atom/collection/surface components, and accidental full-reference screenshot embedding.

Run `node export-screenshots.mjs` to regenerate `_screenshots/workbench-1672x941-final.png` and `_screenshots/workbench-720x760.png` with headless Edge before each audit pass.

Current local validation covers JavaScript syntax, the interaction contract script, no external resource dependency, HTTP preview availability, desktop/mobile screenshots, and the region pixel audit. The interaction contract also asserts the top-level window surface, drawer/window surfaces, and panel-view surfaces so the prototype remains a bottom-up component stack instead of a single page-specific string. Tab, segmented control, and panel tab interactions now keep both `.is-active` and `aria-selected` in sync for browser review. Automated click testing was attempted through Playwright, but the local CLI can capture screenshots while the Node test/runtime package is not importable in this folder, so interaction behavior is validated by source-level contract plus browser-preview behavior rather than a committed browser automation test.
