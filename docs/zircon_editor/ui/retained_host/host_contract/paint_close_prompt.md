---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/layout.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract paint-close-prompt color/layout/button/draw ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Paint Close Prompt

`paint_close_prompt.rs` is the retained-host close-confirmation prompt paint boundary. It now stays as a structural entry that re-exports the dialog draw entry used by Workbench scene-layer painting.

`paint_close_prompt/colors.rs` owns the prompt palette: overlay scrim, dialog surfaces, disabled button surface, text, muted text, warning text, and focus-ring accent. `paint_close_prompt/layout.rs` owns the details text well geometry inside the dialog.

`paint_close_prompt/button.rs` owns button fill, border, label position, and disabled styling. `paint_close_prompt/draw.rs` owns the full prompt draw order: overlay scrim, dialog shell, title, message, details well, and Save/Discard/Cancel actions.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
