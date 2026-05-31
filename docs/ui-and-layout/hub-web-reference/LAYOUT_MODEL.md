# Hub Web Reference Layout Model

This document describes the HTML/CSS reference as a migration model for the
Rust Hub window implementation. The final reference PNGs are export artifacts;
the source of layout truth is the HTML structure plus `styles.css`,
`covers.css`, `responsive.css`, `fullscreen-preview.css`, and the small local
behavior scripts.

## Runtime Asset Policy

- `index.html` loads only local files: `styles.css`, `covers.css`,
  `responsive.css`, `fullscreen-preview.css`, `interaction-states.css`,
  `cover-rendering.js`, `app.js`, and `interaction-enhancements.js`.
- No CDN, framework bundle, component library, or third-party runtime is used.
- The page does not use `hub-ai-drafts/*`, `hub-*.png`, or
  `hub-web-reference-1568x1003.png` as UI source images.
- Project cover art uses real local reference images through `.project-cover`
  and `.project-cover-image` elements in `covers.css`, matching the fixed
  `hub.png` concept-art texture while staying fully local.

## Region Model

| Web region | Rust-window migration target | Notes |
| --- | --- | --- |
| `.hub-shell` | Root window grid | Defines topbar, sidebar, workspace, and optional reference button-state strip. |
| `.topbar` | Header chrome | Fixed-height row with brand, source selector, status badges, account, and window controls. |
| `.sidebar` / `.nav-list` | Navigation rail | Expands at reference width and collapses to icon-only below compact breakpoints. |
| `.workspace` | Scrollable page slot | Owns page-level scrolling so the app shell itself does not overflow. |
| `.page-heading` | Page title/action row | Becomes a stacked header under narrow widths. |
| `.panel` | Shared surface component | 8px radius, single border, shadow at full size, no nested-card decoration. |
| `.project-cover` | Local reference project preview art | Uses the same concept-art cover family as `hub.png` for cards, tables, browser rows, side panels, and detail heroes. |
| `.row-list` rows | Data row component | Fixed icon/main/badge/action rhythm maps to Slint horizontal boxes. |
| `.source-popover`, `.user-popover`, `.menu-panel`, `.confirm-panel` | Overlay layer | Positioned relative to shell; clamps inward for compact windows. |
| `.state-canvas` | Empty/loading/error page template | Uses same panel hierarchy as normal pages, with optional supporting side panel. |

## Breakpoint Model

The 1568x1003 export canvas remains unchanged for design PNG generation because
the exported browser viewport is also 1568x1003. Normal browser preview always
fills the current viewport; smaller windows then use the responsive overrides.

| Breakpoint | Intent | Layout behavior |
| --- | --- | --- |
| all viewports | Live window mode | Root shell is `100vw x 100vh`; workspace owns scrolling. |
| `max-width: 1400px` | Medium desktop | Header gaps, project cards, and page sidebars tighten. |
| `max-width: 1180px` or `max-height: 860px` | Compact desktop | Sidebar becomes icon rail, status badges and bottom reference strip hide, page layouts become one-column where needed. |
| `max-width: 980px` | Narrow desktop | Header actions wrap, dense tables reduce secondary columns, overlays clamp to viewport. |
| `max-width: 760px` | Small review window | Header and rail shrink, page typography tightens, multi-column stats collapse. |
| `640x640` validation floor | Minimum checked preview window | The shell still fits the viewport; page content scrolls inside `.workspace` without document-level horizontal overflow. |

## Component Translation Notes

- CSS variables in `styles.css` are the token source for Rust colors, radii,
  border widths, spacing, and status tones.
- Grid declarations describe ownership boundaries. When porting to Rust UI,
  keep `.hub-shell`, `.workspace`, and per-page layout grids as separate layout
  components instead of baking coordinates into each page.
- `responsive.css` is intentionally an override layer. It documents how each
  page surface collapses without changing the fixed export baseline.
- Use row/card components as reusable primitives: `info-row`, `action-row`,
  `catalog-row`, `browser-row`, `template-row`, and `timeline-item` map to
  small composable Rust/Slint components.
- Overflow policy is part of the contract: the document and shell must not
  horizontally overflow; page content scrolls inside `.workspace`.

## Validation

Run these after changing layout:

```powershell
node docs\ui-and-layout\hub-web-reference\validate-responsive.mjs
node docs\ui-and-layout\hub-web-reference\validate-interactions.mjs
node docs\ui-and-layout\hub-web-reference\validate-visuals.mjs
```

`validate-responsive.mjs` checks all browser-openable pages across `1568x1003`,
`1920x1080`, `1915x508`, `1600x1024`, `1280x900`, `1024x720`, `900x720`, `760x680`, and
`640x640` viewports using DOM geometry. It also resizes 7 representative pages
through a five-step live resize sequence without reloading, verifies local-only
runtime dependencies, requires real reference project cover images on project
pages, requires the root shell to fill the viewport, requires the dashboard card
row to expand in large fullscreen previews, keeps each Projects Browser row on
the same column grid as the header, and rejects UI reuse of AI draft or exported
Hub PNG files.
