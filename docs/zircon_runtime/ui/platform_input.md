---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime_interface/src/ui/window/input.rs
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - docs/zircon_editor/ui/retained_host/host_contract/platform_input.md
implementation_files:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime_interface/src/ui/window/input.rs
  - zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - user: 2026-06-23 implement editor UI architecture plan and record status
tests:
  - cargo check -p zircon_runtime --lib --locked (attempted; blocked by pre-existing Cargo.toml/Cargo.lock drift)
  - cargo check --manifest-path E:\cargo-targets\zircon-runtime-platform-input-scratch-0623\Cargo.toml --offline
  - cargo test --manifest-path E:\cargo-targets\zircon-runtime-platform-input-scratch-0623\Cargo.toml --offline
  - cargo test -p zircon_editor --lib platform_input_translation --locked (attempted; blocked by pre-existing Cargo.toml/Cargo.lock drift)
  - cargo test -p zircon_runtime_interface --locked window_input_contracts -- --nocapture (attempted; blocked by pre-existing Cargo.toml/Cargo.lock drift)
  - cargo test -p zircon_runtime_interface --offline window_input_contracts -- --nocapture
  - cargo test -p zircon_runtime_interface --offline window_runtime_event_adapter_contracts -- --nocapture
  - cargo test -p zircon_runtime_interface --offline ui_input -- --nocapture
  - cargo test -p zircon_runtime --locked winit_translation -- --nocapture (attempted; blocked by pre-existing Cargo.toml/Cargo.lock drift)
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/platform_input/mod.rs zircon_runtime/src/ui/platform_input/keyboard_map.rs zircon_runtime/src/ui/platform_input/winit_translation.rs
  - scoped git diff --check
doc_type: module-detail
---

# Runtime Platform Input

## Purpose

`zircon_runtime::ui::platform_input` is the runtime-owned platform input adapter for the UI subsystem. The first adapter is `winit_translation`, which normalizes winit window events into the stable `zircon_runtime_interface` window pump contract.

This module was introduced for Editor UI 01.M1.S2 and is now consumed by the retained editor host as of 01.M1.S3. The long-term direction is that editor and runtime preview hosts keep their native event loops, but winit semantics are interpreted in one runtime owner before entering `UiWindowInputPumpBatch`.

## Related Files

`zircon_runtime/src/ui/mod.rs` exposes `platform_input` only when the `platform-winit` feature is enabled. `platform_input/mod.rs` keeps the public API surface small and re-exports:

- `translate_winit_window_event(context, event)`
- `translate_winit_modifiers(state)`

`winit_translation.rs` owns event-family matching and creation of `UiWindowInputPumpEvent`. `keyboard_map.rs` owns the legacy key code, native scan code, physical key name, logical key name, and pressed/repeated/released mapping copied from the editor baseline.

## Behavior Model

`translate_winit_window_event(...)` converts one winit `WindowEvent` into either a normalized input event, a window lifecycle event, or `None` for host-local events that do not enter the UI pump.

Mapped window events:

- `CloseRequested`, `SurfaceResized`, `Moved`, `RedrawRequested`, `Focused`, and `Occluded` become `UiWindowEvent` variants.
- Mouse, unknown pointer, and tablet pointer motion become cursor move/enter/leave window events.
- Mouse and tablet pointer buttons become normalized pointer down/up input events when the button maps to primary, secondary, or middle.
- Keyboard input becomes normalized keyboard input using the caller-provided `UiWindowInputContext` metadata and preserves winit's `is_synthetic` flag.
- IME preedit becomes `UiImeInputEventKind::Preedit`; IME commit becomes text input; IME disabled becomes IME cancel; delete-surrounding becomes `UiImeInputEventKind::DeleteSurrounding` with `UiImeDeleteSurrounding`; enabled is still ignored because it is host state rather than routed text input.
- Mouse wheel line deltas preserve line units; pixel deltas preserve precise pixel scroll and keep the editor baseline legacy scalar fallback.

`translate_winit_modifiers(...)` is separate because modifier state is ambient host state in winit. Callers are expected to update their current input context metadata from winit modifier notifications before translating key or pointer events that should carry those modifiers.

## Touch Model

The runtime adapter follows the winit 0.31 pointer model. The current API does not provide a `WindowEvent::Touch` variant. Touch phases are reconstructed from pointer events:

- touch press: `WindowEvent::PointerButton` with `ButtonSource::Touch` and `ElementState::Pressed`
- touch release: `WindowEvent::PointerButton` with `ButtonSource::Touch` and `ElementState::Released`
- touch move: `WindowEvent::PointerMoved` with `PointerSource::Touch`
- touch cancel: `WindowEvent::PointerLeft` with `PointerKind::Touch`

The resulting `UiWindowPlatformInputEvent::touch_*` values normalize to pointer input with `UiPointerSource::Touch`, stable `UiPointerId`, and the primary button only for started/ended phases. Multi-pointer routing, primary-touch mouse synthesis, and cancel cleanup are intentionally left for later Editor UI 01 M4 slices.

## Design And Rationale

The old retained editor translation path was deleted during M1.S3. New platform semantics must be added to `zircon_runtime::ui::platform_input`, not to editor host-contract modules.

The plan originally expected duplicated input translation under `zircon_runtime/src/rhi/ui_surface.rs` and `zircon_runtime/src/rhi_wgpu/ui_surface.rs`. Those files were checked during this slice and currently contain surface descriptor conversion rather than winit input translation, so there was no rhi input code to migrate or delete.

## Control Flow

1. A platform host creates or updates `UiWindowInputContext` for the native window.
2. The host keeps current modifier state by calling `translate_winit_modifiers(...)` when winit reports modifier changes.
3. For each winit `WindowEvent`, the host calls `translate_winit_window_event(context, event)`.
4. Returned pump events are appended to `UiWindowInputPumpBatch`.
5. Later slices route that batch through the runtime input manager and retire the retained host's remaining mouse-only bridge internals.

## Edge Cases And Constraints

`SurfaceResized` currently produces logical and physical size from the physical winit size with scale factor `1.0`; host-specific DPI policy can refine this later when the window context carries scale information.

`PointerEntered` for touch returns `None`; touch start is represented by touch pointer button press. `PointerLeft` for touch becomes cancel because the current input contract needs a cleanup signal when the pointer leaves without a release.

`Ime::DeleteSurrounding` is now routable through the shared event contract as of Editor UI 01.M1.S4. The event preserves the before/after byte counts but runtime editable text currently records the owner route and leaves actual surrounding-text mutation to a later text-edit policy slice.

## Test Coverage

Module-local tests in `winit_translation.rs` cover keyboard baseline behavior, synthetic key preservation, native scan code preservation, IME preedit/commit/disable behavior, wheel precise delta preservation, touch phase normalization, winit pointer-touch reconstruction, modifiers, and window surface events.

Workspace `cargo check -p zircon_runtime --lib --locked` was attempted during M1.S2, but Cargo refused before Rust diagnostics because the existing workspace `Cargo.toml`/`Cargo.lock` state requires a lockfile update unrelated to this slice. To avoid mutating that state, an external temporary Cargo project was used to compile the local `zircon_runtime` crate with `platform-winit` enabled. That scratch `cargo check` passed, and scratch public API tests passed 2/2.

During M1.S3, `cargo test -p zircon_editor --lib platform_input_translation --locked` hit the same workspace lockfile blocker before Rust diagnostics. A supplemental offline attempt without `--locked` timed out after 600 seconds without diagnostics; the temporary lockfile mutation was restored to the pre-attempt hash.

## Open Issues Or Follow-Up

01.M1.S4 closed the shared delete-surrounding IME carrier and confirmed touch phase carriers already existed. Later input-manager/text slices still own editable delete-surrounding mutation, routing order, active pointer tables, primary-touch mouse synthesis, and timer injection.
