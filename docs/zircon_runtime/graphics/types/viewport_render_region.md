---
related_code:
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/line_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/render_region.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_output_transfer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_fxaa/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_smaa/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
  - zircon_plugins/particles/runtime/src/render/gpu/transparent.rs
implementation_files:
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_extract.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/line_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/render_region.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_output_transfer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_fxaa/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_smaa/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
  - zircon_plugins/particles/runtime/src/render/gpu/transparent.rs
plan_sources:
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
tests:
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_defaults_to_full_target_without_camera_rect
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_clamps_camera_rect_to_target
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_clamps_fully_outside_rect_to_last_in_bounds_pixel
  - zircon_runtime/src/graphics/types/viewport_render_region.rs::tests::viewport_region_reports_local_rect_for_graph_owned_targets
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_primary_surface_split_screen_base_cameras_clear_only_their_viewport_regions
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs::tests::scene_region_clear_resources_build_for_offscreen_backend
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

The region deliberately carries both coordinate spaces. The physical rect is the selected camera's location inside the full renderer-owned target. The local rect is always origin zero with the same clamped size, for graph-owned child textures that were allocated exactly to the selected viewport size.

## Pass Coverage

`ViewportRenderRegion::apply_physical_to_render_pass(...)` sets both `set_viewport(...)` and `set_scissor_rect(...)` in full-target coordinates; `apply_to_render_pass(...)` is the physical compatibility wrapper. `apply_local_to_render_pass(...)` writes the same size at origin zero for graph-owned viewport-sized targets. Graph-raster camera passes now receive the selected frame region through `RenderPassGpuExecutionContext` or `ViewportRenderFrame` and return before drawing if the region is empty.

The current coverage includes the pre-graph scene clear draw, normal prepass, base mesh scene pass, deferred G-buffer, deferred lighting, sprites, CPU particle billboards, particle velocity, TAA reactive mask mesh draws, preview sky, grid/wireframe/selection/gizmo/handle line overlays, and the particles plugin transparent GPU draw path. Plugin direct-test callers use `ViewportRenderRegion::full_target(...)`; production plugin draws receive the frame-derived region through `ParticleGpuTransparentDrawContext`.

Terminal post-process now chooses the coordinate space by target ownership. Output-transfer uses the local region when it writes graph-owned `FINAL_COMPOSITED` for terminal AA input. FXAA and the final SMAA resolve use the physical region when writing imported/full-frame `FINAL_COLOR`; SMAA edge and blend intermediates stay local.

## Boundaries

This slice enforces split-screen and sub-viewport raster isolation for scene clear, graph-raster passes, and terminal post-process writeback. It does not finish custom-target final composite rules and does not fully split temporal history by camera. Present submit and direct runtime-frame submit now share the selected-camera loop and viewport-terminal owner policy. The focused split-screen product guard now passes for two PrimarySurface Base cameras clearing independent left/right viewport regions.

## Validation

The source-contract tests cover the full-target default, partial viewport clamp, depth clamp, the last-in-bounds-pixel behavior for a fully outside viewport origin, and the origin-zero local rect for graph-owned child targets. The 2026-06-19 focused lanes passed `zircon_runtime` `core-min` cargo checks, the original three `viewport_render_region` unit tests, the later five-test `viewport_render_region` rerun, the `scene_region_clear_resources` offscreen WGPU submit test, the particles plugin cargo check, and four `particle_gpu_runtime_owner_` tests including the transparent WGPU draw path. `graphics_primary_surface_split_screen_base_cameras_clear_only_their_viewport_regions` first exposed the physical-scissor-on-local-target bug, then passed after Runtime 05's unrelated `dynamic_scene` retention facade compile blocker cleared: `cargo test -p zircon_runtime --lib graphics_primary_surface_split_screen_base_cameras_clear_only_their_viewport_regions --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-dynamic-scene-asset-0619 --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test with the existing 53-warning set.
