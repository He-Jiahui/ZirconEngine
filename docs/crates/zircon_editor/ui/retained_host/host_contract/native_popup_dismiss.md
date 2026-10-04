---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/target.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract native-popup-dismiss dispatch/target ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Native Popup Dismiss

`native_popup_dismiss.rs` owns the Workbench native primary-press popup-dismiss boundary. It is now a structural entry that preserves the native pointer dispatch import while delegating behavior to the dispatch and target child owners.

`native_popup_dismiss/dispatch.rs` owns the outside-click cancellation flow. It reads the current pane interaction state, resolves the active popup bounds, ignores clicks inside the popup target, unions any existing damage with the popup damage frame, invokes the popup cancel action, clears hovered template state, and returns a region frame-update result.

`native_popup_dismiss/target.rs` owns active popup target discovery. It scans retained Workbench template nodes in reverse paint order, builds dropdown/menu dismiss targets, keeps containment separate from repaint damage, and only returns a target when the node is hovered, focused, or selected. Dropdown targets use the projected option popup frame from `template_popup_layout`; menu targets use their control frame as their dismiss containment.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
