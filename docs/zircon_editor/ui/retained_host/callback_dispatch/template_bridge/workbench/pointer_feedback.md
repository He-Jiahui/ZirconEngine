---
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/pointer_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/focus.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/pointer_feedback.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - cargo test -p zircon_editor --lib componentized_workbench_pointer_focuses_input_fields_without_authored_binding --locked -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib componentized_workbench_ --locked -- --nocapture --test-threads=1
doc_type: module-detail
---

# Workbench Pointer Feedback

## Purpose

The componentized Workbench pointer bridge converts shared runtime pointer routes into native host feedback effects. It keeps hover, press, range, text-input, and focus visual state on the runtime `UiSurface` path so the native retained host does not need a parallel per-control feedback model.

This module exists while the Workbench shell is being cut over region by region from retained fallback painting to runtime-authored `.zui` components. Pointer feedback must therefore refresh the Workbench surface projection before the host paints, but it must not require every focusable control to have an authored click binding.

## Related Files

`callback_dispatch/workbench/pointer.rs` owns the host event entry point. It captures the pressed and focused targets before routing the pointer event, asks the bridge to update each feedback family, and returns a paint-only effect when feedback changed without producing an activation command.

`callback_dispatch/template_bridge/workbench/pointer_feedback.rs` owns the feedback mutation rules. It inspects `UiPointerRoute` plus template metadata and calls `refresh_after_state_change(...)` only when a touched feedback candidate actually dirtied the runtime surface.

The lower shared focus behavior remains in `zircon_runtime/src/ui/surface/surface.rs` and `zircon_runtime/src/ui/surface/focus.rs`: primary pointer press resolves the first focusable node in the bubble route, updates component focus state, and marks the changed state dirty.

## Behavior Model

Hover feedback handles pointer enter/leave routes and mutates `hovered` only for controls whose metadata accepts hover feedback.

Press feedback handles primary press/release routes and mutates `pressed` for non-text controls that accept press feedback. Text inputs are excluded from generic press feedback so a field press does not paint as a button press.

Range feedback handles primary press/move/release on range controls. It captures the pointer on press, derives a clamped value from the pointer position inside the arranged frame, mutates the control's value property, and refreshes the surface projection.

Text-input feedback handles primary presses that focus a text-like control. It exists because text fields need an immediate focused visual response even when they have no authored activation binding.

Focus feedback compares the focused target before and after runtime route processing. A primary pointer press that changes focus is treated as a paint-only feedback signal whenever the runtime surface has dirty focus state. This is deliberately independent of text-input class metadata, so any focusable component can repaint focus state through the same shared route.

## Design And Rationale

The bridge refreshes after state changes instead of directly editing host DTO rows. Runtime remains the owner of focus and component state, while the retained host consumes a fresh projected Workbench window node set.

Focus feedback is separate from text-input feedback because focus is a lower shared route fact, not a property of a specific component family. The text-input path still documents and preserves the intended field behavior, but pointer focus changes do not depend on `InputField`, `TextField`, or Workbench field class detection to request paint-only feedback.

Activation stays after feedback refresh. If a pointer route also maps to a Workbench action, the command path can still return full effects; if it has no action, feedback dirtiness alone produces a paint-only request.

## Edge Cases And Constraints

Only primary pointer presses can trigger focus feedback. Hover, move, and release routes may carry the current focused node, but they do not establish a new focus target.

The feedback refresh is guarded by `dirty_flags().any()`. A route that compares as a focus change but does not dirty the runtime surface does not request redundant repaint work.

The bridge must not add a synthetic activation route for input fields. A text input without authored binding should still focus and repaint, but it should not dispatch a Workbench command.

## Test Coverage

`componentized_workbench_pointer_focuses_input_fields_without_authored_binding` is the focused regression for pointer focus feedback. It presses Workbench input fields without authored activation bindings and expects paint-only feedback plus focused visual projection.

The broader `componentized_workbench_` filter is the regression group for Workbench component projection and input behavior during the 08 shell cutover. It may expose unrelated projection or shell hard-cutover drift and should be rerun at milestone testing-stage boundaries.

## Plan Sources

This implementation belongs to the 08 Workbench shell runtime-UI cutover plan, specifically the M2 convergence work that removes retained fallback assumptions before deleting the remaining Workbench painter and drawer bridge paths.
