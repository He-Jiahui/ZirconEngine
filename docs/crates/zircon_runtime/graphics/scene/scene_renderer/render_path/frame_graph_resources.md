---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/render_graph/graph.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/types/viewport_render_region.rs
  - zircon_runtime/src/graphics/types/viewport_camera_stack_attachment_policy.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear/scene_region_clear_resources.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_imports_only_live_compiled_frame_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_reuses_fixed_scene_color_and_depth_targets
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_rebinds_live_final_aliases_to_imported_texture_target
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_leaves_advanced_transients_for_materialization
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_declares_uber_light_list_as_external_when_clustered_lighting_is_disabled
  - cargo test -p zircon_runtime --lib frame_binder_reuses_fixed_scene_color_and_depth_targets --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-camera-record-owner-0618 --message-format short --color never -- --nocapture
doc_type: module-detail
---

# Frame Graph Resources

`bind_frame_graph_resources.rs` owns renderer-owned frame resource binding for the compiled-scene render path. These resources are not allocated by transient graph materialization: they are fixed `OffscreenTarget` views, the optional imported final target, the renderer-owned light-list buffer, and the persistent shadow atlas view.

The binder receives the compiled graph and imports only resources that have a live `CompiledRenderGraph::resource_lifetime_by_name(...)` row. This replaces the old unconditional frame-target import behavior with graph-lifetime-aware actual binding while preserving the renderer-owned backing model for fixed frame targets.

Plan 09 Base/Overlay camera submits rely on this fixed backing model. Each selected-camera child re-enters graph execution with the same sized `OffscreenTarget`, and live `SCENE_COLOR` / `SCENE_DEPTH` rows bind to the fixed target views instead of graph-owned transient textures. `ViewportCameraStackAttachmentPolicy` stores the selected camera clear plan and converts graph-declared first scene attachment clears to loads; `SceneRegionClearResources` then applies the requested color/depth clear as a pre-graph draw clipped by the selected `ViewportRenderRegion`. That keeps Overlay and split-screen children from clearing the whole shared scene target while still executing as separate selected-camera graph submits. `ViewportRenderRegion` is the matching raster state policy: it does not allocate or bind resources, but it keeps graph-raster draws clipped to the selected camera's physical viewport/scissor region on that shared target.

The region-clear color uniform follows the graph/frame transaction boundary. Recording returns one immutable 16-byte buffer upload only when color is present; depth-only, no-clear, and empty-region paths return an empty batch. `RenderGraphStageExecution` retains that upload until every graph stage succeeds, after which the outer frame owner merges it into the single `FrameBufferUpload` admission before submitting the recorded clear draw. The clear resource does not receive `wgpu::Queue`, and graph failure drops both the unsubmitted commands and the unaccepted parameter update.

## Bound Resources

The current binding set covers:

- `SCENE_COLOR` and `SCENE_DEPTH`.
- final target aliases: `FINAL_COLOR`, `VIEWPORT_OUTPUT`, `FINAL_COMPOSITED`, `COLOR_GRADED`, and `EFFECT_STACKED`.
- G-buffer and lighting frame views: `GBUFFER_ALBEDO`, `GBUFFER_NORMAL`, `GBUFFER_MATERIAL`, `AMBIENT_OCCLUSION`, `GLOBAL_ILLUMINATION`, and `BLOOM`.
- renderer-owned buffers/textures: `LIGHT_LIST` and `SHADOW_ATLAS`.

When a direct imported final target is supplied, every live final alias binds to that borrowed target view. Otherwise the alias points at the `OffscreenTarget::final_color` backing. `SHADOW_ATLAS` binds to `ShadowAtlasResources::atlas_view()` only when shadow atlas resources are available and the compiled graph retains that lifetime.

`LIGHT_LIST` is conditional at graph level. If clustered lighting is live, `light-grid-build` writes a graph-owned buffer and `post.uber` reads that buffer. If clustered lighting is disabled but post-process still needs the cluster bind group, descriptor filtering keeps `LIGHT_LIST` live as an External buffer and this frame binder imports the `OffscreenTarget::cluster_buffer` default backing.

## Boundaries

This module does not allocate graph transients and does not pre-bind advanced post-process products such as scene velocity, motion-vector tiles, DoF intermediates, color LUT, HZB furthest, or SSR pyramid resources. Those remain graph-owned and are backed by transient materialization when their compiled lifetimes are live.

The module also does not bind plugin-owned external buffers. First-party plugin buffers are handled by `bind_plugin_graph_resources.rs`, which currently records graph-lifetime-aware fallback backings until real plugin WGPU buffer owners are exposed to the scene renderer. Region-scoped scene clear and fullscreen post-process viewport policy remain outside this frame-resource binder; final composite viewport policy is still Plan 09 follow-up work.

## Validation State

The source-contract tests cover live-only frame binding, fixed scene color/depth target reuse, imported final-target alias rebinding, leaving advanced post-process transient resources unbound until graph materialization, and keeping `LIGHT_LIST` external when clustered lighting is disabled. The 2026-06-18 focused `light_list` filter passed in the warmed HZB product lane; the Plan 09 physical-target regression `frame_binder_reuses_fixed_scene_color_and_depth_targets` passed in `D:\cargo-targets\zircon-runtime-camera-record-owner-0618`; scoped `zircon_runtime --features core-min` cargo checks provide the compile gate for this slice.
