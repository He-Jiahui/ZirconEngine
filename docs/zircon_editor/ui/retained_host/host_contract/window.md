---
related_code:
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/constants.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/native_window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/close_prompt.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/template_hover_state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/consumed.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/unhandled.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/constants.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/native_window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle/presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/handle/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/close_prompt.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/template_hover_state.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/edit/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/consumed.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/unhandled.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - window root presentation/diagnostics/redraw ownership scan
  - window event-loop subtree ownership scan
  - window event-loop event keyboard/pointer/resize ownership scan
  - window event-loop/template-hover ownership scan
  - window module-local test ownership scan
  - window text-input/test-support ownership scan
  - window text-input edit/keyboard ownership scan
  - window text-input edit dispatch/redraw ownership scan
  - window text-input keyboard popup/consumed/unhandled ownership scan
  - window event-loop lifecycle native-window/presenter ownership scan
  - window event-loop redraw present ownership scan
  - window handle/snapshot ownership scan
  - window handle snapshot ownership scan
  - window lifecycle/constants/metadata ownership scan
  - window presentation close-prompt/snapshot/template-hover ownership scan
  - window template-hover pane/node/row ownership scan
  - cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --nocapture --test-threads=1
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor first_presented_frame_exit_policy_defaults_off_and_can_be_enabled --lib --no-default-features --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-popup-preference-0704 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Host Contract Window

`window.rs` is the retained editor host's native-window boundary. It owns `UiHostWindow`, the shared host state field, narrow profile-artifact job ownership, child module wiring, and handle re-exports. Lifecycle construction, native metadata, presentation mutation, refresh diagnostics, redraw queueing, event-loop mechanics, text-editing semantics, and test-only native input helpers live under explicit child owners.

## Lifecycle Ownership

`window/lifecycle.rs` owns `UiHostWindow` construction, clone/show/hide/run entry points, handle construction, close-request response, host globals, and exit requests. Startup injects the single `EditorJobSystem` into the profile-artifact owner before host assembly; a replaced or final `UiHostWindow` owner requests cooperative cancellation of its active artifact `JobId`. The module keeps the shared state constructor and platform run path close to the `UiHostWindow` type while preventing lifecycle methods from growing back into the root module.

`window/constants.rs` owns the default native editor canvas size and the stable native host window id used by runtime input metadata and deterministic test metadata.

## Native Metadata Ownership

`window/metadata.rs` owns native input timestamp/sequence/window-id construction and platform error conversion. Event-loop input sequencing calls this module directly, and test support uses the shared host window id from `window/constants.rs` for deterministic keyboard metadata.

## Presentation Ownership

`window/presentation.rs` is the structural presentation API and rebuild-diagnostic owner. It increments presentation rebuild counters, emits verbose rebuild diagnostics, exposes menu and pane interaction state snapshots, and delegates specialized state updates to child owners: `presentation/close_prompt.rs` owns close-prompt mutation and redraw damage, `presentation/template_hover_state.rs` owns transient hovered template node/row writes and clearing, and `presentation/snapshot.rs` owns bootstrap geometry plus cloned presentation aggregation with template hover application.

Screenshot capture in `window/handle.rs` uses the same module-owned `host_presentation_from_state(...)` helper so snapshot rendering and normal presentation reads share the same hover/text/viewport-image aggregation path without moving clone logic back into `window.rs`.

## Diagnostics Ownership

`window/diagnostics.rs` owns refresh invalidation diagnostics and refresh-rate overlay text mutation. The event-loop redraw path reads the invalidation counters before presenter `present(...)` and writes the presenter diagnostics overlay after present succeeds.

Keeping these methods in a child module prevents `window.rs` from accumulating presenter instrumentation while preserving direct access for event-loop redraw and module-local tests.

## Redraw Ownership

`window/redraw.rs` owns frame-update requests, completed frame-update scenario handoff, region redraw requests, frame-update redraw requests, external redraw queue coalescing, and queue drain accounting.

Event-loop redraw drains this module-owned queue in `window/event_loop/redraw.rs`, while higher-level retained host callbacks call the public region request methods. This keeps redraw queue policy separate from platform `WindowEvent` dispatch and from presentation mutation.

## Event Loop Ownership

`window/event_loop.rs` owns only the winit `ApplicationHandler` entry and the `UiHostWindowEventLoop` state fields. Its child modules own the concrete behavior families:

- `window/event_loop/lifecycle.rs` orchestrates startup, logs presenter creation, syncs native size/position/maximized state, and handles `about_to_wait`; `lifecycle/native_window.rs` owns native window creation and `lifecycle/presenter.rs` owns GPU presenter creation plus softbuffer fallback.
- `window/event_loop/events.rs` owns the `WindowEvent` match and close/redraw branch ordering; `events/pointer.rs` owns pointer move/button/scroll dispatch, `events/keyboard.rs` owns keyboard/IME dispatch, and `events/resize.rs` owns surface resize/move handling.
- `window/event_loop/input.rs` owns native input helper conversion: pointer button mapping, button state mapping, scroll delta mapping, IME enablement, and input metadata sequence advancement.
- `window/event_loop/redraw.rs` owns pending/external redraw merge/drain, pointer dispatch redraw fanout, frame-update scenario attribution, and present dispatch.
- `window/event_loop/redraw/present.rs` owns presenter `present(...)`, profiling artifact export, refresh diagnostics overlay update, and presenter failure exit.

This separation keeps platform lifecycle, event dispatch, input mapping, and present/redraw orchestration visible in the path while `window.rs` remains the stable API used by retained host callbacks and tests. The event-loop subtree may access `UiHostWindow` internals because it is a child of the window module, but it should not grow Workbench hit routing or text-editing semantics; those stay in `native_pointer.rs`, `native_keyboard.rs`, and the parent window state API.

The bounded first-frame startup smoke follows the same ownership boundary. `EditorHostRunConfig` enables the default-off policy, `window/lifecycle.rs` stores it on `UiHostWindow`, `host_contract/globals/state.rs` holds the flag, and `window/event_loop/redraw/present.rs` exits only after a successful presenter `present(...)` and diagnostics overlay update. Normal editor windows leave the flag disabled.

## Text Input Ownership

`window/text_input.rs` is now a structural entry for focused text handling. `window/text_input/edit.rs` owns text focus state checks, insertion, backspace, and commit entry dispatch. `window/text_input/edit/dispatch.rs` owns edited-value fanout to Welcome/showcase/inspector/asset/generic surface callbacks, and `window/text_input/edit/redraw.rs` owns text-focus redraw damage.

`window/text_input/keyboard.rs` owns focused key dispatch and native keyboard event orchestration. `keyboard/popup.rs` owns popup keyboard command/text-search fallback before text focus is active, `keyboard/consumed.rs` owns keyboard-consumption decisions after text focus, and `keyboard/unhandled.rs` owns unhandled keyboard forwarding into shared runtime input. Both children remain inherent `UiHostWindow` implementations because event-loop and tests call the same host methods, but the behavior no longer lives in the root window boundary. This keeps native window state, redraw queueing, and snapshot APIs separate from focused text semantics.

## Handle Ownership

`window/handle.rs` owns the external `HostWindowHandle` API, native window position/size/visibility/maximized mutation, close-request callback registration, and snapshot capture through the retained presentation snapshot path. `window/handle/snapshot.rs` owns the `HostWindowSnapshot` payload, RGBA byte storage, and width/height accessors.

This keeps handle-facing window mutation and snapshot byte packaging out of the `UiHostWindow` state boundary. The root window module still constructs handles because it owns the shared state, but the handle module owns the public operations exposed through the handle.

The handle also exposes the retained host window scale factor. `HostContractState` stores a normalized `window_scale_factor` with a default of 1.0, `HostWindowHandle::scale_factor()` reads it, and `HostWindowHandle::set_scale_factor(...)` gives tests and host integration a narrow write path that filters invalid values back to 1.0. Native winit lifecycle sync updates that state from `Window::scale_factor()` when the platform window is available, so retained-host layout code can consume a stable host-contract fact instead of reaching into platform APIs.

## Template Hover Ownership

`window/template_hover.rs` owns the transient hover overlay entry applied when `UiHostWindow::get_host_presentation()` clones presentation state. Pointer move stores the currently hovered template control plus optional structured row identity in `HostPaneInteractionStateData`; the hover module then applies that state to the clone only.

`template_hover/panes.rs` walks dock panes and floating-window active panes before handing pane template-node lists to the node owner. `template_hover/nodes.rs` sets the matching template node's `hovered` flag and delegates structured row overlays. `template_hover/rows.rs` rewrites structured dropdown option hover/focus/pressed flags for `workbench_option` and structured popup menu row flags for `workbench_menu_item`. The underlying componentized Workbench surface remains unchanged until an actual click/change/commit callback mutates it.

## Module-local Tests

`window/tests.rs` owns the tests that need direct access to `UiHostWindow` internals. It covers refresh diagnostics overlay text, close-request callback mutation without reentrant state borrowing, frame-update redraw requests that preserve damage regions, and one-shot completed frame-update scenario storage. `window/profile_artifact_job_tests.rs` separately owns the injected job-system and final-owner cancellation regressions. Keeping these regressions in child modules lets `window.rs` remain the production host-state boundary without embedding test bodies at the bottom of the file.

`window/test_support.rs` owns `#[cfg(test)]` native input helper methods on `UiHostWindow`: direct native keyboard dispatch, pointer move/press/release/scroll helpers, focused text helpers, popup navigation helpers, and deterministic keyboard metadata. Keeping this file separate prevents the root window boundary from accumulating test-only native input adapters.

## Boundary Rules

- Keep `window.rs` focused on the `UiHostWindow` state field, child module wiring, and handle re-exports.
- Keep default native window constants in `window/constants.rs`.
- Keep host construction, show/hide/run lifecycle entry points, host globals, close-request response, and exit requests in `window/lifecycle.rs`.
- Keep native input metadata for the event loop and platform error conversion in `window/metadata.rs`.
- Keep presentation rebuild diagnostics and menu/pane snapshot getters in `window/presentation.rs`; keep close-prompt redraw damage in `window/presentation/close_prompt.rs`, transient template hover state writes in `window/presentation/template_hover_state.rs`, and bootstrap/snapshot aggregation in `window/presentation/snapshot.rs`.
- Keep refresh invalidation diagnostics and refresh overlay text mutation in `window/diagnostics.rs`.
- Keep frame-update requests, redraw-region requests, external redraw queue coalescing, and external redraw drain accounting in `window/redraw.rs`.
- Keep `HostWindowHandle`, native-window handle mutation, close callback registration, and snapshot capture in `window/handle.rs`; keep `HostWindowSnapshot` byte packaging and dimension accessors in `window/handle/snapshot.rs`.
- Keep window scale-factor storage in `HostContractState`, handle accessors in `window/handle.rs`, and native platform scale sync in `window/event_loop/lifecycle.rs`; layout modules should consume the host contract value rather than reading winit directly.
- Keep winit event matching, native presenter creation/fallback, IME toggling, pointer button mapping, scroll delta mapping, and redraw scheduling inside `window/event_loop.rs`; keep native-window construction in `window/event_loop/lifecycle/native_window.rs` and presenter backend creation/fallback in `window/event_loop/lifecycle/presenter.rs`.
- Keep focused text-input edit entry points inside `window/text_input/edit.rs`, callback fanout inside `window/text_input/edit/dispatch.rs`, and redraw damage fallback inside `window/text_input/edit/redraw.rs`.
- Keep focused key dispatch and native keyboard orchestration inside `window/text_input/keyboard.rs`, popup fallback inside `window/text_input/keyboard/popup.rs`, consumption decisions inside `window/text_input/keyboard/consumed.rs`, and unhandled keyboard forwarding inside `window/text_input/keyboard/unhandled.rs`.
- Keep cloned presentation hover entry orchestration inside `window/template_hover.rs`; keep pane/floating traversal in `window/template_hover/panes.rs`, template-node mutation in `window/template_hover/nodes.rs`, and structured option/menu row hover mutation in `window/template_hover/rows.rs`.
- Keep module-local host-window regressions in `window/tests.rs`; do not reintroduce inline test bodies into `window.rs`.
- Keep test-only native input adapters in `window/test_support.rs`; do not add `#[cfg(test)]` pointer/key helper blocks back to `window.rs`.
- Keep semantic pointer routing and damage calculation in `native_pointer/`, and keep popup keyboard navigation in `native_keyboard.rs`.

## Validation Notes

The 2026-06-18 split is implementation-first. The slice uses formatting, ownership scans, trailing-whitespace/diff checks, and a scoped `zircon_editor` library type check as its evidence. Focused native-window interaction tests and the full Cargo test matrix remain deferred to the milestone testing stage per the user's instruction.

The follow-up test split keeps the event-loop/template-hover structure intact while moving the remaining inline window regressions into `window/tests.rs`. After the split, `window.rs` is 817 lines, `window/event_loop.rs` is 381 lines, `window/template_hover.rs` is 168 lines, and `window/tests.rs` is 83 lines.

The 2026-06-18 text-input/test-support split reduced `window.rs` to 382 lines. The new owners are `window/text_input.rs` at 199 lines for focused text and keyboard dispatch and `window/test_support.rs` at 148 lines for test-only native input helper methods. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window text-input/test-support ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 handle/snapshot split reduced `window.rs` to 322 lines. `window/handle.rs` is 69 lines and owns `HostWindowHandle`, `HostWindowSnapshot`, handle mutation methods, close callback registration, and snapshot construction. Formatting and ownership scans passed; the scoped `zircon_editor` library type check is currently blocked before editor analysis by unrelated dirty `zircon_runtime` render graph changes (`ParticleGpuTransparentDrawContext` export drift, `execute_graph_stage` call arity/type mismatches, and a particle encoder borrow error). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-21 handle snapshot split reduced `window/handle.rs` from 85 lines to a 59-line handle API entry. `window/handle/snapshot.rs` owns `HostWindowSnapshot`, frame-to-byte conversion, and snapshot dimension/byte accessors. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window handle snapshot ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-18 event-loop subtree split reduced `window/event_loop.rs` from 381 lines to 67 by moving native window lifecycle/presenter creation into `event_loop/lifecycle.rs`, winit event matching into `events.rs`, native input conversion into `input.rs`, and redraw/present orchestration into `redraw.rs`. Current line counts are `lifecycle.rs` 115, `events.rs` 117, `redraw.rs` 89, and `input.rs` 54. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window event-loop root ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-21 event-loop lifecycle native-window/presenter split reduced `window/event_loop/lifecycle.rs`
from 118 lines to a 73-line lifecycle orchestration entry. `lifecycle/native_window.rs` owns native
window attribute construction, creation failure logging, and event-loop exit on failure, while
`lifecycle/presenter.rs` owns default GPU presenter creation, softbuffer fallback, presenter failure
logging, and exit. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`,
a window event-loop lifecycle native-window/presenter ownership scan, scoped trailing-whitespace scan,
and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the
user's feature-first instruction.

The 2026-06-21 event-loop event keyboard/pointer/resize split reduced `window/event_loop/events.rs`
from 115 lines to a 57-line event dispatch entry. `events/keyboard.rs` owns keyboard and IME commit
handoff, `events/pointer.rs` owns pointer move/button/scroll dispatch and pointer-position state, and
`events/resize.rs` owns native move, surface resize, presenter resize, resize redraw request, and
presenter resize failure exit. Validation used `cargo fmt -p zircon_editor`,
`cargo fmt -p zircon_editor --check`, a window event-loop event keyboard/pointer/resize ownership
scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and
full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 event-loop redraw present split reduced `window/event_loop/redraw.rs` from 87 lines
to a 59-line redraw queue/scenario entry. `redraw/present.rs` owns the presenter call, profiling
artifact export, refresh diagnostics overlay update, and presenter failure exit. Validation used
`cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window event-loop redraw
present ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level
Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-18 window presentation/diagnostics/redraw split reduced `window.rs` to 117 lines. `window/presentation.rs` is 171 lines and owns host presentation mutation, presentation clone aggregation, close-prompt damage, template hover state writes, and bootstrap geometry; `window/diagnostics.rs` is 26 lines and owns refresh diagnostics state/overlay mutation; `window/redraw.rs` is 66 lines and owns frame-update requests plus external redraw queue coalescing/drain accounting. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window root presentation/diagnostics/redraw ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-20 lifecycle/constants/metadata split reduced `window.rs` to a 22-line structural entry. `window/constants.rs` is 5 lines and owns default size/window-id constants; `window/lifecycle.rs` is 62 lines and owns host construction, lifecycle methods, close-request response, globals, handle construction, and exit requests; `window/metadata.rs` is 21 lines and owns input metadata and platform error conversion. Validation used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming lifecycle/metadata behavior no longer lives in `window.rs`, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction, and package-level Cargo check is still waiting on unrelated `zircon_runtime` render-history compile errors.

The 2026-06-21 window presentation close-prompt/snapshot/template-hover split reduced `window/presentation.rs` from 171 lines to a 63-line presentation API/rebuild diagnostics owner. `presentation/close_prompt.rs` is 24 lines and owns close-prompt state mutation plus redraw damage, `presentation/template_hover_state.rs` is 65 lines and owns transient hovered template node/row state writes and clearing, and `presentation/snapshot.rs` is 37 lines and owns host bootstrap geometry plus cloned presentation aggregation through `host_presentation_from_state(...)`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window presentation close-prompt/snapshot/template-hover ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 window template-hover pane/node/row split reduced `window/template_hover.rs` from 168 lines to a 19-line clone-overlay entry. `template_hover/panes.rs` is 82 lines and owns dock/floating pane traversal plus pane-kind node-list resolution, `template_hover/nodes.rs` is 31 lines and owns matching template-node hover mutation, and `template_hover/rows.rs` is 68 lines and owns structured option/menu row hover flag rewriting. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window template-hover pane/node/row ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 text-input edit/keyboard split reduced `window/text_input.rs` from 201 lines to a 2-line structural entry. `window/text_input/edit.rs` is 103 lines and owns focused text edit/value dispatch; `window/text_input/keyboard.rs` is 103 lines and owns focused key dispatch, popup fallback, and unhandled keyboard forwarding. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window text-input edit/keyboard ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check was not relaunched because the current editor package check lane has been timing out before actionable editor diagnostics, and full Cargo tests remain deferred per the user's instruction.

The 2026-06-21 text-input edit dispatch/redraw split reduced `window/text_input/edit.rs` from 110 lines to a 53-line edit entry. `edit/dispatch.rs` owns target callback fanout and asset dispatch source selection, while `edit/redraw.rs` owns text-focus redraw region fallback. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window text-input edit dispatch/redraw ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-06-21 text-input keyboard popup/consumed/unhandled split reduced `window/text_input/keyboard.rs` from 109 lines to a 55-line keyboard orchestration entry. `keyboard/popup.rs` owns popup command/text-search fallback, `keyboard/consumed.rs` owns text-focus keyboard consumption checks, and `keyboard/unhandled.rs` owns shared runtime keyboard forwarding. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, a window text-input keyboard popup/consumed/unhandled ownership scan, scoped trailing-whitespace scan, and scoped `git diff --check`; package-level Cargo check and full Cargo tests remain deferred per the user's feature-first instruction.

The 2026-07-05 logical-width breakpoint cutover added the retained host window scale-factor contract. `HostContractState` now defaults invalid or missing scale to 1.0, `window/handle.rs` exposes getter/setter access, and `window/event_loop/lifecycle.rs` syncs real winit `Window::scale_factor()` into the state before retained-host recompute. Validation used `cargo test -p zircon_editor --lib window_scale_factor_defaults_to_one_and_filters_invalid_values --locked --jobs 1 --target-dir F:\cargo-targets\zircon-editor-layout-tier-logical-0705 --message-format short --color never -- --nocapture --test-threads=1`, which passed 1/1.
