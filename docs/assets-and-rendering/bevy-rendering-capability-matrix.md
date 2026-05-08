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
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/asset/assets/texture.rs
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/shader.rs
  - zircon_runtime/src/asset/assets/material.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_runtime_feature_flags/scene_runtime_feature_flags.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
implementation_files:
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/assets-and-rendering/render-framework-architecture.md
  - docs/zircon_runtime/core/framework/render/profile.md
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - user: 2026-05-08 implement ZirconEngine Bevy-Level Rendering Completion Plan M0
  - user: 2026-05-08 continue ZirconEngine Bevy-Level Rendering Completion Plan M1
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
tests:
  - "M0 docs acceptance only: no runtime tests required by plan"
  - cargo test -p zircon_runtime render_profile --locked
  - cargo check -p zircon_app --locked --all-targets
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
| `default` | `dev/bevy/Cargo.toml:133-135`, `dev/bevy/docs/cargo_features.md:22-30` combine `2d`, `3d`, `ui`, and `audio`. | `DefaultRender` for rendering scope only. Audio is out of this plan. | Future `zircon_app` profile selection plus `zircon_runtime::core::framework::render` bundles. | Not implemented. M1 must enable 2D, 3D, and UI without Solari, VG, or HGI by default. |
| `common_api` | `dev/bevy/Cargo.toml:198-211` includes camera, image, mesh, shader, material, text, color, and HDR without `bevy_render`. | `CommonRenderApi`. | `zircon_runtime/src/core/framework/render/*`, `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/*`. | Partially present as viewport camera, scene components, texture/model/shader/material assets, and render extract DTOs. |
| `2d_api` | `dev/bevy/Cargo.toml:213-215` adds `bevy_sprite` without a renderer. | `Render2d` API slice. | Future `render::{sprite,mesh,material,camera,core_pipeline}` plus current asset and scene owners. | Missing first-class sprite and 2D mesh/material contracts. |
| `2d_bevy_render` | `dev/bevy/Cargo.toml:216-224` adds `bevy_render`, `bevy_core_pipeline`, `bevy_post_process`, and `bevy_sprite_render`. | `Render2d` implementation bundle. | `zircon_runtime::graphics`, `zircon_runtime::render_graph`, `zircon_runtime::rhi`, future `core_pipeline` and `sprite` contracts. | Current renderer has compiled passes and UI/overlay support, but no camera-driven `Core2d` or sprite renderer. |
| `3d_api` | `dev/bevy/Cargo.toml:226-237` adds light, KTX2, morph, SMAA and tonemapping support around common API. | `Render3d` API slice. | `zircon_runtime/src/scene/components/scene.rs`, `zircon_runtime/src/asset/assets/material.rs`, `zircon_runtime/src/asset/assets/model.rs`, future `render::{light,pbr,mesh,material}`. | Partially present as `CameraComponent`, `MeshRenderer`, directional/point/spot lights, `MaterialAsset`, and `ModelAsset`. |
| `3d_bevy_render` | `dev/bevy/Cargo.toml:239-250` adds `bevy_render`, `bevy_core_pipeline`, `bevy_anti_alias`, `bevy_pbr`, and `bevy_post_process`. | `Render3d` implementation bundle. | `zircon_runtime/src/graphics/pipeline/*`, `zircon_runtime/src/graphics/scene/scene_renderer/*`, future `core_pipeline`, `pbr`, `anti_alias`, and `post_process` contracts. | Current closest paths are `RenderPipelineAsset::default_forward_plus()` and `default_deferred()`, but they are not yet profile-selected Bevy-style `Core3d`. |
| `ui_api` | `dev/bevy/Cargo.toml:252-253` combines `default_app`, `common_api`, and `bevy_ui`. | `Ui` API slice. | `zircon_runtime/src/ui/*`, `zircon_runtime_interface/src/ui/surface/*`, `zircon_runtime/src/asset/assets/ui.rs`. | Existing runtime UI asset/layout/input/render-extract chain exists, but it is not yet a render product profile. |
| `ui_bevy_render` | `dev/bevy/Cargo.toml:255-261` adds `bevy_render`, `bevy_core_pipeline`, and `bevy_ui_render`. | `Ui` implementation bundle. | `zircon_runtime/src/graphics/scene/scene_renderer/ui/*` plus future `render::ui_render`. | Current UI renderer is screen-space and integrated into compiled scene rendering. M7 must make per-camera UI target placement explicit. |
| `bevy_solari` / `SolariPlugins` | `dev/bevy/crates/bevy_solari/src/lib.rs:39-58` adds raytracing scene and Solari lighting with ray query and binding-array requirements. | `SolariExperimental`. | Future `zircon_runtime::core::framework::render::solari` plus advanced `zircon_runtime::graphics` capability checks. | No Zircon Solari owner exists yet. Hybrid GI and raytracing-related capability flags are the closest current advanced path. |

## Capability Matrix

| Product capability | Bevy evidence | Current Zircon landing module | Target Zircon contract owner | Gap before later milestones |
| --- | --- | --- | --- | --- |
| Render sub-app and render stages | `dev/bevy/crates/bevy_render/src/lib.rs:120-208` defines a render sub-app and `RenderSystems`. | `zircon_runtime/src/graphics/runtime/render_framework/*`, `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/*`, `zircon_runtime/src/render_graph/*`. | `render::core_pipeline` for neutral schedule/phase names, concrete execution in `zircon_runtime::graphics`. | Zircon has compiled graph stages and render framework submit paths, but no Bevy-like product profile bundle or `RenderSystems` equivalent contract. |
| Camera-driven core pipeline | `dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:1-11` states rendering is camera driven, and `camera_driver` starts at `schedule.rs:119`. | `zircon_runtime/src/core/framework/render/camera.rs` with `ViewportCameraSnapshot`; `zircon_runtime/src/scene/components/scene.rs` with `CameraComponent`; `RenderFrameExtract` in `frame_extract.rs`. | `render::camera` and `render::core_pipeline`. | `Camera`, `Camera2d`, `Camera3d`, target, viewport, order, clear color, HDR, exposure, MSAA, and schedule selection are not yet explicit. |
| 2D pipeline and phases | `dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs:49-91` registers `Core2d`, `Opaque2d`, `AlphaMask2d`, and `Transparent2d`. | No dedicated 2D core pipeline. Existing render pass stages live in `zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs`. | `render::core_pipeline` plus `render::sprite` and `render::mesh`. | M4 must add `Core2d` schedules and 2D render phases. |
| 3D pipeline and phases | `dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs:94-157` registers `Core3d`, prepass, deferred, opaque, alpha mask, and transparent phases. | `default_forward_plus.rs`, `default_deferred.rs`, `render_pass_stage.rs`, and compiled scene renderer paths. | `render::core_pipeline` and `render::pbr`. | Current pipelines are global renderer assets, not camera-selected `Core3d` schedules. |
| Camera components | `dev/bevy/crates/bevy_camera/src/components.rs:8-89` defines `Camera2d`, `Camera3d`, and `Hdr`. | `zircon_runtime/src/core/framework/render/camera.rs:8-92`; `zircon_runtime/src/scene/components/scene.rs:99-113`. | `render::camera`. | Current `CameraComponent` only has FOV and near/far values. |
| Visibility layers | `dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs:10-20`, `45-50`, and `115-135` define default layer 0, empty invisible, and intersection semantics. | `zircon_runtime/src/scene/components/scene.rs:87-94` and `default_render_layer_mask()` at `scene.rs:419-421`; render masks also flow through render extract DTOs. | `render::camera` or `render::visibility` under framework render. | Current `RenderLayerMask(u32)` is not a Bevy-style set and does not encode empty-layer invisibility as a first-class contract. |
| Image / texture | Bevy `common_api` includes `bevy_image` at `dev/bevy/Cargo.toml:204`. | `zircon_runtime/src/asset/assets/texture.rs`; runtime GPU texture resources under `zircon_runtime/src/graphics/scene/resources/*`. | `render::image` plus asset-side `ImageAsset`. | Missing sampler, color space, GPU usage, format metadata, and fallback opaque/transparent image contracts. |
| Mesh / model | Bevy `common_api` includes `bevy_mesh` at `dev/bevy/Cargo.toml:205`. | `zircon_runtime/src/asset/assets/model.rs`; `MeshRenderer` in `zircon_runtime/src/scene/components/scene.rs:115-142`. | `render::mesh`. | Current product surface is model/primitive oriented, not explicit `MeshAsset`, `Mesh2d`, or `Mesh3d`. |
| Shader | Bevy `common_api` includes `bevy_shader` at `dev/bevy/Cargo.toml:206`. | `zircon_runtime/src/asset/assets/shader.rs`; `zircon_runtime/src/graphics/shader/mod.rs`. | `render::shader`. | Shader variants exist minimally, but embedded/library dependencies, bind-group layouts, and full pipeline descriptors are not explicit. |
| Material and PBR baseline | Bevy `common_api` includes `bevy_material` at `dev/bevy/Cargo.toml:207`; `StandardMaterial` starts at `dev/bevy/crates/bevy_pbr/src/pbr_material.rs:26`. | `zircon_runtime/src/asset/assets/material.rs:5-28`; `zircon_runtime/src/graphics/material/mod.rs`. | `render::material` and `render::pbr`. | `MaterialAsset` is close to a baseline PBR asset, but `Material`, `StandardMaterial`, `ColorMaterial`, `MeshMaterial2d`, and `MeshMaterial3d` contracts are not first-class. |
| Lights | `dev/bevy/crates/bevy_light/src/lib.rs:159-245` wires light visibility, clusters, shadow maps, and directional/point/spot/rect light visibility. | `DirectionalLight`, `PointLight`, and `SpotLight` in `zircon_runtime/src/scene/components/scene.rs:301-355`; render light snapshots in `scene_extract.rs`. | `render::light` and `render::pbr`. | Missing rect/ambient lights, physical defaults, probes, shadow config, fog, and volumetric contracts. |
| Sprite | `dev/bevy/crates/bevy_sprite_render/src/lib.rs:52-125` wires sprite extraction, image bind groups, and 2D queueing. | No first-class runtime sprite product surface; only viewport overlay icon sprite internals exist. | `render::sprite`. | M6 must add sprite contracts, bounds, atlas/slice/flip/anchor handling, and 2D queueing. |
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

- Camera and render layers need hard cutover semantics, not compatibility aliases: the narrow `CameraComponent` and `RenderLayerMask(u32)` do not meet the Bevy camera/layer contract.
- Asset work must land before PBR and sprite rendering: `ImageAsset`, `MeshAsset`, shader descriptors, material contracts, and alpha behavior are prerequisites for stable 2D/3D products.
- Sprite is currently the largest basic-product gap. No current runtime component is equivalent to Bevy `Sprite::from_image`.
- UI render already has strong runtime assets and SDF/text support, but it must become a profile-controlled render pass instead of an incidental compiled-scene side path.
- Advanced VG/HGI paths are powerful but must remain behind profile and backend capability gates so they do not mask missing default 2D/3D/UI behavior.

## M0 Acceptance Evidence

This document maps every Bevy feature collection required by M0 to a Zircon owner module and records the implementation gaps that later milestones must close. Runtime tests are intentionally not run for M0 because the plan defines documentation evidence as the acceptance stage.

## M1 Acceptance Evidence

M1 product-profile validation is now recorded in the module-detail doc for [Runtime Render Profile Contracts](../zircon_runtime/core/framework/render/profile.md). The fresh 2026-05-08 gates were `cargo test -p zircon_runtime render_profile --locked` and `cargo check -p zircon_app --locked --all-targets`; both completed successfully with warning-only compile output outside the focused render-profile assertions.
