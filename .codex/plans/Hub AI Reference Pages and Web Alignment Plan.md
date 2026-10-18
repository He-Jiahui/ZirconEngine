# Hub AI Reference Pages and Web Alignment Plan

## Summary
- Keep `docs/ui-and-layout/hub.png` as the fixed Projects Dashboard visual source.
- Generate AI full-page direction drafts for the full Hub page set, using `hub.png` as the style reference.
- Use those AI drafts as visual direction, then rebuild the final reference pages as real HTML/CSS in `docs/ui-and-layout/hub-web-reference/` so text, icons, spacing, and components stay accurate.
- Export final `1568x1003` PNGs back to the existing `docs/ui-and-layout/hub-*.png` filenames and update the visual contracts/docs to stop requiring the old deterministic Pillow generator.

## Key Changes
- Replace the current one-page web reference with a data-driven multi-page reference covering:
  `hub-editor.png`, `hub-builds.png`, `hub-assets.png`, `hub-plugins.png`, `hub-cloud.png`, `hub-team.png`, `hub-learn.png`, `hub-settings.png`,
  `hub-projects-new.png`, `hub-projects-browser.png`, `hub-projects-detail.png`,
  `hub-projects-browser-filter-menu.png`, `hub-projects-browser-sort-menu.png`, `hub-projects-detail-delete-confirm.png`, `hub-source-engine-popup.png`, `hub-user-menu.png`,
  `hub-state-empty.png`, `hub-state-loading.png`, `hub-state-error.png`.
- Add an AI prompt/selection manifest under `docs/ui-and-layout/` recording the source reference, prompt family, selected draft per page, and final HTML export filename.
- Retire `tools/generate-hub-design-assets.py` as the authoritative source for these images; replace its contract role with the AI manifest plus web-reference export script.
- Add a Hub web export script modeled after `tools/editor-workbench-preview/export-designs.mjs`, using Playwright/Edge at `1568x1003` to capture each page.
- Update `docs/ui-and-layout/index.md`, `docs/zircon_hub/ui/responsive-component-system.md`, and `zircon_hub/tests/ui_visual_standard_contract.rs` so the accepted source is “AI-directed, HTML/CSS-finalized reference PNGs,” not deterministic Pillow placeholders.

## Implementation Milestones
- Milestone 1: AI direction set
  - Generate one AI draft per listed page using `hub.png` as style reference.
  - Prompts must preserve the topbar/sidebar/button-state rhythm, dark engineering-tool palette, teal active states, compact panels, 8px-ish radius, and dense data UI.
  - Do not rely on AI text accuracy; final copy is owned by HTML/CSS.
- Milestone 2: Web reference rebuild
  - Convert `index.html` into a selectable multi-page shell.
  - Move page data/content into a small local JS module or equivalent static structure.
  - Keep the page browser-openable without a dev server.
  - Add overlays/states for filter menu, sort menu, delete confirm, source engine popup, user menu, empty/loading/error.
- Milestone 3: Export and contracts
  - Add the export script and regenerate the final `hub-*.png` files from the HTML/CSS pages.
  - Update contracts to require: exact filename inventory, `1568x1003`, non-empty images, manifest coverage, and docs matrix coverage.
  - Remove contract checks that require `ImageDraw`, `CANVAS = (1568, 1003)`, or deterministic local generator ownership.
- Milestone 4: Documentation and evidence
  - Update docs to explain the two-stage workflow: AI draft for direction, HTML/CSS export for final reference.
  - Record the export command, generated image list, and known limitation that AI drafts are not acceptance evidence; the final web captures are.

## Test Plan
- Run the new Hub web export command and verify all final PNGs are written at `1568x1003`.
- Run focused contract validation:
  `cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --test ui_visual_standard_contract`
- Run a scoped Hub check if Rust test code changes compile paths:
  `cargo check --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1`
- Visually inspect at least Projects Dashboard, Editor, Assets, Project Detail delete confirm, source engine popup, and empty/error states for no clipped text, no overlap, matching density, and consistent button/badge styling.

## Assumptions
- User-selected direction is: AI full-page drafts, full page/state scope, real HTML/CSS final web reference, and AI+web复刻 final quality policy.
- `hub.png` is not replaced.
- The final deliverables are the existing `docs/ui-and-layout/hub-*.png` filenames plus the expanded `hub-web-reference` page.
- Existing unrelated dirty worktree changes are preserved and not reverted.
