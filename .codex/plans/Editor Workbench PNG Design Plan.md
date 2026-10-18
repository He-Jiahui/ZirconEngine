# Editor Workbench PNG Design Plan

## Summary
Create and maintain a deterministic PNG design corpus for the editor UI, using [workbench.png](/E:/Git/ZirconEngine/docs/ui-and-layout/workbench.png) as the strict visual baseline: dense dark panels, low-radius surfaces, 1px separators, teal active states, compact controls, and no marketing-style card layout.

Use a mixed pipeline: HTML/CSS mockups for all UI chrome and controls, generated or curated raster visuals only inside Scene viewport and asset thumbnails.

The exact pixel target is carried by `zircon_editor/assets/ui/editor/reference/workbench.png`, which must remain byte-identical to `docs/ui-and-layout/workbench.png` through direct byte comparison in `verify-designs.mjs`. The exported design corpus is the broader state and workflow reference set that translates that baseline's density, scale, and control treatment across editor surfaces.

## Deliverables
- Output directory: `docs/ui-and-layout/editor-workbench-designs/`
- `design-manifest.mjs` is the current output contract: 270 design entries at `1672x941`, plus `preview-sheet.png` for browser review.
- Core full workbench PNGs:
  - `scene-workbench.png`
  - `hierarchy-workbench.png`
  - `inspector-workbench.png`
  - `asset-browser-workbench.png`
  - `console-workbench.png`
  - `project-overview-workbench.png`
- Focused PNGs showing enlarged window/panel states:
  - Scene viewport toolbar and gizmo states
  - Hierarchy tree selection and context menu
  - Inspector transform/material/component sections
  - Asset browser grid/list/import states
  - Console log/filter/detail states
  - Project overview dashboard/actions states
- Expanded editor-page batches for production coverage:
  - tool/window pages, layout specs, state specs, drawer content specs, overlay specs, workflow specs, and floating windows
  - content, diagnostics, infrastructure, collaboration/release, cinematic/animation, world-building, gameplay/runtime, platform/online, UI/UX, rendering/GPU, audio/voice, physics/simulation, AI/navigation, asset pipeline/DCC, engineering/production, project governance, runtime QA, graphics deep-dive, animation production, UI diagnostics, world streaming pages, and LiveOps pages

## Key Changes
- Extend `tools/editor-workbench-preview` into a design-export preview tool instead of relying on freeform image generation.
- Add richer mock data for Scene, Hierarchy, Inspector, Assets, Console, and Project Overview so PNGs are not empty fixture shells.
- Add a `workbench-strict` visual theme derived from the reference image and `editor_material.zui`, favoring:
  - near-black surfaces `#101417` / `#16191c`
  - panel fills around `#181c20` / `#202830`
  - teal accent `#35c7d0`
  - compact 28-32px controls
  - 0-8px radius, mostly 1px borders
- Add screenshot export support using a browser automation path; if Playwright/Chromium is unavailable, install/use the project-local browser toolchain during execution.
- Use generated/curated raster imagery only for viewport backgrounds and thumbnail content, never for UI text or controls.
- Treat `tools/editor-workbench-preview/design-manifest.mjs` and `tools/editor-workbench-preview/design.js` as a strict paired registry: every rendered design id must be exported, and every manifest output must render.
- Keep `tools/editor-workbench-preview/preview-sheet.js` as the compact manifest overview renderer so `preview-sheet.png` covers every design id on the fixed `1672x941` canvas instead of silently clipping a long card list.
- Use `tools/editor-workbench-preview/verify-designs.mjs` as the acceptance gate for the pinned docs `workbench.png` and editor runtime reference asset SHA-256/size/dimensions baselines, direct byte comparison, exported PNG dimensions, PNG structure, manifest/renderer consistency, preview-sheet coverage/freshness, per-design PNG freshness against the rendering inputs, package script definitions, output-directory drift, `STYLE-NOTES.md` coverage/freshness, plan/export/index documentation markers, stale preview/export process cleanup, export option parsing, and broad dark-workbench visual profile.

## Review Flow
- Produce or refresh `preview-sheet.png` whenever the manifest changes.
- Use `npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=<id-list> --no-sheet` for targeted page refreshes.
- Use `npm --prefix tools/editor-workbench-preview run design:export:only -- --ids=sheet` to refresh the browser review sheet.
- For layout review passes, prioritize overall interaction structure over local content detail: main editor tabs, dock drawer slots, central workspace ownership, bottom console/timeline placement, and compact flat control rhythm.
- Keep `STYLE-NOTES.md` beside the PNGs as the reusable workbench rule summary for future editor UI generation.

## Test Plan
- Verify every manifest PNG and `preview-sheet.png` are exactly `1672x941`.
- Verify the source `workbench.png` reference and `zircon_editor/assets/ui/editor/reference/workbench.png` still match SHA-256 `4AD7706C08138EF422802C0C46B5DE4775237021F474200EFC100DAD577C02D0`, byte length `1526533`, and `1672x941`.
- Negative guard: first confirm a different same-size export such as `scene-workbench.png` is ignored when passed as `ZIRCON_WORKBENCH_RUNTIME_REFERENCE_OVERRIDE` without `ZIRCON_WORKBENCH_REFERENCE_NEGATIVE_GUARD=1`, then enable the guard flag, confirm direct byte comparison fails, and confirm the real `zircon_editor/assets/ui/editor/reference/workbench.png` remains byte-identical.
- Run `npm --prefix tools/editor-workbench-preview run design:verify:reference-negative` for the automated version of that negative guard.
- Compare all drafts against `workbench.png` for density, panel hierarchy, accent color, separator weight, and control scale.
- Check text is legible and not clipped in every draft.
- Confirm UI controls are rendered by HTML/CSS, not AI-generated raster text.
- Confirm final files are saved under `docs/ui-and-layout/editor-workbench-designs/`.
- Run `npm --prefix tools/editor-workbench-preview run design:verify`; the current passing baseline is 271 PNG files: 270 design entries plus `preview-sheet.png`.

## Assumptions
- The existing `workbench.png` is the visual authority.
- The intended output is design reference PNGs, not immediate Rust/Slint/ZUI implementation.
- Future implementation should use these PNGs and the style note as the target for making actual editor UI closer to the workbench style.
