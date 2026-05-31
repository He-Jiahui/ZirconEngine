# Current Structure Review Status

This file is the compact status entry for checking the Hub design-board package.
The review priority remains overall interaction structure layout first;
functional content is only supporting evidence.

## Review Priority

1. Inspect `docs/ui-and-layout/hub-design-structure-layout.png`.
2. Confirm browser-measured shell geometry in `STRUCTURE_GEOMETRY_EVIDENCE.md`.
3. Confirm the same measurements are locked in `structure-geometry-baseline.json`.
4. Confirm responsive structure ownership in `structure-responsive-baseline.json`.
5. Confirm `structure-review-route-baseline.json` keeps the structure-first
   review order and blocking rules.
6. Confirm `structure-overlay-baseline.json` keeps floating overlay ownership
   separate from shell geometry.
7. Confirm `structure-reference-route-baseline.json` maps each structure check
   to a known final web-reference PNG.
8. Use `docs/ui-and-layout/hub-design-structure-supplement.png` only for route,
   overlay, and responsive ownership questions.
9. Use `docs/ui-and-layout/hub-design-functional-details.png` last for local
   content density and control coverage.

## Primary Review Evidence

| Evidence | Role |
| --- | --- |
| `docs/ui-and-layout/hub.png` | Target visual reference for structure and palette comparison. |
| `hub-design-structure-layout.png` | Primary shell, navigation, Workspace, bottom strip, and overlay layout. |
| `manifest.json` | Lists artifact categories, review entry points, target reference, reference-alignment checks, support documents, validation commands, manual sign-off state, acceptance boundary, the structure-first review sequence, and manual review items. |
| `manifest.schema.json` | Defines the top-level manifest shape for external validation. |
| `STRUCTURE_COVERAGE_MATRIX.md` | Maps every structure item to the artifact that must be inspected first. |
| `STRUCTURE_GEOMETRY_EVIDENCE.md` | Records the measured Topbar, Sidebar, Workspace, Bottom strip, and overlay boundaries. |
| `structure-geometry-baseline.json` | Locks the same shell geometry as machine-readable baseline data. |
| `structure-responsive-baseline.json` | Locks Desktop, Compact Window, and Collapsed Sidebar ownership rules. |
| `structure-review-route-baseline.json` | Locks the structure-first review order and local-detail blocking rules. |
| `structure-overlay-baseline.json` | Locks Source, account, menu, and confirm overlays as floating no-reflow layers. |
| `structure-reference-route-baseline.json` | Locks structure-review concerns to final web-reference PNG routes. |
| `STRUCTURE_TO_REFERENCE_MAP.md` | Maps structure questions to the final Hub web-reference PNGs. |
| `REFERENCE_ALIGNMENT_MATRIX.md` | Maps target `hub.png` regions to the design-board artifact that owns each decision. |
| `STRUCTURE_SIGNOFF_CHECKLIST.md` | Keeps every structure approval item pending until user review. |
| `STRUCTURE_DECISION_LOG.md` | Records automated-evidence-ready decisions and the manual sign-off blocker for each target region. |
| `STRUCTURE_ACCEPTANCE_RECORD.md` | Separates automated evidence readiness from final manual sign-off. |
| `structure-review-packet.json` | Provides the machine-readable artifact, manual-review-item, support-document, command, and boundary package. |
| `structure-review-packet.schema.json` | Defines the packet shape for external validation. |
| `export-metadata.json` | Locks source-input hashes and exported PNG hashes to the latest export. |

## Automated Gate Coverage

- `validate-design-board.mjs` checks the fixed `1568x1003` canvas, PNG dynamic
  range, manifest paths, export metadata hashes, coverage-matrix row order,
  browser geometry against `structure-geometry-baseline.json`, responsive
  ownership against `structure-responsive-baseline.json`, review route order
  against `structure-review-route-baseline.json`, overlay ownership against
  `structure-overlay-baseline.json`, final-reference routes against
  `structure-reference-route-baseline.json`, scroll overflow, and key-label fit.
- `validate-visuals.mjs` checks the final Hub web-reference PNG set and keeps
  the generated reference pages aligned with the source baseline.
- `validate-interactions.mjs` checks the final Hub web-reference route and click
  targets.

## Manual Structure Checks

| Check | Pass condition |
| --- | --- |
| Shell frame | Topbar, Sidebar, Workspace, and Bottom strip keep fixed ownership. |
| Navigation | Sidebar routes replace Workspace content only. |
| Workspace | Dashboard, secondary pages, state pages, and local tools stay inside Workspace. |
| Overlay layer | Header menus, filters, sort menus, and confirms float without layout reflow. |
| Responsive structure | Compact and collapsed structures preserve shell ownership. |
| Reference alignment | Topbar, Sidebar, Workspace, overlays, and bottom state strip are checked against `docs/ui-and-layout/hub.png`. |
| Functional support | Local functional density supports the structure decision but does not override it. |

## Acceptance State

Accept the design-board package only after the latest export has a matching
`export-metadata.json`, `validate-design-board.mjs` passes, and the manual
checks above are reviewed in the listed order.
