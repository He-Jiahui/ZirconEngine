---
related_code:
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/line_pass.rs
  - zircon_plugins/particles/runtime/src/render/gpu/transparent.rs
implementation_files:
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/line_pass.rs
  - zircon_plugins/particles/runtime/src/render/gpu/transparent.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
tests:
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_defaults_to_full_target_without_camera_rect
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_clamps_camera_rect_to_target
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_clamps_fully_outside_rect_to_last_in_bounds_pixel
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619 --message-format short --color never
  - cargo test -p zircon_runtime --lib viewport_render_region --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619 --message-format short --color never -- --nocapture
  - cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619-particles --message-format short --color never
  - cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-viewport-region-0619-particles --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Viewport Render Region

## Purpose

`viewport_render_region.rs` is the Plan 09 bridge from a selected `CameraRenderDescriptor.viewport_rect` to WGPU viewport and scissor state. `ViewportRenderFrame` owns the derived `ViewportRenderRegion`, so render passes consume a renderer-side DTO instead of reaching back into neutral extract data or duplicating clamp rules.

When no selected camera viewport exists, the region covers the full physical target. When one exists, it uses `RenderViewportRect::clamped_to_size(...)` against the current target size and clamps depth to `[0.0, 1.0]`. The clamp follows the framework viewport contract: an origin beyond a nonzero target is clamped to the last in-bounds pixel with a 1x1 region rather than becoming empty.

## Pass Coverage

`ViewportRenderRegion::apply_to_render_pass(...)` sets both `set_viewport(...)` and `set_scissor_rect(...)`. Graph-raster camera passes now receive the selected frame region through `RenderPassGpuExecutionContext` or `ViewportRenderFrame` and return before drawing if the region is empty.

The current coverage includes normal prepass, base mesh scene pass, deferred G-buffer, deferred lighting, sprites, CPU particle billboards, particle velocity, TAA reactive mask mesh draws, preview sky, grid/wireframe/selection/gizmo/handle line overlays, and the particles plugin transparent GPU draw path. Plugin direct-test callers use `ViewportRenderRegion::full_target(...)`; production plugin draws receive the frame-derived region through `ParticleGpuTransparentDrawContext`.

## Boundaries

This slice enforces split-screen and sub-viewport raster isolation for graph-raster passes only. It does not finish custom-target final composite rules, does not split fullscreen post-process or temporal history by camera, and does not extend generated multi-camera submit to present submit or direct runtime-frame submit. Pixel/product and RenderDoc evidence for `render_product_split_screen_viewports` remain open Plan 09 acceptance work.

## Validation

The source-contract tests cover the full-target default, partial viewport clamp, depth clamp, and the last-in-bounds-pixel behavior for a fully outside viewport origin. The 2026-06-19 focused lane passed `zircon_runtime` `core-min` cargo check, the three `viewport_render_region` unit tests, the particles plugin cargo check, and four `particle_gpu_runtime_owner_` tests including the transparent WGPU draw path.
