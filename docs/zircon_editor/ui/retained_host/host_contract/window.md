---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/constants.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/constants.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - window root presentation/diagnostics/redraw ownership scan
  - window event-loop subtree ownership scan
  - window event-loop/template-hover ownership scan
  - window module-local test ownership scan
  - window text-input/test-support ownership scan
  - window handle/snapshot ownership scan
  - window lifecycle/constants/metadata ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Host Contract Window

`window.rs` is the retained editor host's native-window boundary. It now owns only `UiHostWindow`, the shared host state field, child module wiring, and handle re-exports. Lifecycle construction, native metadata, presentation mutation, refresh diagnostics, redraw queueing, event-loop mechanics, text-editing semantics, and test-only native input helpers live under explicit child owners.

## Lifecycle Ownership

`window/lifecycle.rs` owns `UiHostWindow` construction, clone/show/hide/run entry points, handle construction, close-request response, host globals, and exit requests. The module keeps the shared state constructor and platform run path close to the `UiHostWindow` type while preventing lifecycle methods from growing back into the root module.

`window/constants.rs` owns the default native editor canvas size and the stable native host window id used by runtime input metadata and deterministic test metadata.

## Native Metadata Ownership

`window/metadata.rs` owns native input timestamp/sequence/window-id construction and platform error conversion. Event-loop input sequencing calls this module directly, and test support uses the shared host window id from `window/constants.rs` for deterministic keyboard metadata.

## Presentation Ownership

`window/presentation.rs` owns host presentation mutation and cloning. It increments presentation rebuild counters, emits verbose rebuild diagnostics, exposes menu and pane interaction state snapshots, mutates close-prompt state and its redraw damage, records transient hovered template node/row state, builds bootstrap geometry, and applies template hover state to cloned presentation data.

Screenshot capture in `window/handle.rs` uses the same module-owned `host_presentation_from_state(...)` helper so snapshot rendering and normal presentation reads share the same hover/text/viewport-image aggregation path without moving clone logic back into `window.rs`.

## Diagnostics Ownership

`window/diagnostics.rs` owns refresh invalidation diagnostics and refresh-rate overlay text mutation. The event-loop redraw path reads the invalidation counters before presenter `present(...)` and writes the presenter diagnostics overlay after present succeeds.

Keeping these methods in a child module prevents `window.rs` from accumulating presenter instrumentation while preserving direct access for event-loop redraw and module-local tests.

## Redraw Ownership

`window/redraw.rs` owns frame-update requests, completed frame-update scenario handoff, region redraw requests, frame-update redraw requests, external redraw queue coalescing, and queue drain accounting.

Event-loop redraw drains this module-owned queue in `window/event_loop/redraw.rs`, while higher-level retained host callbacks call the public region request methods. This keeps redraw queue policy separate from platform `WindowEvent` dispatch and from presentation mutation.

## Event Loop Ownership

`window/event_loop.rs` owns only the winit `ApplicationHandler` entry and the `UiHostWindowEventLoop` state fields. Its child modules own the concrete behavior families:

- `window/event_loop/lifecycle.rs` creates the native window, chooses GPU or softbuffer presenter backends, logs presenter creation, syncs native size/position/maximized state, and handles `about_to_wait`.
- `window/event_loop/events.rs` owns the `WindowEvent` match and delegates pointer, keyboard, IME, scroll, resize, close, and redraw branches to focused helpers.
- `window/event_loop/input.rs` owns native input helper conversion: pointer button mapping, button state mapping, scroll delta mapping, IME enablement, and input metadata sequence advancement.
- `window/event_loop/redraw.rs` owns pending/external redraw merge/drain, pointer dispatch redraw fanout, frame-update scenario attribution, presenter `present(...)`, profiling artifact export, refresh diagnostics overlay update, and presenter failure exit.

This separation keeps platform lifecycle, event dispatch, input mapping, and present/redraw orchestration visible in the path while `window.rs` remains the stable API used by retained host callbacks and tests. The event-loop subtree may access `UiHostWindow` internals because it is a child of the window module, but it should not grow Workbench hit routing or text-editing semantics; those stay in `native_pointer.rs`, `native_keyboard.rs`, and the parent window state API.

## Text Input Ownership

`window/text_input.rs` owns focused text edit dispatch, text insertion/backspace/commit, popup keyboard fallback before text focus is active, unhandled keyboard forwarding into shared runtime input, and dispatch of edited values to Welcome, showcase, inspector, asset, and generic surface callbacks.

The module is still an inherent `UiHostWindow` implementation because event-loop and tests call the same host methods, but the behavior no longer lives in the root window boundary. This keeps native window state, redraw queueing, and snapshot APIs separate from focused text semantics.

## Handle Ownership

`window/handle.rs` owns the external `HostWindowHandle` API plus `HostWindowSnapshot`. It mutates native window position, size, visibility, maximized state, close-request callback registration, and snapshot capture through the retained presentation snapshot path.

This keeps handle-facing window mutation and snapshot byte packaging out of the `UiHostWindow` state boundary. The root window module still constructs handles because it owns the shared state, but the handle module owns the public operations exposed through the handle.

## Template Hover Ownership

`window/template_hover.rs` owns the transient hover overlay applied when `UiHostWindow::get_host_presentation()` clones presentation state. Pointer move stores the currently hovered template control plus optional structured row identity in `HostPaneInteractionStateData`; the hover module then applies that state to the clone only.

The module walks Workbench window nodes, dock panes, and floating-window active panes. It sets the matching template node's `hovered` flag, rewrites structured dropdown option hover/focus/pressed flags for `workbench_option`, and rewrites structured popup menu row flags for `workbench_menu_item`. The underlying componentized Workbench surface remains unchanged until an actual click/change/commit callback mutates it.

## Module-local Tests

`window/tests.rs` owns the tests that need direct access to `UiHostWindow` internals. It covers refresh diagnostics overlay text, close-request callback mutation without reentrant state borrowing, frame-update redraw requests that preserve damage regions, and one-shot completed frame-update scenario storage. Keeping these regressions in a child module lets `window.rs` remain the production host-state boundary without embedding test bodies at the bottom of the file.

`window/test_support.rs` owns `#[cfg(test)]` native input helper methods on `UiHostWindow`: direct native keyboard dispatch, pointer move/press/release/scroll helpers, focused text helpers, popup navigation helpers, and deterministic keyboard metadata. Keeping this file separate prevents the root window boundary from accumulating test-only native input adapters.

## Boundary Rules

- Keep `window.rs` focused on the `UiHostWindow` state field, child module wiring, and handle re-exports.
- Keep default native window constants in `window/constants.rs`.
- Keep host construction, show/hide/run lifecycle entry points, host globals, close-request response, and exit requests in `window/lifecycle.rs`.
- Keep native input metadata for the event loop and platform error conversion in `window/metadata.rs`.
- Keep host presentation mutation, presentation snapshot aggregation, close-prompt redraw damage, transient template hover state, and bootstrap geometry in `window/presentation.rs`.
- Keep refresh invalidation diagnostics and refresh overlay text mutation in `window/diagnostics.rs`.
- Keep frame-update requests, redraw-region requests, external redraw queue coalescing, and external redraw drain accounting in `window/redraw.rs`.
- Keep `HostWindowHandle`, native-window handle mutation, close callback registration, and `HostWindowSnapshot` byte packaging in `window/handle.rs`.
- Keep winit event matching, native presenter creation/fallback, IME toggling, pointer button mapping, scroll delta mapping, and redraw scheduling inside `window/event_loop.rs`.
- Keep focused text-input editing, text commit routing, and unhandled keyboard forwarding inside `window/text_input.rs`.
- Keep cloned presentation hover mutation inside `window/template_hover.rs`; do not add node-model mutation helpers back to `window.rs`.
- Keep module-local host-window regressions in `window/tests.rs`; do not reintroduce inline test bodies into `window.rs`.
- Keep test-only native input adapters in `window/test_support.rs`; do not add `#[cfg(test)]` pointer/key helper blocks back to `window.rs`.
- Keep semantic pointer routing and damage calculation in `native_pointer/`, and keep popup keyboard navigation in `native_keyboard.rs`.

## Validation Notes

The 2026-06-18 split is implementation-first. The slice uses formatting, ownership scans, trailing-whitespace/diff checks, and a scoped `zircon_editor` library type check as its evidence. Focused native-window interaction tests and the full Cargo test matrix remain deferred to the milestone testing stage per the user's instruction.

The follow-up test split keeps the event-loop/template-hover structure intact while moving the remaining inline window regressions into `window/tests.rs`. After the split, `window.rs` is 817 lines, `window/event_loop.rs` is 381 lines, `window/template_hover.rs` is 168 lines, and `window/tests.rs` is 83 lines.

The 2026-06-18 text-input/test-support split reduced `window.rs` to 382 lines. The new owners are `window/text_input.rs` at 199 lines for focused text and keyboard dispatch and `window/test_support.rs` at 148 lines for test-only native input helper methods. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window text-input/test-support ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 handle/snapshot split reduced `window.rs` to 322 lines. `window/handle.rs` is 69 lines and owns `HostWindowHandle`, `HostWindowSnapshot`, handle mutation methods, close callback registration, and snapshot construction. Formatting and ownership scans passed; the scoped `zircon_editor` library type check is currently blocked before editor analysis by unrelated dirty `zircon_runtime` render graph changes (`ParticleGpuTransparentDrawContext` export drift, `execute_graph_stage` call arity/type mismatches, and a particle encoder borrow error). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 event-loop subtree split reduced `window/event_loop.rs` from 381 lines to 67 by moving native window lifecycle/presenter creation into `event_loop/lifecycle.rs`, winit event matching into `events.rs`, native input conversion into `input.rs`, and redraw/present orchestration into `redraw.rs`. Current line counts are `lifecycle.rs` 115, `events.rs` 117, `redraw.rs` 89, and `input.rs` 54. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window event-loop root ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 window presentation/diagnostics/redraw split reduced `window.rs` to 117 lines. `window/presentation.rs` is 171 lines and owns host presentation mutation, presentation clone aggregation, close-prompt damage, template hover state writes, and bootstrap geometry; `window/diagnostics.rs` is 26 lines and owns refresh diagnostics state/overlay mutation; `window/redraw.rs` is 66 lines and owns frame-update requests plus external redraw queue coalescing/drain accounting. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window root presentation/diagnostics/redraw ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-20 lifecycle/constants/metadata split reduced `window.rs` to a 22-line structural entry. `window/constants.rs` is 5 lines and owns default size/window-id constants; `window/lifecycle.rs` is 62 lines and owns host construction, lifecycle methods, close-request response, globals, handle construction, and exit requests; `window/metadata.rs` is 21 lines and owns input metadata and platform error conversion. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming lifecycle/metadata behavior no longer lives in `window.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.
