---
related_code:
  - zircon_runtime_interface/src/ui/window/mod.rs
  - zircon_runtime_interface/src/ui/window/metadata.rs
  - zircon_runtime_interface/src/ui/window/metrics.rs
  - zircon_runtime_interface/src/ui/window/impact.rs
  - zircon_runtime_interface/src/ui/window/event.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/metadata.rs
  - zircon_runtime_interface/src/tests/window_input_contracts.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/utils/mod.rs
  - dev/godot/servers/display/display_server.cpp
implementation_files:
  - zircon_runtime_interface/src/ui/window/mod.rs
  - zircon_runtime_interface/src/ui/window/metadata.rs
  - zircon_runtime_interface/src/ui/window/metrics.rs
  - zircon_runtime_interface/src/ui/window/impact.rs
  - zircon_runtime_interface/src/ui/window/event.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
  - zircon_runtime_interface/src/ui/mod.rs
plan_sources:
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - user: 2026-05-08 continue M1 window/input pump convergence interface slice
tests:
  - zircon_runtime_interface/src/tests/window_input_contracts.rs
  - 2026-05-08: cargo test -p zircon_runtime_interface --lib window_input_contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-window-input-m1 --message-format short --color never (2 passed; 0 failed; 68 filtered out; existing sibling `ui_contract_spine` unused-import warning)
  - 2026-05-08: cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-window-input-m1 --message-format short --color never (passed)
  - 2026-05-08: rustfmt --edition 2021 --check touched M1 window/input files (passed)
  - 2026-05-08: git diff --check -- touched M1 window/input files and docs (passed with CRLF conversion warnings only)
doc_type: module-detail
---

# UI Window Input Contracts

`zircon_runtime_interface::ui::window` owns the neutral M1 window-event and input-pump DTO vocabulary. It is an interface-only subtree: runtime winit, editor native/Slint, and future app-host converters may emit these records, but platform event-loop behavior remains outside `zircon_runtime_interface`.

## Reference Anchors

Bevy is the dominant reference for this slice. `dev/bevy/crates/bevy_window/src/event.rs` models window events as platform-neutral messages for cursor moved/entered/left, focus, resize, scale factor, redraw, close, lifecycle, file drag/drop, theme, mouse, keyboard, touch, and IME traffic. `dev/bevy/crates/bevy_winit/src/state.rs` converts winit cursor movement into logical coordinates plus optional delta, clears cached cursor position on cursor leave, emits focus/IME/resize/move events, and treats redraw requests as a separate event-loop concern.

Fyrox is the Rust-native editor/runtime cross-check. `dev/Fyrox/editor/src/lib.rs` handles resize, focus, move, scale factor, and redraw at the editor window boundary, then translates applicable winit events into UI OS events through `dev/Fyrox/fyrox-impl/src/utils/mod.rs`.

Godot is the UI/editor platform vocabulary cross-check. `dev/godot/servers/display/display_server.cpp` binds window event constants for mouse enter/exit, focus in/out, close request, DPI change, force close, and output color metadata. Zircon keeps Rust DTO names and does not mirror Godot's C++ enum names directly.

## Module Shape

`metadata.rs` defines `UiWindowEventMetadata`, which carries a shared `UiWindowId`, timestamp, sequence, and synthetic flag. It deliberately reuses the input metadata clock and sequence types so window events and `UiInputEvent` records can be ordered in one pump stream.

`metrics.rs` defines `UiWindowMetrics`, `UiWindowPixelSize`, and `UiWindowPixelPosition`. Logical size uses existing `UiSize`; physical size and position stay integer pixel DTOs. `scale_factor` defaults to `1.0` for old or partial payloads.

`event.rs` defines `UiWindowEvent`, `UiWindowEventKind`, and `UiWindowRedrawReason`. Current M1 kinds cover created, close requested, closed, destroyed, cursor moved/entered/left, focus, occlusion, resize, scale-factor changes, backend scale-factor changes, move, and redraw request. IME and text remain represented by the existing shared `UiInputEvent` families instead of duplicating text-input payloads in the window module.

`impact.rs` defines `UiWindowEventImpact`, a declarative dirty/consequence summary. It encodes the M1 acceptance semantics directly: cursor leave clears hover and requests redraw, scale-factor changes mark layout metrics dirty without mutating input state or forcing redraw by themselves, resize marks layout metrics dirty and redraws, and close requests are distinguishable from closed/destroyed cleanup events.

`pump.rs` defines `UiWindowInputPumpEvent` and `UiWindowInputPumpBatch`. A pump event is either a window event or an existing shared `UiInputEvent`; this prevents runtime/editor hosts from inventing a second input vocabulary. `push_coalesced(...)` drops only consecutive redraw requests so repeated redraw notifications do not flood later M1 consumers, while preserving cursor/input ordering around redraws.

## Boundary

This module does not translate winit, Slint, Godot, or Fyrox events. It does not clear hover, dirty layout, request platform redraw, close windows, or mutate runtime/editor state. Those behaviors belong to later runtime and editor M1 slices after active platform/window/input sessions settle.

The contract tests in `zircon_runtime_interface/src/tests/window_input_contracts.rs` prove the DTOs are constructible, serializable, and carry the intended impact/coalescing semantics. The 2026-05-08 scoped interface gate passed for the focused contract tests and crate check. It did not claim workspace-wide validation or runtime/editor host conversion.
