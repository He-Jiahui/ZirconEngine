# Hub Design Board Review Index

This index is the entry point for reviewing the Zircon Hub design-board package.
The review priority is overall interaction structure layout first, functional
content second.

## Screenshot Artifacts

| Priority | Artifact | Purpose |
| ---: | --- | --- |
| 1 | `docs/ui-and-layout/hub-design-structure-layout.png` | Primary overall shell, navigation, workspace, bottom strip, and overlay layout. |
| 2 | `docs/ui-and-layout/hub-design-structure-supplement.png` | Route grouping, responsive ownership, collapsed-sidebar behavior, and structure invariants. |
| 3 | `docs/ui-and-layout/hub-design-functional-details.png` | Local functional-content coverage only. |

## Review Documents

- `STRUCTURE_REVIEW_GUIDE.md`: human pass/fail guide.
- `STRUCTURE_REVIEW.md`: checklist and acceptance order.
- `STRUCTURE_COVERAGE_MATRIX.md`: review item to primary/secondary artifact map.
- `STRUCTURE_GEOMETRY_EVIDENCE.md`: browser-measured shell geometry evidence.
- `structure-geometry-baseline.json`: machine-readable geometry baseline for browser measurement.
- `structure-responsive-baseline.json`: machine-readable responsive structure baseline.
- `structure-review-route-baseline.json`: machine-readable structure-first review route and blocking rules.
- `structure-overlay-baseline.json`: machine-readable overlay ownership and no-reflow rules.
- `structure-reference-route-baseline.json`: machine-readable structure item to final reference PNG routes.
- `STRUCTURE_TO_REFERENCE_MAP.md`: structure item to final web-reference PNG map.
- `REFERENCE_ALIGNMENT_MATRIX.md`: target `hub.png` region to design-board check map.
- `STRUCTURE_SIGNOFF_CHECKLIST.md`: pending manual approval checklist for the target structure.
- `STRUCTURE_DECISION_LOG.md`: automated-evidence-ready decision log with manual sign-off blockers.
- `STRUCTURE_REVIEW_STATUS.md`: compact current-status entry for manual review.
- `STRUCTURE_ACCEPTANCE_RECORD.md`: automated evidence and manual sign-off boundary.
- `manifest.schema.json`: schema for the top-level design-board manifest.
- `structure-review-packet.json`: machine-readable artifact/support/command package.
- `structure-review-packet.schema.json`: schema for the machine-readable review package.
- `export-metadata.json`: source and screenshot hash ledger for export freshness.
- `manifest.json`: artifact categories and review entry points.
- `EXPORTS.md`: exported screenshot list and replay/support order.

## Validation Commands

Run these from the repository root:

```powershell
node docs/ui-and-layout/hub-design-board/export-design-board.mjs
node docs/ui-and-layout/hub-design-board/validate-design-board.mjs
node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs
node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs
```

Use a generous timeout for the export command because it starts a headless Edge
capture session, but the current exporter reuses one browser session for all
three boards.

## Acceptance Rule

Approve the design-board package only when:

1. `hub-design-structure-layout.png` matches the shell ownership model.
2. `STRUCTURE_GEOMETRY_EVIDENCE.md` matches the visible structure.
3. `structure-geometry-baseline.json` matches the measured shell geometry.
4. `structure-responsive-baseline.json` keeps Desktop, Compact Window, and
   Collapsed Sidebar ownership rules in review order.
5. `structure-review-route-baseline.json` keeps the structure board first,
   structure supplement second, and functional detail board last.
6. `structure-overlay-baseline.json` keeps Source, account, menu, and confirm
   layers floating without resizing shell regions.
7. `structure-reference-route-baseline.json` maps every structure check to a
   known final web-reference PNG.
8. `STRUCTURE_COVERAGE_MATRIX.md` maps every structure item to the right
   primary artifact.
9. Functional details do not override the structure decision.
10. `export-metadata.json` hashes match the current source inputs and PNGs.
11. `structure-review-packet.json` lists the same artifact order, manual review
   items, support documents, validation commands, and acceptance boundary.
12. `manifest.schema.json` and `structure-review-packet.schema.json` match the
   manifest, packet fields, and fixed structure-first review constraints.
13. `REFERENCE_ALIGNMENT_MATRIX.md` maps every `hub.png` target region to the
   expected primary design-board artifact and acceptance rule.
14. `STRUCTURE_SIGNOFF_CHECKLIST.md` keeps every structure approval item pending
   until the primary structure screenshot has been reviewed.
15. `STRUCTURE_DECISION_LOG.md` keeps every target-region decision blocked on
   manual sign-off even when automated evidence is ready.
16. `STRUCTURE_ACCEPTANCE_RECORD.md` still marks manual sign-off pending until
   the primary structure screenshot has been reviewed.
17. `validate-design-board.mjs` passes after the latest export.
