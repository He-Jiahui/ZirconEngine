# Hub Design Board Structure Review Guide

Use this guide for manual review after the design-board PNGs have been exported
and `validate-design-board.mjs` has passed.

## Review Inputs

- Primary structure: `docs/ui-and-layout/hub-design-structure-layout.png`
- Structure supplement: `docs/ui-and-layout/hub-design-structure-supplement.png`
- Functional detail support: `docs/ui-and-layout/hub-design-functional-details.png`
- Coverage matrix: `STRUCTURE_COVERAGE_MATRIX.md`
- Geometry evidence: `STRUCTURE_GEOMETRY_EVIDENCE.md`
- Geometry baseline: `structure-geometry-baseline.json`
- Responsive baseline: `structure-responsive-baseline.json`
- Review route baseline: `structure-review-route-baseline.json`
- Overlay baseline: `structure-overlay-baseline.json`
- Reference route baseline: `structure-reference-route-baseline.json`

## Manual Checklist

| Check | Use first | Pass condition |
| --- | --- | --- |
| Shell proportions | `hub-design-structure-layout.png` | Topbar, Sidebar, Workspace, and Bottom strip remain visibly owned by the shell. |
| Navigation ownership | `hub-design-structure-layout.png` | Sidebar changes pages without changing shell geometry. |
| Workspace replacement | `hub-design-structure-layout.png` | Dashboard, secondary pages, and state pages fit inside Workspace. |
| Overlay ownership | `hub-design-structure-layout.png` | Header, menu, filter, sort, and confirm overlays float above content. |
| Overlay baseline | `structure-overlay-baseline.json` | Source, account, menu, and confirm overlays keep floating ownership without resizing shell regions. |
| Reference route baseline | `structure-reference-route-baseline.json` | Each structure check maps to a known final web-reference PNG. |
| Route grouping | `hub-design-structure-supplement.png` | Projects, global pages, and overlays remain separate route groups. |
| Responsive ownership | `hub-design-structure-supplement.png` | Compact and collapsed structures preserve shell ownership. |
| Functional density | `hub-design-functional-details.png` | Local content is dense enough without changing the structure decision. |

## Rejection Signals

- A page needs to resize Topbar, Sidebar, or Bottom strip to fit content.
- A dropdown, menu, modal, or confirm layer pushes cards, tables, or shell
  chrome instead of floating above them.
- Functional detail concerns override the primary structure board.
- The geometry evidence or `structure-geometry-baseline.json` no longer matches
  the visible structure screenshot.
- `structure-responsive-baseline.json` no longer matches the flow board's
  Desktop, Compact Window, and Collapsed Sidebar ownership model.
- `structure-review-route-baseline.json` no longer keeps the primary structure
  board before the structure supplement and functional detail board.
- `structure-overlay-baseline.json` no longer keeps header, menu, and confirm
  layers as floating no-reflow overlays.
- `structure-reference-route-baseline.json` references a final PNG that is not
  part of the web-reference export registry.
- Navigation, state pages, and overlays cannot be explained by the coverage
  matrix.

## Review Order

1. Open the primary structure screenshot and compare it to
   `STRUCTURE_GEOMETRY_EVIDENCE.md`.
2. Confirm `structure-geometry-baseline.json` keeps the same measured shell
   boundaries.
3. Confirm `structure-responsive-baseline.json` keeps the responsive ownership
   model in review order.
4. Confirm `structure-review-route-baseline.json` keeps the structure-first
   route and blocking rules.
5. Confirm `structure-overlay-baseline.json` keeps overlay ownership separate
   from shell geometry.
6. Confirm `structure-reference-route-baseline.json` maps the concern to a
   known final web-reference PNG.
7. Use `STRUCTURE_COVERAGE_MATRIX.md` to inspect each row's primary artifact.
8. Open the structure supplement only when route grouping or responsive
   ownership needs clarification.
9. Open the functional detail board last, and only to check local content
   coverage.
