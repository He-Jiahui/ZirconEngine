---
related_code:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_core_pipeline/src/schedule.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
plan_sources:
  - user: 2026-05-16 continue ZirconEngine Bevy-level rendering completion plan M2A/M2B
  - docs/superpowers/plans/2026-05-16-render-camera-target-routing-m2c.md
  - docs/superpowers/plans/2026-05-17-render-camera-ordering-m2d.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
tests:
  - zircon_runtime/src/core/framework/tests.rs::render_camera_contracts_cover_viewports_and_bevy_layer_intersection
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_filters_meshes_by_active_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::explicit_render_camera_snapshot_layers_override_scene_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_projects_scene_camera_component_product_fields
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::inactive_render_camera_extracts_no_scene_renderables
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_camera_product_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_defaults_bevy_camera_fields_when_omitted
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_reports_unsupported_without_primary_fallback_capture
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_present_reports_unsupported_surface_fallback
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras
  - zircon_runtime/src/graphics/tests/visibility.rs
doc_type: module-detail
---

# Runtime Render Camera Contracts

## Purpose

`zircon_runtime::core::framework::render::camera` owns the neutral camera data surface used by render extraction and graphics backends. M2A expands the earlier viewport-only snapshot into a Bevy-informed camera contract without moving concrete renderer execution out of `zircon_runtime::graphics`.

The contract is still data-oriented: it describes projection, target, viewport, render order, active state, HDR, exposure, clear color, MSAA sample count, and render layers. Scene and editor systems project their local state into this contract before render graph or RHI code consumes it.

## Bevy Evidence

The M2A shape follows four local Bevy source areas:

- `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs` treats rendering as camera-driven and chooses 2D/3D core schedules from camera setup.
- `dev/bevy/crates/bevy_camera/src/components.rs` defines `Camera2d`, `Camera3d`, and `Hdr` as explicit camera-side product signals.
- `dev/bevy/crates/bevy_camera/src/camera.rs` models camera viewport rectangles, render targets, order, active state, output mode, clear color, and target size calculations.
- `dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs` makes render layer intersection a first-class rule: default entities and cameras are on layer `0`, and an empty layer set is invisible.
- `dev/bevy/crates/bevy_render/src/camera.rs` removes extracted camera components when `Camera::is_active` is false, and `dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs` plus `core_3d/mod.rs` skip phase preparation for inactive cameras.

Zircon keeps the same product semantics but does not copy Bevy ECS components one-for-one. The stable boundary is `ViewportCameraSnapshot`, because current runtime render extraction already passes that snapshot through `RenderViewExtract`, `RenderFrameExtract`, picking, visibility, and renderer preparation.

## Data Model

`ViewportCameraSnapshot` now includes:

- projection data: `projection_mode`, `fov_y_radians`, `ortho_size`, `z_near`, `z_far`, and `aspect_ratio`;
- output data: `target`, optional `viewport`, `order`, `is_active`, `clear_color`, and `msaa_samples`;
- imaging data: `hdr` and `exposure_ev100`;
- visibility data: `render_layers`.

`RenderViewportRect` stores physical position, physical size, and normalized depth range. `clamped_to_size(...)` mirrors Bevy's viewport containment rule by keeping the rectangle inside the target size before the camera recomputes aspect ratio.

`RenderCameraTarget` currently distinguishes the primary surface, texture targets, and headless targets. Backends can map those variants to native windows, offscreen textures, or no-color-output paths later without making framework consumers depend on WGPU.

`RenderCameraClearColor` separates default clear policy, no clear, and explicit color.

## Render Layers

`RenderLayerSet` is the Bevy-style layer contract. Its default value is layer `0`; `RenderLayerSet::none()` belongs to no layers and does not intersect anything, including another empty set.

The current scene component still stores a legacy `u32` mask, so M2A provides `RenderLayerSet::from_legacy_mask(...)`, `to_legacy_mask_lossy(...)`, and `intersects_legacy_mask(...)`. This lets scene extraction enforce Bevy-style visibility immediately while leaving the broader scene serialization migration for a later deliberate cutover.

During scene render extraction, `World::build_render_camera(...)` projects the active camera entity's `RenderLayerMask` into `ViewportCameraSnapshot::render_layers`. Mesh extraction then filters mesh snapshots against the camera layer set before building `GeometryExtract`, phase inputs, and visibility input. Explicit `SceneViewportExtractRequest::camera` snapshots keep their own layer set and can override the scene camera.

When `ViewportCameraSnapshot::is_active` is false, scene extraction keeps the camera data available for diagnostics and editor state but emits no scene meshes, phase inputs, visibility renderables, or scene lights. This mirrors Bevy's inactive-camera path within Zircon's current single-camera extract DTOs.

## Scene And Asset Projection

M2B moves the product fields down into `CameraComponent` and `SceneCameraAsset`. Scene cameras now carry projection mode, orthographic size, target, viewport, render order, active state, HDR, exposure, clear color, and MSAA sample count. Serde defaults preserve older scene and project documents that only stored `fov_y_radians`, `z_near`, and `z_far`.

`SceneCameraTargetAsset` uses asset references for texture targets and contributes those texture references to `SceneAsset::direct_references()`. `World::from_scene_asset(...)` resolves texture targets into `RenderCameraTarget::Texture`, while `World::to_scene_asset(...)` writes component camera targets back to scene asset form. Headless camera targets round-trip through explicit physical sizes and can drive aspect-ratio calculation when no viewport size is supplied by the request.

## Concrete Target Routing

M2C starts concrete target routing at the graphics submission boundary. `RenderCameraTarget::PrimarySurface` still uses the active viewport record size. `RenderCameraTarget::Headless { size }` now resolves the submission size before visibility, history validation, runtime-frame construction, offscreen target allocation, and capture. The renderer clamps zero axes to at least one pixel through the same target-size path used by viewport records.

`RenderCameraTarget::Texture(_)` intentionally returns `RenderFrameworkError::UnsupportedCapability { capability: "camera texture render target" }` for now. This matches Bevy's missing render-target behavior: image and texture-view targets do not silently render into a different target when the target resource is unavailable. Zircon keeps the real texture writeback for the asset/GPU texture residency milestone instead of coupling it to the camera slice.

Surface presentation is primary-surface-only in M2C. A headless camera submitted through `present_frame_extract` reports `UnsupportedCapability { capability: "headless camera surface present" }`, so callers cannot accidentally blit a headless offscreen target into a bound surface.

M2C validation used `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c` on 2026-05-16. `cargo test -p zircon_runtime camera_target --locked --jobs 1 --message-format short --color never` passed the three focused target-routing tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward.

## Camera Ordering

M2D adds the neutral ordering contract needed before Zircon expands from single-camera extracts into split-screen and multi-target camera schedules. `sort_render_cameras(...)` accepts camera snapshots paired with entity ids and returns active cameras sorted by render order, normalized target key, and deterministic entity tie-break.

The behavior follows Bevy's render-app `sort_cameras` path in `dev/bevy/crates/bevy_render/src/camera.rs:663-722`: active cameras are sorted by `(order, target)`, duplicate active `(order, target)` groups are reported as ambiguities, and each camera receives a `sorted_camera_index_for_target` counted per `(target, hdr)`. Inactive cameras are skipped because Bevy removes inactive cameras from extraction before sorting.

`RenderCameraTargetOrderKey` normalizes Zircon targets without depending on concrete WGPU objects: the primary surface is a single key, texture targets use the stable `ResourceId`, and headless targets use their physical size. This keeps the contract usable by later graphics and editor viewport scheduling while true texture residency remains owned by the asset/GPU resource lane.

M2D validation used WSL/Linux with `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-render-camera-m2d-wsl` on 2026-05-17. `cargo test -p zircon_runtime --lib render_camera_ordering --locked --jobs 1 --message-format short --color never` passed the two focused ordering tests, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed afterward with only existing unused-function warnings outside this module. Windows validation currently fails earlier in `wgpu-hal 29.0.3` DX12 dependency compilation, before Zircon source is checked.

## Scope Boundary

M2A/M2B/M2C/M2D still leave editor authoring for the new fields, true texture-target writeback, and the later hard cutover from scene `RenderLayerMask(u32)` to `RenderLayerSet` for separate milestones. Texture writeback should resume only after the asset/image/texture residency work provides a stable GPU resource owner.

The M2C entry gate was captured on 2026-05-16 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-render-camera-m2-1819`: `cargo test -p zircon_runtime camera --locked --jobs 1 --message-format short --color never` passed 13 focused camera/layer/scene-asset tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward.
