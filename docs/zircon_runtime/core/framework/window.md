---
related_code:
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/mode.rs
  - zircon_runtime/src/core/framework/window/position.rs
  - zircon_runtime/src/core/framework/window/primary_window_handle.rs
  - zircon_runtime/src/core/framework/window/present_mode.rs
  - zircon_runtime/src/core/framework/window/resize_constraints.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/core/framework/window/validation.rs
  - zircon_runtime/src/core/framework/window/tests.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/prelude.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes.rs
implementation_files:
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/mode.rs
  - zircon_runtime/src/core/framework/window/position.rs
  - zircon_runtime/src/core/framework/window/primary_window_handle.rs
  - zircon_runtime/src/core/framework/window/present_mode.rs
  - zircon_runtime/src/core/framework/window/resize_constraints.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/core/framework/window/validation.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - dev/bevy/crates/bevy_window/src/lib.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
tests:
  - zircon_runtime/src/core/framework/window/tests.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes.rs
  - cargo test -p zircon_runtime core::framework::window --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime runtime_prelude_exports_platform_window_and_input_contracts --lib --no-default-features --features core-min --locked
doc_type: module-detail
---

# Runtime Framework Window Contracts

## Purpose

`zircon_runtime::core::framework::window` owns Zircon's neutral runtime window descriptor vocabulary. It is the M3 foundation for Bevy-style window defaults without moving host-specific `winit` ownership out of `zircon_app`.

The module is descriptive. It does not create OS windows, hold raw handles, run an event loop, or present swapchains. Those remain owned by `zircon_app` host code and runtime graphics surfaces. Runtime and editor-facing code can still agree on a primary-window shape through `WindowDescriptor`.

## Reference Evidence

Bevy is the primary reference for this slice:

- `dev/bevy/crates/bevy_window/src/lib.rs` defines `WindowPlugin::primary_window: Option<Window>` and marks the spawned entity as `PrimaryWindow`.
- `dev/bevy/crates/bevy_window/src/window.rs` defines `Window`, `WindowResolution`, `WindowResizeConstraints`, `WindowPosition`, `WindowMode`, and `PresentMode`.
- `dev/bevy/crates/bevy_winit/src/winit_config.rs` keeps event-loop policy separate from the platform-neutral window description.

Zircon deliberately keeps this first window model as framework DTOs instead of Bevy ECS components. The ECS/window-entity path can be added later once scene/runtime scheduling consumes window state directly.

## Data Model

The folder-backed module keeps declaration families separated:

- `PrimaryWindowHandle` is the neutral primary-window identity used before a concrete host handle exists.
- `WindowDescriptor` carries the primary-window handle, title, present mode, mode, position, resolution, resize constraints, and basic host-visible booleans.
- `WindowResolution` tracks physical size, backend scale factor, optional scale-factor override, and logical-size conversion.
- `WindowResizeConstraints` records logical min/max bounds and validates invalid or inverted bounds into a safe window range while preserving unbounded defaults.
- `WindowPosition` covers automatic placement, centered placement, and explicit physical pixel coordinates.
- `WindowMode` covers windowed, borderless fullscreen, and fullscreen intent.
- `WindowPresentMode` mirrors Bevy's `AutoVsync`, `AutoNoVsync`, `Fifo`, `FifoRelaxed`, `Immediate`, and `Mailbox` vocabulary without importing `wgpu` or `winit` types.

`DEFAULT_WINDOW_TITLE` is `Zircon Runtime`. The default descriptor matches the current runtime preview host policy: primary window handle `0`, 1280x720, FIFO present mode, windowed, automatic position, resizable, decorated, visible, and focused.

## Ownership Boundary

The owner is `zircon_runtime::core::framework` because the values are neutral DTOs shared by runtime modules, the app host, and later editor host integration. `zircon_app` remains responsible for translating these DTOs into `winit::window::WindowAttributes` and event-loop behavior. `zircon_runtime::platform` remains responsible for declaring whether a window backend is available for the current target.

The first app-host conversion lives in `zircon_app/src/entry/runtime_entry_app/window_attributes.rs`. It maps the neutral descriptor to winit title, physical surface size, logical resize constraints, visibility, decoration, resizable and focus flags, explicit physical position, and borderless fullscreen intent. Centered placement and exclusive fullscreen are intentionally deferred because they require monitor selection that the neutral DTO does not yet own.

This split keeps Bevy-style `bevy_window` / `bevy_winit` separation: the runtime framework owns the window vocabulary, while the app host owns concrete platform creation.

## Test Coverage

`zircon_runtime/src/core/framework/window/tests.rs` verifies:

- default primary-window policy,
- physical/logical/scale-factor conversion,
- invalid resize-constraint clamping and unbounded defaults,
- builder methods for host-neutral settings,
- structural root-module exports so implementation stays out of `window/mod.rs`.
- app-host conversion from `WindowDescriptor` to winit `WindowAttributes` for default and custom descriptors.

`zircon_runtime/src/tests/prelude.rs` verifies the stable runtime prelude exports the window descriptor vocabulary. Use the fully qualified `core::framework::window` test filter for this module's focused Cargo validation; the broader `window` substring also matches unrelated dynamic/render tests.

Milestone-level Cargo acceptance remains the platform/window/input testing stage from the active Bevy parity plan. The neutral descriptor foundation is now consumed by the runtime preview app for initial winit window attributes; later M3 slices can move from default-only consumption to profile/config-driven window descriptors and fuller event-loop update policy.
