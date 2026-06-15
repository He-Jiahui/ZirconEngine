---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_dialogs.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs
  - zircon_runtime/src/ui/surface/render/dialog.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_dialog.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_dialogs.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/surface_defaults.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/index.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_material_painter_dialog.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
  - rustfmt --edition 2021 touched Dialog native painter/projection Rust files
  - git diff --check -- touched Dialog native painter/projection/doc files
doc_type: module-detail
---

# Dialog Overlays

`template_dialogs.rs` owns retained native painting for `Dialog`, `ConfirmDialog`, and `AlertDialog`-role overlay shells. It is dispatched from `template_nodes.rs` before tooltip/material fallback painting so a recognized dialog root can consume the node, suppress generic surface/text fallback, and keep the native retained path aligned with runtime render extraction.

## Painter Contract

- Closed dialog roots are consumed without emitting commands. This matches runtime render extraction, where Dialog/ConfirmDialog owner surface/text/image fallbacks are suppressed and no dialog commands are emitted unless `open`/`popup_open` is true.
- Open `Dialog` roots draw a panel, active border, title, optional body text, and a right-aligned action label. Colors and geometry mirror the runtime dialog render constants: `#171c20` panel, `#35c7d0` active border/action, 20 px horizontal padding, 18 px title offset, 48 px body offset, and 6 px radius.
- Open `ConfirmDialog`/`AlertDialog` roots draw the same panel plus a left severity mark. Severity comes from retained `component_variant`, `validation_level`, `text_tone`, or related state tokens, with error/destructive state coloring the border, mark, and title.
- Disabled or loading roots use disabled panel/text tones. Disabled confirm actions are represented by `confirmDisabled`, `confirm-disabled`, `confirm_disabled`, or `disabledConfirm` owner-state tokens and draw the confirm label with disabled text color while leaving cancel available.
- Action labels are read from `TemplatePaneNodeData.actions`: index 0 is cancel/primary dialog action, index 1 is confirm for confirm dialogs. This avoids adding one-off fields to the shared host DTO.

## Projection Contract

`pane_component_projection` prepares the fields the painter consumes:

- Dialog title uses `title`, then `text`, then `label`, projected into `TemplatePaneNodeData.text`.
- Dialog body uses `message`, `description`, or `body`, projected into `TemplatePaneNodeData.value_text`.
- Dialog `action`/`primary_action_text`/`confirm_text`/`close_text` becomes a single action row.
- ConfirmDialog always projects two action rows: cancel then confirm, defaulting to `Cancel` and `Confirm` when labels are omitted.
- `surface_defaults` appends retained owner-state tokens for Dialog/ConfirmDialog action presence, severity, `color*`, `destructive`, and `confirmDisabled`; it also aligns `confirm-dialog` with dialog popup surface, border, elevation, and modal z-index defaults.

## Validation

`native_material_painter_dialog.rs` covers the retained pixel contract:

- `native_template_painter_draws_open_dialog_panel_text_and_action` verifies an open Dialog paints a panel, active border, title/body glyphs, action glyphs, and no outside overdraw.
- `native_template_painter_draws_confirm_dialog_error_and_disabled_confirm_action` verifies ConfirmDialog error severity mark/border/title plus disabled confirm action rendering.
- `native_template_painter_consumes_closed_dialog_without_surface_fallback` verifies a closed Dialog produces no native pixels.

Projection coverage lives in `runtime_component_projection_applies_mui_overlay_surface_defaults` and `component_showcase_pane_projects_runtime_component_nodes_for_template_pane`. Focused Cargo execution is still pending for this slice because shared Windows cargo/rustc lanes were active during implementation; the current accepted evidence is formatting, scoped diff check, and conflict-marker scan.
