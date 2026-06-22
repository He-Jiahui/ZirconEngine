---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/unhandled.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime_interface/src/ui/window/input.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard/unhandled.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-23 implement editor UI architecture plan and record status
tests:
  - cargo fmt -p zircon_editor --check
  - cargo test -p zircon_editor --lib platform_input_translation --locked (blocked before Rust diagnostics by existing Cargo.toml/Cargo.lock drift)
  - cargo test -p zircon_editor --lib platform_input_translation --offline (supplemental attempt timed out after 600s; Cargo.lock restored to pre-attempt bytes)
  - rustfmt --edition 2021 --check touched editor/runtime platform-input files
  - scoped trailing-whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Retained Host Platform Input

## Purpose

The retained editor host no longer owns winit-to-shared-input translation. Editor UI 01.M1.S3 switches the live winit event loop to the runtime-owned `zircon_runtime::ui::platform_input` adapter and deletes the editor-local `host_contract/native_input_translation*` Rust sources.

`window/event_loop/platform_input.rs` is intentionally a thin bridge. It builds `UiWindowInputContext` with the retained host input sequence and current modifier state, calls `translate_winit_window_event(...)`, and exposes small extractors for keyboard, text, pointer, and cursor-move pump events.

## Ownership

- Runtime owns platform semantics: key naming, legacy key code fallback, native scan codes, modifiers, IME normalization, wheel precision, touch pointer reconstruction, and winit pointer button mapping.
- Editor owns native event-loop lifetime, window state synchronization, presenter resize/redraw, retained text focus behavior, and retained pointer route dispatch until later input-manager slices replace those internals.
- `window/text_input/keyboard.rs` now receives an optional `UiKeyboardInputEvent` from runtime translation. Focused text and popup fallback still inspect the raw `KeyEvent`, but unhandled command binding receives the runtime DTO directly.
- `window/event_loop/events/pointer.rs` now reads pointer move/button/scroll data from the normalized pump event. Touch-sourced pointer events are intentionally not routed into the old mouse-only retained pointer bridge; multi-pointer routing is still a later M4 responsibility.

## Deleted Editor-Local Path

The following Rust sources were removed in 01.M1.S3:

- `zircon_editor/src/ui/retained_host/host_contract/native_input_translation.rs`
- `zircon_editor/src/ui/retained_host/host_contract/native_input_translation/{ime,keyboard,keys,modifiers,wheel}.rs`
- `zircon_editor/src/ui/retained_host/host_contract/native_input_translation/keys/{legacy,names,scan,state}.rs`
- `zircon_editor/src/tests/host/retained_window/native_input_translation.rs`

`host_contract/mod.rs` no longer declares or test-re-exports native input translation helpers. The remaining `native_keyboard.rs` module is not winit translation; it owns retained workbench popup keyboard commands and stays in place.

## Behavior Notes

Keyboard handling preserves the old retained order: focused text/popup behavior runs first, then unconsumed pressed keys are forwarded to workbench command binding as `UiKeyboardInputEvent`. Runtime `translate_winit_window_event(...)` now preserves winit's `is_synthetic` flag so retained tests can keep their previous synthetic-key semantics.

IME commit is consumed as runtime text input and inserted through the existing focused text path. Runtime also produces IME preedit, disabled-as-cancel, and delete-surrounding events. After 01.M1.S4 delete-surrounding is a shared runtime-interface DTO rather than an editor-local special case; the retained host still leaves preedit/cancel/delete-surrounding application to the later input-manager/text-policy slices.

Mouse wheel events use runtime's precise line/pixel normalization for the scroll scalar, while retained pointer dispatch still supplies the last known pointer position because winit `MouseWheel` does not carry a cursor point.

Mouse pointer move/button events are routed through normalized cursor/pointer pump events and then into the current retained pointer bridge. Touch pointer events are translated by runtime but remain blocked from the old retained mouse bridge to avoid synthesizing partial touch behavior before the planned input manager and active pointer table land.

## Tests And Guards

`zircon_editor/src/tests/host/retained_window/platform_input_translation.rs` replaces the old native translation test file. It verifies runtime public translation for keyboard, synthetic flags, native scan codes, IME preedit/commit/disable/delete-surrounding, wheel precision, and touch pointer reconstruction, then checks that the editor event loop calls the runtime platform-input adapter.

`zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs` now guards the hard cutover: the editor-local native translation entry must be absent, the old helper exports must not return to `host_contract/mod.rs`, and the event loop adapter must keep calling `translate_winit_window_event(...)`.

## Validation Notes

`cargo fmt -p zircon_editor --check`, touched-file `rustfmt --check`, scoped trailing-whitespace scan, and scoped `git diff --check` pass for this slice.

The planned `cargo test -p zircon_editor --lib platform_input_translation --locked` is blocked before Rust diagnostics because the current workspace `Cargo.toml`/`Cargo.lock` state requires a lockfile update. A supplemental offline attempt without `--locked` was run only to seek compile diagnostics; it timed out after 600 seconds without diagnostics. The running cargo/rustc processes were stopped and `Cargo.lock` was restored to the exact pre-attempt hash.
