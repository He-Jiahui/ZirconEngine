---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/inspector.rs
  - zircon_editor/src/ui/retained_host/app/inspector/drag_source.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/apply_arguments.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/click.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/field_ids.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/value_change.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/inspector.rs
  - zircon_editor/src/ui/retained_host/app/inspector/drag_source.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/apply_arguments.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/click.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/field_ids.rs
  - zircon_editor/src/ui/retained_host/app/inspector/surface_controls/value_change.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app inspector drag/control ownership scan
  - app inspector surface-control subowner ownership scan
  - git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Inspector App Boundary

## Purpose

The retained-host inspector app boundary owns native/template callbacks for inspector interaction. It keeps inspector-specific `RetainedEditorHost` methods visible inside the app module while separating object drag-source behavior from inspector surface control binding.

This split supports the 08 M3.S2 retained-host owner cleanup: `app/inspector.rs` is now a structural entry, and inspector behavior lands in named child modules that can grow independently.

## Related Files

- `zircon_editor/src/ui/retained_host/app/inspector.rs` declares the inspector child modules only.
- `zircon_editor/src/ui/retained_host/app/inspector/drag_source.rs` owns inspector header pointer behavior and selected-object drag payload construction.
- `zircon_editor/src/ui/retained_host/app/inspector/surface_controls.rs` is the structural entry for inspector surface control behavior.
- `zircon_editor/src/ui/retained_host/app/inspector/surface_controls/field_ids.rs` owns static and dynamic inspector control id to field id mapping.
- `zircon_editor/src/ui/retained_host/app/inspector/surface_controls/apply_arguments.rs` owns selected-entity batch apply argument construction.
- `zircon_editor/src/ui/retained_host/app/inspector/surface_controls/value_change.rs` owns component-adapter `ValueChanged` dispatch for inspector field edits.
- `zircon_editor/src/ui/retained_host/app/inspector/surface_controls/click.rs` owns Apply/Delete click routing.
- `zircon_editor/src/ui/retained_host/app/inspector/surface_controls/dispatch.rs` owns builtin inspector surface control dispatch.
- `zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs` forwards inspector pane surface edits/clicks into these app-visible methods.

## Behavior Model

Inspector reference pointer input enters through `inspector_reference_pointer_event(...)`. Primary press clears incompatible active drag payloads, focuses the callback source window, constructs an object drag payload from the selected inspector snapshot, and reports the drag source in the status line. Primary release clears the active object drag payload.

Inspector field edits enter through `dispatch_inspector_control_changed(...)`. The control id is mapped to a stable inspector field id, including dynamic plugin component fields. The host sends a component-adapter `ValueChanged` event targeted at the selected entity and invalidates presentation data when the adapter reports a changed projection.

Inspector clicks enter through `dispatch_inspector_control_clicked(...)`. `ApplyBatchButton` builds the selected-entity batch apply argument list from the current inspector snapshot, including editable plugin component properties. `DeleteSelected` dispatches with no arguments. Unknown controls write a status-line diagnostic.

## Design and Rationale

Object drag-source behavior and surface control dispatch change for different reasons. Drag-source behavior depends on pointer semantics and active drag payload state, while surface controls depend on inspector field ids, component adapter envelopes, and builtin inspector surface routes. Keeping these paths in separate child modules prevents future inspector additions from turning a small callback file into another mixed owner.

The child methods use `pub(in crate::ui::retained_host::app)` for app-local visibility. This preserves existing sibling callers without exposing inspector callbacks outside the retained-host app boundary.

## Edge Cases and Constraints

- Dynamic inspector fields must keep the `DynamicComponentField:` prefix contract so plugin component properties can reuse the same edit path.
- Empty parent values are sent as `UiBindingValue::Null`; all other field values are sent as strings to match the existing component adapter contract.
- Drag-source creation depends on the selected inspector snapshot. With no current inspector selection, no object drag payload is created.
- Unknown field or click controls are status-line diagnostics rather than silent no-ops.

## Test Coverage

Implementation-slice validation covers formatting, ownership scans, scoped diff checks, and the current practical Cargo check status. `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` is currently blocked before editor code by unrelated active-worktree `zircon_runtime` post-process render errors. Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 inspector surface-control subowner split reduced `inspector/surface_controls.rs` from 141 lines to a 5-line structural entry. `surface_controls/field_ids.rs` is 13 lines and owns control-id mapping; `apply_arguments.rs` is 49 lines and owns selected-entity batch apply arguments; `value_change.rs` is 44 lines and owns component-adapter field edit dispatch; `click.rs` is 24 lines and owns Apply/Delete click routing; `dispatch.rs` is 23 lines and owns builtin inspector surface control dispatch.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app inspector surface-control subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test/check processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- Re-run the scoped `zircon_editor` Cargo check after the active `zircon_runtime` post-process render compile errors are resolved.
- Keep future inspector drag/drop behavior in `drag_source.rs`.
- Keep `surface_controls.rs` structural.
- Keep control id mapping in `surface_controls/field_ids.rs`, batch apply argument construction in `surface_controls/apply_arguments.rs`, field edit component-adapter dispatch in `surface_controls/value_change.rs`, click routing in `surface_controls/click.rs`, and builtin inspector surface dispatch in `surface_controls/dispatch.rs`.
