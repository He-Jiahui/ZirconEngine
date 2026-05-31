# Zircon Hub Design Board

This folder contains review-oriented design boards for the Hub layout work.
They are separate from the final `hub-web-reference` screenshots so structural
annotations do not become part of the accepted UI reference images.

Start manual review from `REVIEW_INDEX.md`; it links the exported screenshots,
supporting review documents, validation commands, and acceptance rule.

## Boards

- `structure` exports `docs/ui-and-layout/hub-design-structure-layout.png`.
  This is the primary review artifact for the overall shell, navigation,
  workspace, bottom strip, overlays, and route ownership.
- `flow` exports `docs/ui-and-layout/hub-design-structure-supplement.png`.
  This reviews page route groups, overlay ownership, compact-window behavior,
  collapsed-sidebar behavior, and structural invariants as an overall layout
  supplement.
- `details` exports `docs/ui-and-layout/hub-design-functional-details.png`.
  This is a secondary artifact for local functional content checks.

## Workflow

1. Open `index.html?board=structure` for the overall layout board.
2. Open `index.html?board=flow` for the overall structure supplement.
3. Open `index.html?board=details` for local functional content.
4. Run `node docs/ui-and-layout/hub-design-board/export-design-board.mjs`.
5. Run `node docs/ui-and-layout/hub-design-board/validate-design-board.mjs`.
6. Keep design-board exports out of the 19 final Hub web-reference PNG list.

The export command starts one headless Microsoft Edge session and captures all
three boards through the debugging protocol. Use a generous timeout when
running it from automation, but a normal run should complete much faster than
the older per-board Playwright process flow.

Review priority: check the overall structure layout first, then use the local
flow board for route/responsive ownership, and use the functional board only to
confirm page-content coverage.

Use `STRUCTURE_REVIEW.md` as the checklist while reviewing the screenshots. It
keeps the acceptance order focused on shell proportions, navigation ownership,
Workspace zones, overlays, shared states, and only then local functional
coverage.

Use `STRUCTURE_COVERAGE_MATRIX.md` to map each review item to the primary
artifact that should be inspected first and the secondary artifact that can
clarify the decision.

Use `STRUCTURE_GEOMETRY_EVIDENCE.md` when checking the primary structure board
against concrete measured shell boundaries.

Use `structure-geometry-baseline.json` as the machine-readable geometry
baseline that browser measurement must match before manual review.

Use `structure-responsive-baseline.json` as the machine-readable responsive
structure baseline for Desktop, Compact Window, and Collapsed Sidebar review.

Use `structure-review-route-baseline.json` as the machine-readable review route
baseline that keeps `hub-design-structure-layout.png` first,
`hub-design-structure-supplement.png` second, and
`hub-design-functional-details.png` last.

Use `structure-overlay-baseline.json` as the machine-readable overlay ownership
baseline that keeps Source, account, menu, and confirm layers floating without
changing shell geometry.

Use `structure-reference-route-baseline.json` as the machine-readable version
of `STRUCTURE_TO_REFERENCE_MAP.md`; it links each structure review item to the
final web-reference PNG that should be used for visual-detail confirmation.

Use `STRUCTURE_REVIEW_GUIDE.md` as the human-facing pass/fail guide for manual
structure review.

Use `STRUCTURE_TO_REFERENCE_MAP.md` to trace a structure-review concern from the
design-board artifact to the final Hub web-reference PNG that demonstrates the
same area with full visual detail.

Use `REFERENCE_ALIGNMENT_MATRIX.md` to compare each target `hub.png` region
against the design-board artifact that owns the structure decision.

Use `STRUCTURE_SIGNOFF_CHECKLIST.md` as the pending manual checklist for the
structure decisions that still need user review.

Use `STRUCTURE_DECISION_LOG.md` to see which target-region decisions have
automated evidence ready and which ones remain blocked on manual sign-off.

Use `STRUCTURE_REVIEW_STATUS.md` as the compact current-status entry before a
manual review pass. It lists the primary evidence, automated gates, manual
checks, and acceptance state in review order.

Use `STRUCTURE_ACCEPTANCE_RECORD.md` to separate automated evidence from final
manual sign-off. It records that automatic gates can be ready while user review
of the primary structure screenshot remains pending.

Use `structure-review-packet.json` when a tool or reviewer needs the complete
machine-readable review package: artifact order, enforced review sequence,
manual review items, the `docs/ui-and-layout/hub.png` target reference,
reference-alignment checks, support documents including the manifest,
validation commands, and acceptance boundary.

Use `structure-review-packet.schema.json` to validate the review package shape
without reading the JavaScript validator.

`manifest.json` records the review priority, manifest schema, manual sign-off
state, `docs/ui-and-layout/hub.png` target reference, reference-alignment
checks, review sequence, manual review items, support-document list, validation
commands, acceptance boundary, review index, checklist, coverage-matrix,
geometry-evidence, geometry baseline, responsive baseline, review route baseline,
overlay baseline, reference route baseline, review-guide, reference-map, structure-review status,
reference-alignment matrix, structure sign-off checklist, structure decision log,
structure-acceptance record, structure-review packet,
structure-review packet schema, export metadata entry points, and artifact categories: two primary structure artifacts
(`structure`, `flow`) and one secondary functional detail artifact (`details`).

Use `manifest.schema.json` to validate the top-level design-board manifest
without reading the JavaScript validator.

`export-metadata.json` is written by the export command. It records SHA-256
hashes for the design-board source inputs (`index.html`, `styles.css`, and
`board-registry.mjs`) plus the three exported PNG files. The validator compares
those hashes and PNG byte sizes with the current files so a source edit without
a fresh screenshot export is rejected.

`board-registry.mjs` is the shared source of truth for design-board ids,
filenames, artifact categories, required review text, the manifest, the
structure-review checklist, and the coverage matrix. The validator checks every
exported PNG for the fixed `1568x1003` canvas and basic image dynamic range so
blank or low-information captures are rejected, verifies `manifest.json`,
`manifest.schema.json`, the `docs/ui-and-layout/hub.png` target reference,
reference-alignment checks, manifest support-document paths, `EXPORTS.md`, `export-metadata.json`,
`STRUCTURE_REVIEW.md`, and
`STRUCTURE_COVERAGE_MATRIX.md`, `REFERENCE_ALIGNMENT_MATRIX.md`,
`STRUCTURE_SIGNOFF_CHECKLIST.md`, `STRUCTURE_DECISION_LOG.md`,
`structure-geometry-baseline.json`,
`structure-responsive-baseline.json`,
`structure-review-route-baseline.json`,
`structure-overlay-baseline.json`,
`structure-reference-route-baseline.json`,
`STRUCTURE_REVIEW_STATUS.md`, and
`STRUCTURE_ACCEPTANCE_RECORD.md` against the registry, verifies the manifest
and packet use the same structure-first review sequence and manual review item
order, verifies
`structure-review-packet.json` plus `structure-review-packet.schema.json` as
the machine-readable review package, enforces its structure-first review
sequence and manual review item order, applies the schema subset used by the
packet (`type`, `required`, `additionalProperties`, `const`, `enum`, array item
counts, and string patterns), runs positive and negative self-tests for that
schema subset, enforces the
coverage-matrix row order and primary/secondary artifact categories, and opens
each board in Microsoft Edge to make sure the fixed canvas does not scroll
horizontally or vertically. The structure board also has browser geometry
checks for Topbar, Sidebar, Workspace, Bottom strip, and overlay boundaries,
plus key-label fit checks for the dimension strip, tabs, shell labels, route
labels, and priority labels so the primary layout cannot drift while the labels
remain.
