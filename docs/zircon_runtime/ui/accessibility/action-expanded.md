---
related_code:
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/action/expanded.rs
  - zircon_runtime/src/ui/accessibility/action/expanded/result.rs
  - zircon_runtime/src/ui/accessibility/action/expanded/target.rs
  - zircon_runtime/src/ui/accessibility/action/popup.rs
implementation_files:
  - zircon_runtime/src/ui/accessibility/action.rs
  - zircon_runtime/src/ui/accessibility/action/expanded.rs
  - zircon_runtime/src/ui/accessibility/action/expanded/result.rs
  - zircon_runtime/src/ui/accessibility/action/expanded/target.rs
  - zircon_runtime/src/ui/accessibility/action/popup.rs
plan_sources:
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
tests:
  - zircon_runtime/src/ui/tests/accessibility_widget_actions.rs
---

# Accessibility Expanded Action Module

## Scope

`zircon_runtime/src/ui/accessibility/action/expanded.rs` owns accessibility `Expand` and `Collapse` dispatch for expandable runtime UI nodes. The module covers role/action exposure checks, `state.expanded` availability checks, expandable target lookup, and retained/runtime-state mutation through `UiSurface::mutate_property`.

`zircon_runtime/src/ui/accessibility/action/expanded/result.rs` owns the mutation result boundary below that dispatcher. It maps accepted, unchanged, rejected, and failed property mutations to the same neutral action status notes, binding-report diagnostics, and popup/disclosure component event reports used before the split.

`zircon_runtime/src/ui/accessibility/action/expanded/target.rs` owns the expandable target lookup beneath that dispatcher. It matches Disclosure and Popup widget behavior, resolves authored `open_property` aliases with the retained fallback names, carries the internal target kind, and maps accepted state changes to `ToggleExpanded`, `OpenPopup`, or `ClosePopup` component events.

`zircon_runtime/src/ui/accessibility/action/popup.rs` now stays focused on accessibility `Dismiss`: popup close mutation through the default popup dismissal target and tooltip hide routing through `UiDispatchEffect::Tooltip`. Keeping expandable state mutation in `expanded.rs` prevents Disclosure expansion from being coupled to popup/tooltip dismissal details.

## Dispatch Boundary

The top-level `action.rs` dispatcher still captures the accessibility snapshot, validates target availability through `target.rs`, and routes `Expand` / `Collapse` to `dispatch_expanded_state`. The split is structural: action status notes, mutation diagnostics, binding reports, and component event payloads remain the same as the previous popup-owned implementation.

The `expanded/result.rs` and `expanded/target.rs` child modules are intentionally not public outside the accessibility action subtree. `result.rs` keeps mutation output shaping local to expandable actions, while `target.rs` keeps widget-contract interpretation local to expandable actions and `expanded.rs` remains the mutation/action boundary consumed by the root dispatcher.

## Validation

- `rustfmt --edition 2021 --check` passed for `action.rs`, `action/expanded.rs`, `action/expanded/result.rs`, `action/expanded/target.rs`, `action/popup.rs`, and the sibling accessibility action modules.
- Scoped `git diff --check` passed for tracked touched files with LF/CRLF warnings only.
- Touched-file trailing-whitespace and conflict-marker scans passed for the Rust files, this document, the milestone plan, and the active session note.
- Cargo validation was not started because unrelated Cargo/Rust compiler jobs remained active in the shared checkout.
- The existing `docs/zircon_runtime/ui/accessibility.md` overview and machine-readable header now include the `action/expanded.rs`, `action/expanded/result.rs`, and `action/expanded/target.rs` ownership boundaries.
