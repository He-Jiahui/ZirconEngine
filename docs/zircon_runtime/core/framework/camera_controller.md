---
related_code:
  - zircon_runtime/src/core/framework/camera_controller/mod.rs
  - zircon_runtime/src/core/framework/camera_controller/common.rs
  - zircon_runtime/src/core/framework/camera_controller/free/mod.rs
  - zircon_runtime/src/core/framework/camera_controller/free/controller.rs
  - zircon_runtime/src/core/framework/camera_controller/free/input.rs
  - zircon_runtime/src/core/framework/camera_controller/free/settings.rs
  - zircon_runtime/src/core/framework/camera_controller/free/state.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/mod.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/controller.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/input.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/settings.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/state.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/mod.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/action.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/controller.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/input.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/settings.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/state.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_navigation.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state_new.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_camera.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_frame_selection.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_reset_from_scene.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs
implementation_files:
  - zircon_runtime/src/core/framework/camera_controller/common.rs
  - zircon_runtime/src/core/framework/camera_controller/free/controller.rs
  - zircon_runtime/src/core/framework/camera_controller/free/input.rs
  - zircon_runtime/src/core/framework/camera_controller/free/settings.rs
  - zircon_runtime/src/core/framework/camera_controller/free/state.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/controller.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/input.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/settings.rs
  - zircon_runtime/src/core/framework/camera_controller/pan/state.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/action.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/controller.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/input.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/settings.rs
  - zircon_runtime/src/core/framework/camera_controller/orbit/state.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_navigation.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_state_new.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_camera.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_frame_selection.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_reset_from_scene.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_selection.rs
plan_sources:
  - user: 2026-05-25 continue Runtime Picking / Gizmos / Camera / Remote Bevy completion plan
  - .codex/plans/runtime-picking-gizmos-camera-remote-bevy-completion-plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/tests/camera_controller.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - cargo test -p zircon_runtime --lib camera_controller --locked --color never --jobs 1
  - cargo test -p zircon_editor --lib viewport_perspective_camera_navigation_uses_runtime_orbit_controller --locked --color never --jobs 1
doc_type: module-detail
---

# Runtime Camera Controller Framework Contracts

## Purpose

`zircon_runtime::core::framework::camera_controller` is the M5 runtime-owned camera navigation contract layer. It gives runtime demos, editor viewport adapters, dynamic API frames, and later remote/dev tools one reusable implementation for free, pan, and orbit camera behavior.

The module is not an input system and not scene ownership. It accepts neutral input DTOs, updates controller state, and returns a `CameraControllerOutput` with the next `Transform`, transform deltas, and optional cursor grab intent. Callers remain responsible for mapping keyboard, mouse, touch, or remote commands into input DTOs and for writing the result back to their owning world or viewport.

## Reference Evidence

The primary reference is Bevy camera controller source:

- `dev/bevy/crates/bevy_camera_controller/src/free_camera.rs` separates `FreeCamera` settings from `FreeCameraState` and the controller system, applies scroll speed exponentially, decays velocity with friction, and updates yaw/pitch from pointer or touch deltas.
- `dev/bevy/crates/bevy_camera_controller/src/pan_camera.rs` keeps 2D pan/zoom settings in a separate controller and treats drag panning as a viewport-to-world style translation rather than editor-specific pointer routing.

Fyrox is the editor split cross-check:

- `dev/Fyrox/editor/src/camera/mod.rs` stores editor camera yaw, pitch, offset, movement flags, speed factor, and mouse interaction mode in a controller rather than scattering those values across viewport UI code.

Zircon intentionally diverges from Bevy by not modeling these controllers as ECS components yet. The framework layer keeps the same settings/state/input/update separation, but exposes plain Rust structs so `zircon_runtime`, `zircon_editor`, dynamic API code, and future remote tools can share the behavior before a full scheduling integration lands.

## Ownership Boundary

The framework module owns:

- `CameraControllerOutput`, `CursorGrabIntent`, and `CursorGrabMode` as shared controller results.
- `FreeCameraController`, `FreeCameraSettings`, `FreeCameraState`, and `FreeCameraInput` for fly/free navigation.
- `PanCameraController`, `PanCameraSettings`, `PanCameraState`, and `PanCameraInput` for 2D pan, drag pan, zoom, and roll.
- `OrbitCameraController`, `OrbitCameraSettings`, `OrbitCameraState`, `OrbitCameraInput`, and `OrbitCameraAction` for orbit, pan, zoom, and focus target changes.

The framework module does not own window focus, raw device events, editor selection, undo, scene serialization, or runtime world authority. Dynamic API and editor code should adapt their local inputs into these DTOs, apply the returned transform through their own scene or viewport authority, and avoid adding new private camera algorithms.

## Data Model

The module is folder-backed to keep root wiring structural:

- `common.rs` defines output deltas and cursor grab intent.
- `free/*` defines free-camera settings, state, input, and update behavior.
- `pan/*` defines 2D pan/zoom settings, state, input, and update behavior.
- `orbit/*` defines orbit actions, settings, state, input, and update behavior.
- `mod.rs` only declares children and re-exports the public surface.

Each controller stores its settings and state. Inputs are frame-like values produced by a caller: elapsed time, movement axes, drag deltas, scroll amounts, focus gates, viewport size, and cursor-grab state where relevant. The update step returns a full transform plus deltas so callers can either write the transform directly or inspect the movement that occurred.

## Behavior

`FreeCameraController` follows Bevy's free camera core: movement axes are interpreted in camera right, world up, and camera forward space; scroll adjusts a speed multiplier exponentially; no-input velocity decays by friction; pitch/yaw are clamped and applied from look deltas; cursor grab changes are reported as intent instead of directly mutating a window.

`PanCameraController` supports keyboard-style pan axes, drag panning scaled by viewport extent and zoom factor, roll around local Z, and zoom clamped between configured minimum and maximum values. It uses neutral math rather than a renderer-specific viewport-to-world API, so future editor integration can substitute a more exact projection adapter without changing the controller contract.

`OrbitCameraController` preserves the existing dynamic API orbit behavior as public runtime logic: orbit keeps distance around a target, pan translates both camera and target in the camera plane, zoom moves along camera forward while respecting a minimum target distance, and focus changes update target state without moving the camera.

## Dynamic API Migration

`zircon_runtime/src/dynamic_api/camera_controller.rs` now keeps only viewport size, drag state, and an `OrbitCameraController`. Right and middle mouse drags plus scroll are adapted into `OrbitCameraInput`, and the returned transform is written back to the active runtime scene camera. This prevents the dynamic API from growing a second private orbit/pan/zoom algorithm while preserving its existing exported frame behavior.

## Editor Viewport Adapter

`zircon_editor/src/scene/viewport/controller` now stores an `OrbitCameraController` beside its editor viewport camera snapshot. Selection, frame-selection, reset, and view alignment update the controller target whenever they update `orbit_target`, so the editor does not keep two independent orbit targets.

Perspective editor navigation now adapts right-drag, middle-drag, and scroll into `OrbitCameraInput::orbit`, `OrbitCameraInput::pan`, and `OrbitCameraInput::zoom`. The returned transform is applied to the editor-owned `ViewportCameraSnapshot`, preserving editor ownership of viewport state while sharing the runtime math.

Orthographic pan and zoom remain editor-local in this slice because they depend on `ViewportCameraSnapshot::ortho_size` and screen-height scaling rather than orbit target distance. That divergence is intentional and should be revisited when the runtime camera controller grows a first-class orthographic/pan projection adapter.

## Test Coverage

`zircon_runtime/src/tests/camera_controller.rs` covers:

- Free camera forward movement, pitch clamp, and cursor grab intent.
- Free camera scroll speed multiplier and friction decay.
- Pan camera translation, rotation, zoom scale, and zoom clamping.
- Orbit camera distance preservation, target panning, and zoom toward target.

`zircon_runtime/src/dynamic_api/camera_controller.rs` includes a module-local smoke test proving the old dynamic runtime controller still moves the active scene camera closer to its orbit target through the public `OrbitCameraController`.

`zircon_editor/src/tests/editing/viewport.rs` includes a source guard that freezes the editor perspective navigation cutover: `scene_viewport_controller_navigation.rs` must call the runtime `OrbitCameraInput` constructors and must not restore the old local yaw/pitch or perspective pan constants.
