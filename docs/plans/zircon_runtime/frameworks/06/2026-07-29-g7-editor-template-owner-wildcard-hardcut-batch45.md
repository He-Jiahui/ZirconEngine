---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry
  - zircon_editor/src/ui/retained_host/host_contract/paint_text
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts
  - zircon_editor/src/ui/retained_host/host_contract/profiling_hit_routes
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alert_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_field_style
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_button_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chip_glyphs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_command_conversion
implementation_files:
  - docs/plans/performance/01/2026-07-17-editor-paint-text-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-paint-theme-overlays-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-profiling-artifacts-hit-routes-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-retained-pane-data-conversion-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-sprite-atlas-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-alerts-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-axis-controls-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-buttons-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-chips-static-review.md
  - docs/plans/performance/01/2026-07-17-editor-template-command-pipeline-static-review.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- exact11 Batch45 paths
---

# Frameworks06 G7 Editor Template Owner Wildcard Hard Cut Batch 45

Status: focused_g7_passed_global_g7_red_review_passed
Date: 2026-07-29
Session: `frameworks06-g7-editor-template-owner-wildcard-hardcut-batch45-r2-20260729`

## Completed Items

- Replaced 31 glob-shaped machine owners across ten Performance01 editor
  static-review records with the existing directory that owns each reviewed
  Rust file set.
- Preserved concrete root-file owners, review counts, performance findings,
  pending dynamic gates, and every existing plan status.
- Added no wildcard compatibility interpretation, alias, shim, generated owner,
  or duplicate architecture record.

## Validation State

- Fresh G7 reports zero violations for all exact11 documents. The 31 retired
  glob owners have zero hits, and all 31 current directory owners exist.
- Exact-scope `git diff --check` passes. The shared current-source G7 baseline
  remains red at 629 violations across 142 documents and 68,838 checked paths.
- Independent exact11 review found Critical/Important/Moderate/Minor
  `0/0/0/0` with zero input drift. This batch does not claim Frameworks06 M1,
  M2, the global G7 gate, or plan completion.
