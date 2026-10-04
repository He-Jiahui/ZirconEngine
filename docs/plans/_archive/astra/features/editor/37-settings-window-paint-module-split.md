---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window/scrollbars.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window/visible_rows.rs
tests:
  - tools/tests/test_editor_settings_window_mutation_wiring_contract.py
---

# Settings Window Paint Module Split

The 950-line Settings paint command owner mixed panel orchestration, scrollbar
drawing, and visible-row clipping. Scrollbar rendering and visibility/frame
intersection are now focused sibling modules; `commands.rs` retains Settings
panel orchestration and control projection only.

The split keeps the existing one-row overscan and clip-bounded row behavior,
while making the production orchestration owner 747 lines. It is a structural
maintainability and hot-path ownership improvement, not a product latency
claim.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E37 | Split Settings window scrollbar and visible-row responsibilities from command orchestration | implemented_pending_validation | The original Settings static contract passed 12/12 after the split; all touched Rust files passed scoped rustfmt and scoped diff checks passed. Managed Rust compilation remains blocked at immutable admission by dirty external worktree `E:\\Git\\zr_vm`; Windows Release allocation/time p50/p95/p99 evidence remains pending. |
