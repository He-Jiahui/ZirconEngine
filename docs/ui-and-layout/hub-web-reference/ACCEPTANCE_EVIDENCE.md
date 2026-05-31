# Hub Web Reference Acceptance Evidence

Date: 2026-05-30
Canvas: `1568x1003`

This record closes the AI-directed, HTML/CSS-finalized Hub reference PNG slice.
`docs/ui-and-layout/hub.png` remains the fixed Projects Dashboard source
reference. AI structure drafts and design-board screenshots are review support, not final acceptance evidence.

## Final Source And Export

| Item | Evidence |
| --- | --- |
| Source reference | `docs/ui-and-layout/hub.png` |
| Final page source | `docs/ui-and-layout/hub-web-reference/index.html` |
| Cover markup source | `docs/ui-and-layout/hub-web-reference/cover-rendering.js` maps project ids to local reference cover images. |
| CSS cover source | `docs/ui-and-layout/hub-web-reference/covers.css` styles reference cover images to match the fixed `hub.png` visual texture. |
| Export command | `node docs/ui-and-layout/hub-web-reference/export-pages.mjs` |
| Export port policy | Default port `5198` falls back to a free local port; explicit `ZIRCON_HUB_WEB_REFERENCE_PORT` remains strict. |
| Visual validator | `node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs` |
| Interaction validator | `node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs` |
| Responsive validator | `node docs/ui-and-layout/hub-web-reference/validate-responsive.mjs` |
| AI manifest schema | `docs/ui-and-layout/hub-ai-reference-manifest.schema.json` |
| Design-board validator | `node docs/ui-and-layout/hub-design-board/validate-design-board.mjs` |
| Focused Rust contract | `cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-shared\hub-scope-model --test ui_visual_standard_contract --message-format short --color never` |

## Generated Final PNG Inventory

These `docs/ui-and-layout/hub-*.png` files are the final reference outputs:

- `hub-editor.png`
- `hub-builds.png`
- `hub-assets.png`
- `hub-plugins.png`
- `hub-cloud.png`
- `hub-team.png`
- `hub-learn.png`
- `hub-settings.png`
- `hub-projects-new.png`
- `hub-projects-browser.png`
- `hub-projects-detail.png`
- `hub-projects-browser-filter-menu.png`
- `hub-projects-browser-sort-menu.png`
- `hub-projects-detail-delete-confirm.png`
- `hub-source-engine-popup.png`
- `hub-user-menu.png`
- `hub-state-empty.png`
- `hub-state-loading.png`
- `hub-state-error.png`

## Automated Gate Results

| Gate | Result | Evidence |
| --- | --- | --- |
| Web reference export | Pass | `export-pages.mjs` regenerated the 19 final `hub-*.png` files and dashboard capture; default port fallback and strict explicit-port failure were both exercised. |
| Web reference visual validation | Pass | `validate-visuals.mjs` validated 19 exported PNGs at `1568x1003`, root inventory, retired generator absence, `EXPORTS.md`, docs matrix, README policy, AI manifest schema, schema subset validation, negative schema self-tests, 19 required AI drafts at `1024x1024`, exact AI draft directory inventory with no orphaned AI draft PNGs or stray draft files, spot checks, and dashboard pixel comparison. |
| Web reference interaction validation | Pass | `validate-interactions.mjs` validated 20 browser-openable pages, 19 output filename replay paths, 19 `EXPORTS.md` replay paths, 53 click routes, 4 local UI state interactions, 3 search/filter interactions, 6 applied state interactions, zero unhandled actionable buttons, and left no `zircon-hub-cdp-*` temp profile. |
| Web reference responsive validation | Pass | `validate-responsive.mjs` validated the dashboard plus all 19 exported pages across `1568x1003`, `1920x1080`, `1915x508`, `1600x1024`, `1280x900`, `1024x720`, `900x720`, `760x680`, and `640x640` using DOM geometry, then validated 7 representative pages through 5 live resize steps without reload, local-only runtime dependencies, viewport-full shell geometry, large-preview dashboard card expansion, required `.project-cover-image` reference cover elements on project pages, Projects Browser header/every-row column alignment, and no UI reuse of AI draft/exported Hub PNG files. |
| Design-board validation | Pass | `validate-design-board.mjs` validated 3 supplemental PNGs at `1568x1003`, manifest/export metadata hashes, structure review documents, review packet schema, fixed canvas fit, geometry boundaries, and key-label fit. |
| Focused Rust visual contract | Pass | `ui_visual_standard_contract` ran through Cargo with 8 passed, 0 failed, 0 ignored. |

## Current Session Rerun Notes

- `node docs/ui-and-layout/hub-web-reference/export-pages.mjs` was rerun after the CSS cover conversion and regenerated the dashboard capture plus all 19 final `hub-*.png` files.
- `node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs` passed after the regenerated PNGs and updated dashboard comparison metrics.
- `node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs` passed after the regenerated PNGs, including 53 click routes, 4 local UI state interactions, 3 search/filter interactions, and 6 applied state interactions for filter, sort, pagination, source-engine popup selection, new-project engine selection, and browser grid/list mode.
- `node docs/ui-and-layout/hub-web-reference/validate-responsive.mjs` passed after extending responsive coverage to the `1915x508` design-review crop, `760x680`, and `640x640`, adding a five-step live resize sequence for 7 representative pages, and checking viewport-full shell geometry.
- `hub-projects-browser.png` was regenerated after tightening the browser table CSS; `validate-responsive.mjs` now checks the All Projects table, header, every row, and selected-row highlight share the same width, left edge, right edge, and column widths.
- Project cards, recent project rows, the project browser, selected-project side
  preview, and project detail hero now use `.project-cover-image` local
  reference cover images from `zircon_hub/assets/covers/reference/*`.
- The `zircon_hub/assets/covers/reference/*` local cover images are accepted UI
  inputs for matching the fixed `hub.png` project-card style.
- A fresh rerun of `cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --test ui_visual_standard_contract` was attempted twice in this session and timed out after 5 minutes, then after 10 minutes, without a pass/fail result. The timed-out Hub `cargo`/`rustc` processes were stopped so no Hub visual-contract compile remained running.

## Known Limits

- AI drafts are direction records only. They are not acceptance evidence; final
  HTML/CSS exported PNGs own text, icons, spacing, overlays, and page geometry.
- Responsive browser preview is accepted through DOM geometry validation and
  live resize checks, not through additional responsive screenshots.
- AI draft inventory is still enforced: `docs/ui-and-layout/hub-ai-drafts/`
  must contain exactly the 19 manifest-listed PNGs and no stray draft files.
- The three design-board PNGs are structure/function review support. They are
  not part of the 19 final `hub-*.png` acceptance inventory.
- Manual structure approval remains separate in
  `docs/ui-and-layout/hub-design-board/STRUCTURE_ACCEPTANCE_RECORD.md`.
- optional `cargo check` timeout: a scoped `cargo check --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1` was attempted as an optional extra check, but it timed out under concurrent Hub library compilation. That timeout is not acceptance evidence for or against this visual-reference slice; the focused Cargo visual contract above is the Rust gate for these files.

## Acceptance Boundary

Accept only the final web-reference PNGs and their static validation package.
Runtime behavior, persistence, project management, and live Hub screenshot
approval remain covered by the separate runtime capture evidence listed in the
Hub UI documentation.
