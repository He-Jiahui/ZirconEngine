---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/ime.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/keys.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/modifiers.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/wheel.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/ime.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/keys.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/modifiers.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_input_translation/wheel.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract native-input keyboard/ime/wheel/key/modifier ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Native Input Translation

`native_input_translation.rs` is the retained-host native input conversion entry. It keeps the stable test-only exports used by host-contract tests while moving keyboard, IME, wheel, key-code, and modifier conversion into child modules.

## Keyboard Ownership

`native_input_translation/keyboard.rs` owns conversion from winit `KeyEvent` plus modifier state into `UiInputEvent::Keyboard`. It applies shared metadata modifier state, synthetic status, keyboard state, legacy key code, scan code, physical key name, logical key name, and text payload.

`native_input_translation/keys.rs` owns keyboard helper rules: pressed/repeated/released state mapping, physical/logical key names, legacy web-style key-code fallback, single-character key-code mapping, and native scan-code extraction.

## Modifier Ownership

`native_input_translation/modifiers.rs` owns `ModifiersState` to `UiInputModifiers` conversion. Caps-lock and num-lock remain explicit false values because the current winit path does not supply those lock states through this call site.

## IME Ownership

`native_input_translation/ime.rs` owns winit IME conversion. Preedit becomes `UiImeInputEventKind::Preedit`, commit becomes `UiInputEvent::Text`, disabled becomes IME cancel, and enabled/delete-surrounding events are ignored. It also owns cursor byte-range clamping.

## Wheel Ownership

`native_input_translation/wheel.rs` owns mouse wheel conversion. Line deltas preserve line units; pixel deltas produce precise pixel scroll and legacy line-scale fallback for older pointer consumers.

## Root Boundary

The root `native_input_translation.rs` only declares child modules and re-exports `native_keyboard_event_to_shared_input(...)`, `native_ime_event_to_shared_input(...)`, and `native_mouse_wheel_event_to_shared_input(...)`. It should not regain winit imports, key-code tables, modifier conversion, IME match logic, wheel scaling, or helper bodies.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `native_input_translation.rs` no longer owns keyboard, IME, wheel, key, or modifier conversion bodies, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
