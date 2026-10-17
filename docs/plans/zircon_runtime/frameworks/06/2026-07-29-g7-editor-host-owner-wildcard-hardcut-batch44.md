---
related_code:
  - zircon_editor/src/ui/retained/contract/paint_primitives/image
  - zircon_editor/src/ui/retained/app/workspace_docking/drawer_resize
implementation_files:
  - docs/plans/_archive/performance/01/2026-07-17-editor-chrome-command-stream-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-host-contract-data-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-host-globals-redraw-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-host-presenter-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-material-state-layer-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-native-keyboard-popup-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-native-pointer-damage-redraw-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-native-pointer-drag-resize-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-native-pointer-move-scroll-static-review.md
  - docs/plans/_archive/performance/01/2026-07-17-editor-paint-frame-primitives-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/audits/check_conventions.py --only docs --json
  - git diff --check -- exact11 Batch44 paths
---

# Frameworks06 G7 Editor Host Owner Wildcard Hard Cut Batch 44

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-editor-host-owner-wildcard-hardcut-batch44-r1-20260729`

## Completed Items

- Replaced 27 glob-shaped machine owners across ten Performance01 editor host
  static-review records with the existing directory that owns each reviewed
  Rust file set.
- Preserved concrete root-file owners, review counts, performance findings,
  pending dynamic gates, and every existing plan status.
- Added no wildcard compatibility interpretation, alias, shim, generated owner,
  or duplicate architecture record.

## Validation State

- Fresh G7 reports zero violations for all exact11 documents. The 27 retired
  glob owners have zero hits, and every current directory owner exists.
- Exact-scope `git diff --check` passes. The shared current-source G7 baseline
  remains red at 660 violations across 152 documents and 68,797 checked paths.
- Independent exact11 review found Critical/Important/Moderate/Minor
  `0/0/0/0` with zero input drift. This batch does not claim Frameworks06 M1,
  M2, the global G7 gate, or plan completion.
