# Structure Acceptance Record

This record summarizes the current review state for the Hub design-board
package. The decision order remains overall interaction structure layout first,
then functional content as supporting evidence.

## Review Result

Manual sign-off pending.

The automated evidence is ready for manual inspection, but this file does not
claim final design approval. Final approval still requires checking the primary
structure screenshot and the manual review items below.

## Automated Evidence

| Evidence | Required state |
| --- | --- |
| `docs/ui-and-layout/hub.png` | Remains the fixed target reference at `1568x1003`. |
| `validate-design-board.mjs` | Passes after the latest export. |
| `validate-visuals.mjs` | Passes for the final Hub web-reference PNG set. |
| `validate-interactions.mjs` | Passes for page replay paths and representative click routes. |
| `manifest.json` | Lists the same target reference, reference-alignment checks, structure-first review sequence, manual review items, manual sign-off state, acceptance boundary, validation commands, and review-support entry points as the packet, plus the top-level support-document list. |
| `manifest.schema.json` | Defines the top-level manifest shape and fixed structure-first constraints. |
| `export-metadata.json` | Matches current source-input hashes and exported PNG hashes. |
| `STRUCTURE_REVIEW_STATUS.md` | Lists the current structure review priority, evidence, gates, and acceptance state. |
| `structure-review-packet.json` | Lists the review artifacts, manual review items, support documents, validation commands, and acceptance boundary. |
| `structure-review-packet.schema.json` | Defines the review packet fields and fixed structure-first constraints. |
| `STRUCTURE_COVERAGE_MATRIX.md` | Keeps every manual item mapped to the correct primary and secondary artifact. |
| `STRUCTURE_GEOMETRY_EVIDENCE.md` | Keeps measured shell geometry tied to the primary structure screenshot. |
| `structure-geometry-baseline.json` | Keeps the measured shell geometry available as machine-readable baseline data. |
| `structure-responsive-baseline.json` | Keeps responsive ownership rules available as machine-readable baseline data. |
| `structure-review-route-baseline.json` | Keeps the structure-first review route and local-detail blocking rules available as machine-readable baseline data. |
| `structure-overlay-baseline.json` | Keeps overlay ownership and no-reflow rules available as machine-readable baseline data. |
| `structure-reference-route-baseline.json` | Keeps structure-review concerns mapped to known final web-reference PNGs. |
| `REFERENCE_ALIGNMENT_MATRIX.md` | Keeps each target `hub.png` region tied to its owning design-board artifact and acceptance rule. |
| `STRUCTURE_SIGNOFF_CHECKLIST.md` | Keeps manual approval pending for every target-region structure check. |
| `STRUCTURE_DECISION_LOG.md` | Keeps each target-region decision in automated-evidence-ready state until manual sign-off. |

## Manual Review Items

| Item | Primary evidence | Decision needed |
| --- | --- | --- |
| Shell ownership | `hub-design-structure-layout.png` | Topbar, Sidebar, Workspace, and Bottom strip are fixed structural regions. |
| Workspace replacement | `hub-design-structure-layout.png` | Pages replace Workspace content without resizing shell chrome. |
| Overlay behavior | `hub-design-structure-layout.png` | Header menus and page menus float above content without layout reflow. |
| Route grouping | `hub-design-structure-supplement.png` | Projects, global pages, overlays, and state pages remain separate review groups. |
| Reference alignment | `docs/ui-and-layout/hub.png` | Topbar, Sidebar, Workspace, overlays, and bottom state strip match the target structure before local details are accepted. |
| Functional support | `hub-design-functional-details.png` | Functional density supports the layout and does not override the structure decision. |

## Acceptance Boundary

Accept only the structure package represented by these artifacts. This record
does not accept future runtime behavior, persistence, status/error workflows, or
Hub implementation changes outside the design-board and web-reference evidence
set.
