---
related_code:
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_core_pipeline/src/schedule.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs
  - dev/bevy/crates/bevy_camera/src/components.rs
  - dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs
  - dev/bevy/crates/bevy_light/src/lib.rs
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/bevy/crates/bevy_sprite_render/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - dev/bevy/crates/bevy_post_process/src/lib.rs
  - dev/bevy/crates/bevy_anti_alias/src/lib.rs
  - dev/bevy/crates/bevy_solari/src/lib.rs
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/target_resolution.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
implementation_files:
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/assets-and-rendering/render-framework-architecture.md
  - docs/zircon_runtime/core/framework/render/profile.md
  - docs/zircon_runtime/core/framework/render/camera.md
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/render2d/mod.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/scene/components/render2d/mesh2d.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources/scene_post_process_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_ssao/execute_ssao.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - user: 2026-05-08 implement ZirconEngine Bevy-Level Rendering Completion Plan M0
  - user: 2026-05-08 continue ZirconEngine Bevy-Level Rendering Completion Plan M1
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - docs/superpowers/plans/2026-05-17-render-camera-ordering-m2d.md
tests:
  - "M0 docs acceptance only: no runtime tests required by plan"
  - cargo test -p zircon_runtime render_profile --locked
  - cargo check -p zircon_app --locked --all-targets
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime --locked render_product_pbr
  - cargo test -p zircon_runtime --locked material
  - cargo check -p zircon_runtime --lib --locked
  - tests/acceptance/render-product-m5a-pbr-light.md
  - tests/acceptance/render-product-m6a-sprite-default-2d.md
  - zircon_runtime/src/core/framework/tests.rs::render_camera_contracts_cover_viewports_and_bevy_layer_intersection
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_filters_meshes_by_active_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::explicit_render_camera_snapshot_layers_override_scene_camera_layers
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::render_extract_projects_scene_camera_component_product_fields
  - zircon_runtime/src/scene/tests/ecs_schedule.rs::inactive_render_camera_extracts_no_scene_renderables
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_camera_product_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_roundtrip_preserves_bevy_style_camera_fields
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_camera_asset_defaults_bevy_camera_fields_when_omitted
  - zircon_runtime/src/graphics/tests/visibility.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_texture_reports_unsupported_without_primary_fallback_capture
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_present_reports_unsupported_surface_fallback
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index
  - zircon_runtime/src/core/framework/tests.rs::render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_world_frame_extract_exposes_runtime_sprite_components
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_world_frame_extract_filters_by_camera_layers
  - zircon_runtime/src/scene/tests/world_basics.rs::render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_core2d_pipeline_compiles_expected_stage_order_and_passes
  - cargo test -p zircon_runtime --locked render_product_sprite
  - cargo test -p zircon_runtime --locked render_product_pipeline
  - cargo test -p zircon_runtime --locked default_core2d_pipeline_compiles_expected_stage_order_and_passes
doc_type: milestone-detail
---

# Bevy Rendering Capability Matrix

## Scope

This is M0 acceptance evidence for `ZirconEngine Bevy-Level Rendering Completion Plan`. It maps local `dev/bevy` rendering feature collections and source modules to the current Zircon owners before M1 starts adding product profiles or code contracts.

The reference engine is the checked-in `dev/bevy` tree, target Bevy `0.19.0-dev` at commit `c040d7603`. Zircon intentionally maps Bevy Cargo features to runtime product profiles instead of copying Cargo-feature activation as the product surface.

## Ownership Rule

Neutral render contracts land under `zircon_runtime::core::framework::render`. Concrete rendering, resource preparation, render graph execution, WGPU, visibility, pipeline compilation, post-process, and runtime provider work stay under `zircon_runtime::graphics`, `zircon_runtime::rhi`, `zircon_runtime::rhi_wgpu`, and `zircon_runtime::render_graph`. Scene, asset, and runtime UI authoring data stay in their existing runtime owners and are projected into render contracts.

Advanced Virtual Geometry and Hybrid GI remain explicit advanced capability/profile paths through `zircon_plugins/virtual_geometry` and `zircon_plugins/hybrid_gi`. They must not become dependencies for default 2D, 3D, or UI rendering.

## Bevy Profile Mapping

| Bevy collection or profile | Bevy source evidence | Zircon product profile | Zircon owner modules | M0 status |
| --- | --- | --- | --- | --- |
| `default` | `dev/bevy/Cargo.toml:133-135`, `dev/bevy/docs/cargo_features.md:22-30` combine `2d`, `3d`, `ui`, and `audio`. | `DefaultRender` for rendering scope only. Audio is out of this plan. | `zircon_app` profile selection plus `zircon_runtime::core::framework::render` bundles. | M1 defines `DefaultRender` as common 2D, 3D, and UI without Solari, VG, or HGI. M3A, M4A/M4B, M5A, and M6A now cover asset readiness, Core2d/Core3d phases, postprocess, runtime PBR/light baseline, and non-particle Core2d sprites; AA and explicit UI pass placement remain later default-profile gaps. |
| `common_api` | `dev/bevy/Cargo.toml:198-211` includes camera, image, mesh, shader, material, text, color, and HDR without `bevy_render`. | `CommonRenderApi`. | `zircon_runtime/src/core/framework/render/*`, `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/*`. | Partially present as viewport camera, scene components, texture/model/shader/material assets, and render extract DTOs. |
| `2d_api` | `dev/bevy/Cargo.toml:213-215` adds `bevy_sprite` without a renderer. | `Render2d` API slice. | `render::{sprite,mesh,material,camera,core_pipeline}` plus current asset and scene owners. | M6A adds first-class `Sprite2dComponent`, `Mesh2dComponent`, `RenderSpriteSnapshot`, and `SpriteExtract`. `Mesh2dComponent` is stored as 2D scene data but is not yet a mesh-rendered product path. |
| `2d_bevy_render` | `dev/bevy/Cargo.toml:216-224` adds `bevy_render`, `bevy_core_pipeline`, `bevy_post_process`, and `bevy_sprite_render`. | `Render2d` implementation bundle. | `zircon_runtime::graphics`, `zircon_runtime::render_graph`, `zircon_runtime::rhi`, `core_pipeline`, and `sprite` contracts. | M4A/M6A now provide camera-selected Core2d phases, the default Core2d pipeline asset, sprite graph passes, sprite texture fallback stats, and concrete sprite quad drawing. UI placement and AA remain later default-render work. |
| `3d_api` | `dev/bevy/Cargo.toml:226-237` adds light, KTX2, morph, SMAA and tonemapping support around common API. | `Render3d` API slice. | `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/material.rs`, `zircon_runtime/src/asset/assets/model.rs`, future `render::{light,pbr,mesh,material}`. | Partially present as `CameraComponent`, `MeshRenderer`, directional/point/spot lights, `MaterialAsset`, and `ModelAsset`. |
| `3d_bevy_render` | `dev/bevy/Cargo.toml:239-250` adds `bevy_render`, `bevy_core_pipeline`, `bevy_anti_alias`, `bevy_pbr`, and `bevy_post_process`. | `Render3d` implementation bundle. | `zircon_runtime/src/graphics/pipeline/*`, `zircon_runtime/src/graphics/scene/scene_renderer/*`, future `core_pipeline`, `pbr`, `anti_alias`, and `post_process` contracts. | Current closest paths are `RenderPipelineAsset::default_forward_plus()` and `default_deferred()`, but they are not yet profile-selected Bevy-style `Core3d`. |
| `ui_api` | `dev/bevy/Cargo.toml:252-253` combines `default_app`, `common_api`, and `bevy_ui`. | `Ui` API slice. | `zircon_runtime/src/ui/*`, `zircon_runtime_interface/src/ui/surface/*`, `zircon_runtime/src/asset/assets/ui.rs`. | Existing runtime UI asset/layout/input/render-extract chain exists, but it is not yet a render product profile. |
| `ui_bevy_render` | `dev/bevy/Cargo.toml:255-261` adds `bevy_render`, `bevy_core_pipeline`, and `bevy_ui_render`. | `Ui` implementation bundle. | `zircon_runtime/src/graphics/scene/scene_renderer/ui/*` plus future `render::ui_render`. | Current UI renderer is screen-space and integrated into compiled scene rendering. M7 must make per-camera UI target placement explicit. |
| `bevy_solari` / `SolariPlugins` | `dev/bevy/crates/bevy_solari/src/lib.rs:39-58` adds raytracing scene and Solari lighting with ray query and binding-array requirements. | `SolariExperimental`. | Future `zircon_runtime::core::framework::render::solari` plus advanced `zircon_runtime::graphics` capability checks. | No Zircon Solari owner exists yet. Hybrid GI and raytracing-related capability flags are the closest current advanced path. |

## Capability Matrix

| Product capability | Bevy evidence | Current Zircon landing module | Target Zircon contract owner | Gap before later milestones |
| --- | --- | --- | --- | --- |
| Render sub-app and render stages | `dev/bevy/crates/bevy_render/src/lib.rs:120-208` defines a render sub-app and `RenderSystems`. | `zircon_runtime/src/graphics/runtime/render_framework/*`, `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/*`, `zircon_runtime/src/render_graph/*`. | `render::core_pipeline` for neutral schedule/phase names, concrete execution in `zircon_runtime::graphics`. | Zircon has compiled graph stages and render framework submit paths, but no Bevy-like product profile bundle or `RenderSystems` equivalent contract. |
| Camera-driven core pipeline | `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:1-11` states rendering is camera driven, and `camera_driver` starts at `schedule.rs:119`; `dev/bevy/crates/bevy_render/src/camera.rs:527-530` removes extracted camera components when a camera is inactive; `dev/bevy/crates/bevy_render/src/camera.rs:663-722` sorts active cameras by order/target and reports ambiguities. | `zircon_runtime/src/core/framework/render/camera.rs` with `ViewportCameraSnapshot`; `zircon_runtime/src/core/framework/render/camera_ordering.rs` with neutral camera ordering; `zircon_runtime/src/scene/components/scene.rs` with `CameraComponent`; `RenderFrameExtract` in `frame_extract.rs`. | `render::camera` and `render::core_pipeline`. | M2A/M2B landed target, viewport, order, active state, inactive-camera extraction suppression, clear color, HDR, exposure, MSAA, render layers, scene component projection, and scene asset roundtrip. M2C routes headless targets into concrete offscreen submission size and rejects texture targets explicitly until GPU texture residency is ready. M2D adds Bevy-style active camera ordering, ambiguity reporting, and per-target/HDR sorted index contracts. |
| 2D pipeline and phases | `dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs:49-91` registers `Core2d`, `Opaque2d`, `AlphaMask2d`, and `Transparent2d`. | `CorePipelineKind::Core2d`, 2D render phases, `RenderPipelineAsset::default_core2d()`, and sprite phase queues. | `render::core_pipeline` plus `render::sprite` and `render::mesh`. | M4A landed neutral Core2d phase names and pipeline matching. M6A adds sprite queueing into `Opaque2d`, `AlphaMask2d`, and `Transparent2d`; mesh2d draw execution remains future work. |
| 3D pipeline and phases | `dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs:94-157` registers `Core3d`, prepass, deferred, opaque, alpha mask, and transparent phases. | `default_forward_plus.rs`, `default_deferred.rs`, `render_pass_stage.rs`, and compiled scene renderer paths. | `render::core_pipeline` and `render::pbr`. | Current pipelines are global renderer assets, not camera-selected `Core3d` schedules. |
| Camera components | `dev/bevy/crates/bevy_camera/src/components.rs:8-89` defines `Camera2d`, `Camera3d`, and `Hdr`. | `zircon_runtime/src/core/framework/render/camera.rs`; `zircon_runtime/src/scene/components/scene.rs`; `zircon_runtime/src/asset/assets/scene.rs`. | `render::camera`, projected through scene component and scene asset owners. | M2B moved product fields into `CameraComponent` and `SceneCameraAsset`. Editor authoring and concrete graphics target routing remain. |
| Visibility layers | `dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs:10-20`, `45-50`, and `115-135` define default layer 0, empty invisible, and intersection semantics. | `zircon_runtime/src/core/framework/render/camera.rs` with `RenderLayerSet`; `zircon_runtime/src/scene/components/scene.rs` and `scene/world/render.rs` project legacy masks into the snapshot. | `render::camera`, with concrete scene extraction in `zircon_runtime::scene`. | M2A landed Bevy-style layer set semantics and mesh filtering during scene extraction. A later deliberate scene serialization cutover should replace or wrap the `u32` mask in the neutral contract. |
| Image / texture | Bevy `common_api` includes `bevy_image` at `dev/bevy/Cargo.toml:204`. | `zircon_runtime/src/asset/assets/texture/mod.rs`; runtime GPU texture resources under `zircon_runtime/src/graphics/scene/resources/*`. | `render::image` plus asset-side texture projection. | M3A landed `RenderImageDescriptor` with sampler, color space, usage, format, mip/layer counts, and fallback class. Concrete texture fallback stats remain later renderer work. |
| Mesh / model | Bevy `common_api` includes `bevy_mesh` at `dev/bevy/Cargo.toml:205`. | `zircon_runtime/src/asset/assets/model/mod.rs`; `MeshRenderer` in `zircon_runtime/src/scene/components/scene.rs:115-142`. | `render::mesh`. | M3A landed `RenderMeshDescriptor` with topology, bounds, kind, 2D/3D suitability, counts, and VG payload presence. First-class runtime `Mesh2d`/`Mesh3d` scene components remain later work. |
| Shader | Bevy `common_api` includes `bevy_shader` at `dev/bevy/Cargo.toml:206`. | `zircon_runtime/src/asset/assets/shader/mod.rs`; `zircon_runtime/src/graphics/shader/mod.rs`. | `render::shader`. | M3A landed runtime WGSL selection, entry-point descriptors, variant keys, explicit serialized shader dependencies, and serialized pipeline layout descriptors with bind groups/bindings. Automatic source import parsing and deep bind-group reflection remain future work. |
| Material and PBR baseline | Bevy `common_api` includes `bevy_material` at `dev/bevy/Cargo.toml:207`; `StandardMaterial` starts at `dev/bevy/crates/bevy_pbr/src/pbr_material.rs:26`. | `zircon_runtime/src/asset/assets/material/mod.rs`; `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs`; `zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs`. | `render::material` and runtime-owned PBR pipeline keying. | M5A now streams `StandardMaterialDescriptor` into `MaterialRuntime`, PBR texture-slot key bits, alpha/double-sided/unlit pipeline keys, readiness/fallback diagnostics, and renderer material stats. Full shader reflection, `.zmaterial`, material editor authoring, and complete physically based lighting remain future work. |
| Lights | `dev/bevy/crates/bevy_light/src/lib.rs:159-245` wires light visibility, clusters, shadow maps, and directional/point/spot/rect light visibility. | `DirectionalLight`, `PointLight`, and `SpotLight` in `zircon_runtime/src/scene/components/scene.rs:301-355`; render light snapshots in `scene_extract.rs`; submit stats in `update_stats/base_stats.rs`. | `render::light` and runtime `LightingExtract`. | M5A carries directional/point/spot plus neutral ambient/rect DTO vectors through extract, snapshot roundtrip, submit context, and stats. Ambient/rect scene components and concrete shading are still degraded/empty until a later light-authoring/shader milestone. |
| Sprite | `dev/bevy/crates/bevy_sprite_render/src/lib.rs:52-125` wires sprite extraction, image bind groups, and 2D queueing. | `Sprite2dComponent`, `RenderSpriteSnapshot`, `SpriteExtract`, Core2d sprite phase queueing, sprite renderer, graph executors, and sprite stats. | `render::sprite`. | M6A lands non-particle sprite contracts, atlas/rect/flip/anchor/custom-size/tint/z/layer extraction, 2D queueing, fallback texture stats, and product submit evidence. Per-alpha-mode GPU pipeline variants, batching, atlas asset import, and full materialized sprite shaders remain future work. |
| Runtime UI render | `dev/bevy/crates/bevy_ui_render/src/lib.rs:192-270` registers UI extraction and inserts `ui_pass` after post-process and before upscaling for Core2d/Core3d. | `zircon_runtime/src/ui/*`, `zircon_runtime_interface/src/ui/surface/*`, `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`. | `render::ui_render` plus existing runtime UI contracts. | UI render exists but is not yet profile-selected, per-camera targeted, or explicitly ordered relative to Bevy-style core schedules. |
| Post-process | `dev/bevy/crates/bevy_post_process/src/lib.rs:9-39` adds bloom, motion blur, depth of field, effect stack, and MSAA writeback. | `BuiltinRenderFeature::{Bloom, ColorGrading, HistoryResolve, PostProcess}` and `scene_runtime_feature_flags.rs`; post-process resources under `zircon_runtime/src/graphics/scene/scene_renderer/post_process/*`. | `render::post_process`. | Bloom/color grading/history exist, but the per-camera effect stack and several Bevy effects are missing. |
| Anti-aliasing | `dev/bevy/crates/bevy_anti_alias/src/lib.rs:21-35` adds FXAA, SMAA, TAA, CAS, and optional DLSS. | No explicit AA product surface; some quality flags and history paths exist. | `render::anti_alias`. | M8 must add AA components, backend fallback, and per-camera enablement. |
| Solari | `dev/bevy/crates/bevy_solari/src/lib.rs:49-58` requires ray query and binding-array features. | No Zircon Solari owner. Advanced capability fields exist in render capability summaries. | `render::solari` plus `SolariExperimental` profile. | M9 must add explicit Solari profile and backend feature degradation. |
| Virtual Geometry and Hybrid GI | Bevy plan treats these as advanced alongside Solari rather than default profiles. | `zircon_plugins/virtual_geometry/runtime/src/lib.rs`, `zircon_plugins/hybrid_gi/runtime/src/lib.rs`, neutral DTOs in `zircon_runtime/src/core/framework/render/*`, provider facades in `zircon_runtime/src/graphics/*_runtime_provider/*`. | `AdvancedRender` and `SolariExperimental` profile gates, not default profiles. | Existing advanced work must be routed through profiles and camera-driven phases instead of replacing default mesh/PBR/UI rendering. |

## M1 Landing Rules Derived From M0

- M1 has landed `RenderProductProfile` and `RenderProfileBundle` under `zircon_runtime::core::framework::render::profile`, not under concrete `graphics` implementation modules.
- `CommonRenderApi`, `Render2d`, `Render3d`, `Ui`, `DefaultRender`, `AdvancedRender`, and `SolariExperimental` are runtime product choices rather than Cargo feature clones.
- `Headless` is a valid render bundle that carries no render product dependencies and can be selected by `EntryConfig`.
- `DefaultRender` validates as `CommonRenderApi + Render2d + Render3d + Ui` and does not include Virtual Geometry, Hybrid GI, or Solari.
- App-level selection now lives on `EntryConfig::render_profile` and is stored by `BuiltinEngineEntry` in `CoreRuntime` config under `RENDER_PROFILE_CONFIG_KEY` before module activation.

The module-detail doc for this M1 surface is [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md).

## M2-M3 Risk Notes

- M2A/M2B added neutral camera snapshots, scene camera product fields, scene asset projection, and `RenderLayerSet` semantics. `RenderLayerMask(u32)` still needs a later deliberate scene serialization cutover rather than long-lived compatibility aliases.
- Asset work must land before PBR and sprite rendering: `ImageAsset`, `MeshAsset`, shader descriptors, material contracts, and alpha behavior are prerequisites for stable 2D/3D products.
- M6A closes the largest basic sprite gap with `Sprite2dComponent` and `SpriteExtract`; remaining 2D gaps are materialized mesh2d drawing, batching, atlas/importer productization, and per-alpha-mode GPU variants.
- UI render already has strong runtime assets and SDF/text support, but it must become a profile-controlled render pass instead of an incidental compiled-scene side path.
- Advanced VG/HGI paths are powerful but must remain behind profile and backend capability gates so they do not mask missing default 2D/3D/UI behavior.

## M0 Acceptance Evidence

This document maps every Bevy feature collection required by M0 to a Zircon owner module and records the implementation gaps that later milestones must close. Runtime tests are intentionally not run for M0 because the plan defines documentation evidence as the acceptance stage.

## M1 Acceptance Evidence

M1 product-profile validation is now recorded in the module-detail doc for [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md). The fresh 2026-05-08 gates were `cargo test -p zircon_runtime render_profile --locked` and `cargo check -p zircon_app --locked --all-targets`; both completed successfully with warning-only compile output outside the focused render-profile assertions.

## M3A Asset Product Contract Update

M3A gives the profile features for image, mesh, shader, and material a real asset-readiness surface. The owning docs are [Render Assets](../zircon_runtime/asset/render-assets.md) and [Render Material Contracts](../zircon_runtime/core/framework/render/material.md). Renderer phase scheduling, sprite rendering, anti-aliasing, Solari, and deep VG/HGI integration remain later milestones.

## M5A Runtime PBR And Light Baseline Update

M5A gives `Render3d` a runtime-only PBR material/light baseline without changing the coordinated shader/material assetization lane. StandardMaterial descriptor fields now reach runtime material preparation, pipeline variant keys, fallback readiness diagnostics, and renderer material stats. Ambient and rect-light DTO vectors now round-trip through render extract and submit stats with explicit degradation metadata, while world-authored ambient/rect components remain future work. Focused WSL evidence is recorded in [Render Product M5A PBR Light](../../tests/acceptance/render-product-m5a-pbr-light.md).

## M6A Sprite And Default 2D Renderer Update

M6A gives `Render2d` a non-particle sprite path. Runtime scene data can now store `Sprite2dComponent` and `Mesh2dComponent`; sprite components project into `RenderSpriteSnapshot` and `SpriteExtract` with image/material handles, atlas region, rect, flip flags, anchor, custom size, color tint, z order, alpha policy, and render layer mask. World extraction filters sprites by the active camera render layers and adds visible sprites to `VisibilityInput` as dynamic renderables.

The default Core2d pipeline now enables `BuiltinRenderFeature::Sprite` and compiles `sprite.opaque`, `sprite.alpha-mask`, and `sprite.transparent` graph passes for the 2D phase family. The concrete sprite renderer consumes `SpriteExtract.phase_queue`, draws texture-tinted quads through the existing texture streamer fallback path, and records sprite count/readiness/fallback plus sprite graph execution stats. Focused scoped evidence is recorded in [Render Product M6A Sprite Default 2D](../../tests/acceptance/render-product-m6a-sprite-default-2d.md).

## M2A Camera And Layer Contract Update

M2A expands the camera-facing render contract without taking ownership away from scene or graphics modules. `ViewportCameraSnapshot` now carries target, viewport rectangle, render order, active state, clear color, HDR, exposure, MSAA, and `RenderLayerSet`. `RenderViewportRect::clamped_to_size(...)` and `ViewportCameraSnapshot::effective_viewport_size(...)` mirror the Bevy camera viewport size path, while `RenderLayerSet` mirrors Bevy's default layer `0` and empty-set invisibility rule.

Scene render extraction now projects the active camera entity's legacy `RenderLayerMask` into `ViewportCameraSnapshot::render_layers` and filters mesh snapshots by camera/mesh layer intersection. Explicit camera snapshots supplied through `SceneViewportExtractRequest` keep their own layer set, which lets editor/runtime preview requests override the scene camera without changing scene state. Inactive cameras keep their camera snapshot for diagnostics but emit no scene meshes, phase inputs, visibility renderables, or scene lights.

The module-detail doc for this M2A surface is [Runtime Render Camera Contracts](../zircon_runtime/core/framework/render/camera.md).

## M2B Scene Camera Projection Update

M2B moves the camera product surface into scene-level data. `CameraComponent` and `SceneCameraAsset` now carry projection mode, orthographic size, render target, viewport, order, active state, HDR, exposure, clear color, and MSAA. `SceneCameraTargetAsset` uses asset references for texture targets and contributes those references to `SceneAsset::direct_references()`. `World::from_scene_asset(...)` and `World::to_scene_asset(...)` now round-trip those camera fields through project scene assets.

Remaining M2 work is editor authoring for these fields, true texture-target writeback, and a later hard cutover from legacy scene `RenderLayerMask(u32)` to the neutral render layer contract.

M2C entry evidence was captured on 2026-05-16 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-render-camera-m2-1819`: `cargo test -p zircon_runtime camera --locked --jobs 1 --message-format short --color never` passed 13 focused camera/layer/scene-asset tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward. M2C can now begin concrete graphics routing for `RenderCameraTarget::{Texture,Headless}` while keeping the scene/editor authoring and legacy `RenderLayerMask(u32)` cutover as separate later work.

## M2C Camera Target Routing Update

M2C follows the Bevy target-size and missing-target precedent from `dev/bevy/crates/bevy_camera/src/camera.rs:459-483` and `dev/bevy/crates/bevy_render/src/camera.rs:268-328`. Zircon now resolves camera target size during graphics submission: `PrimarySurface` uses the viewport record size, `Headless { size }` drives offscreen submission/capture size, and `Texture(handle)` returns `RenderFrameworkError::UnsupportedCapability { capability: "camera texture render target" }` instead of rendering to the primary viewport.

Presentation remains primary-surface-only for this slice. `Headless` cameras submitted through the surface-present path return `UnsupportedCapability { capability: "headless camera surface present" }`, keeping headless/offscreen capture separate from window blitting until multi-target scheduling and texture residency are ready.

M2C acceptance evidence was captured on 2026-05-16 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-camera-m2c`: `cargo test -p zircon_runtime camera_target --locked --jobs 1 --message-format short --color never` passed 3 focused camera target tests, and `cargo check -p zircon_runtime --lib --locked --message-format short --color never` passed afterward.

## M2D Camera Ordering Update

M2D follows Bevy `SortedCameras` and `sort_cameras` in `dev/bevy/crates/bevy_render/src/camera.rs:663-722`. Zircon now has `sort_render_cameras(...)` under `zircon_runtime::core::framework::render`: inactive cameras are skipped, active cameras are sorted by render order and normalized target key, duplicate active `(order, target)` groups are reported through `RenderCameraOrderAmbiguity`, and `sorted_camera_index_for_target` is assigned per `(target, hdr)`.

This is deliberately a neutral contract rather than a concrete multi-camera render loop. Split-screen, render-to-texture scheduling, editor authoring, and texture writeback remain later slices because the current runtime extract is still single-camera and the asset/GPU texture residency lane is active separately.

M2D acceptance evidence was captured on 2026-05-17 in WSL/Linux with `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-render-camera-m2d-wsl`: `cargo test -p zircon_runtime --lib render_camera_ordering --locked --jobs 1 --message-format short --color never` passed 2 focused ordering tests, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` passed afterward with existing unused-function warnings only. Windows default-feature and core-min attempts both failed before Zircon source at `wgpu-hal 29.0.3` DX12/windows type mismatches.
