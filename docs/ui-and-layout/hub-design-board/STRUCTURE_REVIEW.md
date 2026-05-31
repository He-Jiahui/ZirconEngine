# Hub Design Board Structure Review

Use this checklist when reviewing the Hub design-board screenshots. The primary
review target is the overall interaction structure layout, not local component
styling.

## Primary Artifacts

- `docs/ui-and-layout/hub-design-structure-layout.png`
- `docs/ui-and-layout/hub-design-structure-supplement.png`
- `docs/ui-and-layout/hub-design-functional-details.png`

## Overall Structure Checklist

- Shell frame: Topbar, Sidebar, Workspace, and Bottom strip keep stable
  ownership and do not trade responsibilities between pages.
- Navigation: Sidebar page ids change the Workspace content only.
- Workspace: page title, page actions, search/filter/sort/view controls,
  project cards, lower panels, and secondary-page content stay inside the
  Workspace region.
- Overlay layer: Source Engine popup, User Menu, Filter menu, Sort menu, and
  Delete Confirm float above the shell without consuming layout space.
- State layer: Empty, Loading, and Error states reuse the Workspace state
  canvas and do not alter global shell geometry.
- Responsive structure: compact windows compress Workspace content first;
  collapsed sidebar keeps navigation and status ownership visible.
- Bottom strip: button-state samples remain a review/testing strip and do not
  become page content.

## Functional Detail Checklist

- Projects Dashboard: toolbar, project-card flow, Recent Projects table, and
  Quick Actions are checked as one page-content system.
- Projects secondary pages: New Project, Browser, Detail, and Delete Confirm
  reuse the shell and only replace Workspace internals.
- Non-Projects pages: Editor, Builds, Assets, Plugins, Cloud, Team, Learn, and
  Settings keep the same panel/list/card density model.
- Local content diagrams are secondary evidence; they should not override the
  overall structure boards.

## Acceptance Order

1. Verify shell proportions and ownership.
2. Verify navigation and page replacement boundaries.
3. Verify Workspace content zones.
4. Verify overlay stacking and non-reflow behavior.
5. Verify shared state pages.
6. Verify responsive ownership and the structure-first review route.
7. Verify local functional coverage.

## Automated Geometry Guard

`validate-design-board.mjs` opens the primary structure board in Microsoft Edge
and checks the `.hub-frame` geometry for Topbar, Sidebar, Workspace, Bottom
strip, Source Engine overlay, and Account overlay boundaries. Manual review
should still judge the visible hierarchy and readability, but frame ownership
and overlay non-reflow are guarded by the browser measurement.

See `STRUCTURE_GEOMETRY_EVIDENCE.md` for the current measured geometry table.
See `structure-responsive-baseline.json` and
`structure-review-route-baseline.json` for the machine-readable responsive and
review-order baselines. See `structure-overlay-baseline.json` for the floating
overlay ownership baseline. See `structure-reference-route-baseline.json` for
the machine-readable route from each structure concern to the final
web-reference PNG.
