---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callback_methods.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/host.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/types.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/asset_data.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callback_methods.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/host.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/types.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/asset_data.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context/setters/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - globals state/context/callback ownership scan
  - globals pane-context setters/callbacks ownership scan
  - globals pane-context setter family ownership scan
  - globals callback host/pane/type ownership scan
  - scoped whitespace scan
  - scoped git diff --check
  - cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Host Contract Globals

`globals.rs` is the retained-host global context entry. It keeps the stable exports for `HostContractState`, `HostContractGlobal`, `UiHostContext`, and `PaneSurfaceHostContext` while concrete state storage, host-level context methods, pane-level context methods, callback storage, and callback registration macros live in child modules.

## Purpose

`globals/state.rs` keeps `HostContractState` as the single state cell behind `UiHostWindow`. Window size, window scale factor, visibility, presentation snapshots, diagnostics counters, redraw queue state, viewport image data, menu/pane interaction state, drag/resize state, focused text input, Welcome pane state, and callback registries all live in this state object.

`globals/ui_context.rs` exposes host-level state and chrome callbacks: menu pointer events, activity rail clicks, document and drawer tab clicks, floating header clicks, drag/resize forwarding, frame requests, close-prompt actions, and unhandled keyboard forwarding.

`globals/pane_context.rs` owns the `PaneSurfaceHostContext` wrapper and `HostContractGlobal` construction. `globals/pane_context/setters.rs` is now a structural setter-family entry. `setters/welcome.rs` owns Welcome/project-overview pane setters, `setters/viewport.rs` owns viewport image and mesh-import setters, `setters/asset_data.rs` owns projected asset pane data placeholders, and `setters/interaction.rs` owns retained pane interaction state mutation. `globals/pane_context/callbacks.rs` exposes pane-level callback registration/invocation for Welcome, hierarchy, console, inspector, component showcase, asset panes, viewport, and UI asset detail/collection callbacks.

## Callback Ownership

`globals/callbacks.rs` is now the structural callback registry entry. The concrete callback storage DTOs and arity aliases live in focused children:

- `callbacks/host.rs` owns `UiHostCallbacks` for native host/chrome callbacks.
- `callbacks/pane.rs` owns `PaneSurfaceCallbacks` for pane surface callbacks.
- `callbacks/types.rs` owns `Callback0` through `Callback8`, the private aliases for stored `Rc<dyn Fn(...)>` callback shapes.

`globals/callback_methods.rs` owns the `callback_methods!` macro used by both context owners. The macro is still expanded inside the concrete context modules so registration and invocation methods remain part of the public context APIs, while the macro definition no longer lives in the root file.

## Behavior Model

All context objects are lightweight wrappers around `Rc<RefCell<HostContractState>>`. `HostContractGlobal::from_state(...)` constructs a context for a specific API surface without cloning the state payload itself.

Registration methods store callbacks into the relevant callback registry. Invocation methods clone the current callback out of the `RefCell` before invoking it, so callback execution does not hold the host state borrow while user code runs.

Setter methods either update retained-host state directly or are placeholders for still-projected pane data that is owned elsewhere in the editor pipeline. Values that represent scroll offsets are clamped to non-negative ranges before being stored. The callback child keeps macro expansion near the pane API surface while the parent context file remains structural.

## Design And Rationale

The root file remains the stable import owner because callers depend on `globals::{PaneSurfaceHostContext, UiHostContext}` and `window.rs` depends on `HostContractState`/`HostContractGlobal`. Concrete state and context behavior are now folder-backed so future callback family or pane API splits can happen without turning `globals.rs` back into a large declaration file.

This shape follows the 08 M3.S2 owner-shrink rule: root files keep stable module contracts and narrow re-exports, while state, callback storage, callback macros, and context method families move into child modules. It also avoids adding a compatibility façade; callers keep using the same context methods.

## Edge Cases And Constraints

- `HostContractState` must remain the only shared storage cell used by both contexts.
- Window scale factor must remain normalized at the state boundary. Invalid, non-finite, or non-positive values fall back to 1.0 before layout code observes them.
- Callback invocation must not keep a mutable or immutable state borrow across callback execution.
- Callback storage stays private to the globals subtree; external modules should register/invoke through the context methods.
- Pane methods that are still placeholders should not silently grow into routing or projection logic inside `globals.rs`; concrete pane behavior belongs in the pane conversion, callback dispatch, native pointer, or template-node owner.

## Test Coverage

This slice was validated with `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a globals state/context/callback ownership scan, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.

The 2026-06-21 callback host/pane/type split reduced `globals/callbacks.rs` from 130 lines to a 5-line structural entry. `callbacks/host.rs` owns host/chrome callback storage, `callbacks/pane.rs` owns pane surface callback storage, and `callbacks/types.rs` owns shared callback arity aliases. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a globals callback host/pane/type ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 pane-context setters/callbacks split reduced `globals/pane_context.rs` from 161 lines to a 22-line context wrapper entry. `pane_context/setters.rs` is 114 lines and owns pane state setter/placeholders plus direct retained state mutations, while `pane_context/callbacks.rs` is 51 lines and owns pane callback registration/invocation macro expansion. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a globals pane-context setters/callbacks ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 pane-context setter family split reduced `globals/pane_context/setters.rs` from 114 lines to a 4-line structural entry. `setters/welcome.rs` owns Welcome/project-overview state setters, `setters/viewport.rs` owns viewport image conversion and mesh-import placeholder, `setters/asset_data.rs` owns asset pane projected-data placeholders, and `setters/interaction.rs` owns hierarchy/asset interaction state mutation plus remaining scroll/hover placeholders. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a globals pane-context setter family ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-07-05 logical-width breakpoint cutover added normalized `window_scale_factor` storage to `HostContractState` so retained host layout can consume DPI scale through the same shared state cell as size and visibility. Validation used `cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --nocapture --test-threads=1`, which passed 1/1.

## Plan Sources

This module is part of `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, specifically the M3.S2 retained-host owner shrink work that prepares the editor shell to finish moving from retained software projection toward runtime UI extract and GPU command stream ownership.
