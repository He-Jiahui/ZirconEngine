---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing
  - zircon_editor/src/ui/retained_host
  - zircon_editor/src/ui/retained_host/ui/tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dialogs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center
implementation_files:
  - docs/plans/performance/01/2026-07-17-editor-native-pointer-menu-geometry-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-native-pointer-routing-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-retained-root-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-retained-ui-tests-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-command-palette-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-dialogs-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-drag-overlay-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-icon-glyph-shapes-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-notification-center-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- exact10 Batch43 paths
---

# Frameworks06 G7 Editor Performance Wildcard Owner Hard Cut Batch 43

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-editor-performance-wildcard-owner-batch43-r1-20260729`

## Completed Items

- Replaced nine glob-shaped machine owners in Performance01 editor static-review
  records with the existing directory that owns each reviewed Rust file set.
- Preserved every concrete root-file owner, review count, performance finding,
  pending dynamic gate, and plan status.
- Added no wildcard compatibility interpretation, alias, shim, generated owner,
  or duplicate architecture record.

## Validation State

- Fresh G7 reports zero violations for all exact10 documents. The nine retired
  glob owners have zero hits, and all nine current directory owners exist.
- Exact-scope `git diff --check` passes. The shared current-source G7 baseline
  remains red at 687 violations across 162 documents and 68,761 checked paths.
- Independent exact10 review found Critical/Important/Moderate/Minor
  `0/0/0/0` with zero input drift. This batch does not claim Frameworks06 M1,
  M2, the global G7 gate, or plan completion.
