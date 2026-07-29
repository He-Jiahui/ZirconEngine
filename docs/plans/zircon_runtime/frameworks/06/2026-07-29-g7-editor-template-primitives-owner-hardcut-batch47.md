---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes
implementation_files:
  - docs/plans/performance/01/2026-07-17-editor-template-fields-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-icon-buttons-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-inspector-rows-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-list-rows-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-node-label-text-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-popup-rows-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-property-rows-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-section-titles-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-segmented-controls-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-selection-controls-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - exact11 retired wildcard and existing-directory guard
  - git diff --check -- exact11 Batch47 paths
---

# Frameworks06 G7 Editor Template Primitives Owner Hard Cut Batch 47

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-editor-template-primitives-owner-hardcut-batch47-r1-20260729`

## Completed Items

- Replaced 30 glob-shaped Rust owners across ten retained-host template review
  records with the corresponding 30 existing directory authorities.
- Preserved each review record's findings, status, concrete ownership roots,
  and prose; this metadata hard cut does not claim a new source review.
- Added no wildcard compatibility interpretation, old-path alias, shim,
  generated owner, or duplicate architecture record.

## Validation State

- Fresh G7 reports zero violations for all exact11 documents. Retired wildcard
  owners have zero hits, all 30 replacement directories exist, and exact-scope
  `git diff --check` passes.
- The shared current-source G7 baseline remains red at 597 violations across
  131 documents and 68,853 checked paths. Snapshot1303 independent review
  passed with Critical/Important/Moderate/Minor = 0/0/0/0; this batch does not
  claim Frameworks06 M1, M2, the global G7 gate, or plan completion.
