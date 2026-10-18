# ZirconEngine Bevy-Level Rendering Completion Plan

## Summary

以本地 `dev/bevy` 作为主参考，目标版本为 Bevy `0.19.0-dev` commit `c040d7603`。按你选择的“全功能渲染并进”口径推进：Zircon 同时补齐 Bevy 式基础产品面和高级渲染能力，但里程碑验收仍从基础契约到上层渲染逐层推进。

核心目标：把 `camera / image / mesh / material / shader / light / core_pipeline / pbr / sprite / ui_render / post_process / anti_alias / solari` 变成显式、可组合、可测试、可文档化的 Zircon 产品面；现有 RHI、render graph、virtual geometry、hybrid GI 保留为高级 profile，而不是替代默认 2D/3D/UI 渲染体验。

## Bevy References

- Profile 与 feature 分层：[`dev/bevy/Cargo.toml`](E:/Git/ZirconEngine/dev/bevy/Cargo.toml:137), [`docs/cargo_features.md`](E:/Git/ZirconEngine/dev/bevy/docs/cargo_features.md:25)
- Render sub-app 与渲染阶段：[`bevy_render/src/lib.rs`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_render/src/lib.rs:122), [`RenderSystems`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_render/src/lib.rs:155)
- Camera-driven core pipeline：[`bevy_core_pipeline/src/schedule.rs`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:4), [`camera_driver`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_core_pipeline/src/schedule.rs:119)
- 2D/3D core pipeline：[`core_2d/mod.rs`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_core_pipeline/src/core_2d/mod.rs:34), [`core_3d/mod.rs`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_core_pipeline/src/core_3d/mod.rs:63)
- Camera、layer、光照、PBR、sprite、UI、post/AA/Solari：[`Camera2d/3d`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera/src/components.rs:16), [`RenderLayers`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_camera/src/visibility/render_layers.rs:20), [`LightPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_light/src/lib.rs:161), [`StandardMaterial`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_pbr/src/pbr_material.rs:26), [`SpriteRenderPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_sprite_render/src/lib.rs:54), [`UiRenderPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui_render/src/lib.rs:193), [`PostProcessPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_post_process/src/lib.rs:25), [`AntiAliasPlugin`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_anti_alias/src/lib.rs:23), [`SolariPlugins`](E:/Git/ZirconEngine/dev/bevy/crates/bevy_solari/src/lib.rs:39)

## Public API / Interface Changes

- Add explicit render product profiles under `zircon_runtime::core::framework::render`: `Headless`, `CommonRenderApi`, `Render2d`, `Render3d`, `Ui`, `DefaultRender`, `AdvancedRender`, `SolariExperimental`.
- Split neutral contracts from implementation:
  - `core::framework::render::{camera,image,mesh,material,shader,light,core_pipeline,pbr,sprite,ui_render,post_process,anti_alias,solari}`
  - concrete execution stays under `zircon_runtime::graphics::*`.
- Expand scene/render components:
  - `Camera`, `Camera2d`, `Camera3d`, `Projection`, `RenderTarget`, `Viewport`, `Exposure`, `Hdr`, `Msaa`, `RenderLayers`.
  - `Mesh2d`, `Mesh3d`, `MeshMaterial2d`, `MeshMaterial3d`, `StandardMaterial`, expanded `AlphaMode`.
  - `DirectionalLight`, `PointLight`, `SpotLight`, `RectLight`, `AmbientLight`, probes, shadow config, fog/volumetric config.
- Keep current `ViewportCameraSnapshot`, `RenderExtractPacket`, `RenderFrameExtract`, RHI, render graph, forward-plus/deferred, virtual geometry, and hybrid GI, but route them through explicit profiles and camera-driven core schedules.

## Milestones

### M0: Bevy-to-Zircon Capability Matrix

Goal: produce the source-backed mapping from Bevy modules to Zircon owner modules before implementation.

Implementation slices:
- Map Bevy `common_api`, `2d_api`, `3d_api`, `ui_api`, `2d_bevy_render`, `3d_bevy_render`, `ui_bevy_render` to Zircon profiles.
- Map Bevy source files above to existing Zircon files such as [`camera.rs`](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/render/camera.rs:9), [`scene.rs`](E:/Git/ZirconEngine/zircon_runtime/src/scene/components/scene.rs:99), [`MaterialAsset`](E:/Git/ZirconEngine/zircon_runtime/src/asset/assets/material.rs:14), [`default_forward_plus`](E:/Git/ZirconEngine/zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs:9).
- Update `docs/assets-and-rendering/render-framework-architecture.md` because it still contains stale pre-absorption paths.

Testing stage:
- No runtime tests required.
- Acceptance evidence is a docs matrix with Bevy source/doc references and exact Zircon landing modules.

### M1: Product Profiles And Module Surface

Goal: make render feature groups first-class and app-selectable.

Implementation slices:
- Add `RenderProductProfile` and `RenderProfileBundle`.
- Wire `zircon_app` profile selection to activate render profile bundles.
- Add profile validation: `Render2d` requires camera/image/mesh/material/shader/sprite/core pipeline; `Render3d` requires camera/image/mesh/material/shader/light/pbr/core pipeline/post/AA; `Ui` requires UI + core pipeline + render target.

Testing stage:
- `cargo test -p zircon_runtime render_profile`
- `cargo check -p zircon_app --locked --all-targets`
- Acceptance: profiles reject missing dependencies and default render profile enables 2D + 3D + UI without enabling Solari/VG/GI by default.

### M2: Camera, Render Target, Visibility Layers

Goal: reach Bevy-level camera semantics.

Implementation slices:
- Replace the narrow `CameraComponent` with explicit `Camera`, `Camera2d`, `Camera3d`, projection, viewport, order, clear color, target, HDR, exposure, MSAA/writeback settings.
- Replace `render_layer_mask: u32` with `RenderLayers` default layer `0`, empty means invisible, and `intersects` matching Bevy semantics.
- Make core pipeline camera-driven: each extracted camera selects `Core2d`, `Core3d`, or custom schedule.

Testing stage:
- Unit tests for projection defaults, viewport aspect, render layer intersection, empty-layer invisibility, camera ordering.
- Integration tests for split-screen and render-to-texture extract packets.
- `cargo test -p zircon_runtime camera render_layers render_target --locked`

### M3: Image, Mesh, Shader, Material Foundation

Goal: expose the basic authoring/rendering assets before PBR specialization.

Implementation slices:
- Add `ImageAsset` with sampler, sRGB/linear, GPU usage, fallback opaque/transparent images, and image format metadata.
- Add `MeshAsset`, `Mesh2d`, `Mesh3d`, primitive constructors, bounds/AABB extraction, asset-change invalidation.
- Expand shader contracts with embedded/library shader dependencies, pipeline descriptors, bind group layout descriptors, shader variant keys.
- Expand material contracts with Bevy-style `Material`, `StandardMaterial`, `ColorMaterial`, `MeshMaterial2d/3d`, and full `AlphaMode`.

Testing stage:
- Asset serialization round trips for image/mesh/material/shader.
- Negative tests for missing textures, wrong color space, unsupported shader stage, invalid alpha cutoff.
- `cargo test -p zircon_runtime asset material shader mesh image --locked`

### M4: Core 2D/3D Pipeline And Render Phases

Goal: make default rendering feel Bevy-complete before advanced effects.

Implementation slices:
- Add `Core2d` and `Core3d` schedules with `Prepass`, `MainPass`, `EarlyPostProcess`, `PostProcess`.
- Add render phases: `Opaque2d`, `AlphaMask2d`, `Transparent2d`, `Opaque3d`, `AlphaMask3d`, `Transparent3d`, plus hooks for shadow/deferred/prepass phases.
- Re-map current forward-plus/deferred pipeline assets into these schedules instead of treating them as standalone defaults.

Testing stage:
- Phase queue/sort tests for opaque, alpha mask, transparent.
- Camera-driven multi-camera schedule tests.
- `cargo test -p zircon_runtime core_pipeline render_phase --locked`

### M5: Light And PBR Product Baseline

Goal: Bevy-style 3D scene works with physical-ish defaults and PBR assets.

Implementation slices:
- Add directional/point/spot/rect/ambient lights with lux/lumens defaults, shadow-map config, cascades/frusta, contact shadow flags.
- Add environment map, irradiance volume, reflection probe, lightmap, fog, volumetric fog contracts.
- Implement `StandardMaterial` baseline: base color/texture, emissive, metallic, roughness, normal, occlusion, alpha, double-sided, unlit; stage clearcoat/specular/transmission/IOR/thickness/anisotropy behind capability flags.

Testing stage:
- Render extract tests for all light variants and PBR material fields.
- Default 3D scene fixture matching Bevy `examples/3d/3d_scene.rs`.
- `cargo test -p zircon_runtime light pbr standard_material --locked`

### M6: Sprite And Default 2D Renderer

Goal: Bevy-style `Camera2d + Sprite::from_image` equivalent works.

Implementation slices:
- Add sprite contract: image handle, atlas, rect, flip, anchor, custom size, slicing/tile mode.
- Add 2D bounds calculation for `Mesh2d` and `Sprite`.
- Add sprite render plugin path that extracts sprites, prepares image bind groups, and queues 2D phases.

Testing stage:
- Sprite image, atlas, slice, flip, bounds, and visibility tests.
- Example parity fixture based on Bevy `examples/2d/sprite.rs`.
- `cargo test -p zircon_runtime sprite core2d --locked`

### M7: Runtime UI Render Product Surface

Goal: current Zircon UI becomes an explicit Bevy-style render product, not an editor-only or overlay side path.

Implementation slices:
- Route runtime UI through `UiRenderProfile` and per-camera UI target selection.
- Keep existing `.ui.toml` runtime UI asset chain, text/SDF renderer, image/SVG visual assets, and pointer/navigation dispatch.
- Insert UI pass after post-process and before upscaling for both 2D and 3D cameras, matching Bevy UI render placement.

Testing stage:
- Existing UI tests plus camera-targeted UI render tests.
- Example parity fixture based on Bevy `examples/ui/widgets/button.rs`.
- `cargo test -p zircon_runtime ui_render runtime_ui --locked`
- `cargo check -p zircon_editor --locked --lib`

### M8: Post-Process And Anti-Alias Stack

Goal: post/AA are per-camera feature components, not hardwired global renderer flags.

Implementation slices:
- Add effect stack with bloom, color grading, auto exposure, motion blur, depth of field, chromatic aberration.
- Add AA components: FXAA, SMAA, TAA, CAS; DLSS remains capability-gated placeholder unless SDK/backend is present.
- Define profile defaults: 2D gets tonemapping/upscaling path, 3D gets post + AA stack, UI remains unaffected by 3D AA except explicit UI AA setting.

Testing stage:
- Per-camera enable/disable tests and capability fallback tests.
- Negative tests for unsupported AA/backend combinations.
- `cargo test -p zircon_runtime post_process anti_alias quality_profile --locked`

### M9: Solari, Virtual Geometry, Hybrid GI Advanced Track

Goal: advanced rendering advances in parallel but behind explicit capability and profile gates.

Implementation slices:
- Add `SolariExperimental` profile with required backend feature checks equivalent to Bevy Solari required WGPU features.
- Map existing hybrid GI into `solari::realtime` style contracts and keep pathtracer/offline validation as a separate non-default mode.
- Keep virtual geometry as `AdvancedRender` profile and integrate it with core phases instead of bypassing base mesh/PBR phases.
- Advanced features must report degradation clearly when ray query, acceleration structures, async compute, or bindless resources are unavailable.

Testing stage:
- Capability matrix tests for each advanced feature.
- Hybrid GI/VG existing plugin tests plus profile-gated activation tests.
- `cargo test -p zircon_runtime advanced_render solari hybrid_gi virtual_geometry --locked`
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets`

### M10: Examples, Docs, And Completion Sweep

Goal: users can see and trust the Bevy-level product surface.

Implementation slices:
- Add Zircon examples/test fixtures matching Bevy: 2D sprite, 3D scene, PBR material grid, UI button, render-to-texture, split-screen, post-processing, anti-aliasing, Solari.
- Update docs under `docs/assets-and-rendering/`, `docs/ui-and-layout/`, and module mirrors under `docs/zircon_runtime/...`.
- Add a “Bevy parity checklist” doc with intentional divergences: runtime profiles instead of Cargo features, existing VM/plugin architecture, existing UI asset format, advanced renderer behind profiles.

Testing stage:
- Full CI-equivalent commands from `.github/workflows/ci.yml`:
  - `cargo build --workspace --locked --verbose`
  - `cargo test --workspace --locked --verbose`
  - `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose`
  - `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`
- Acceptance: all example fixtures compile, docs references are current, and no advanced feature is required for default 2D/3D/UI rendering.

## Assumptions

- Bevy reference is the checked-in local source at `dev/bevy`, not a newer online release.
- Scope is rendering/product surface parity, not full Bevy parity for audio, input, ECS, or editor UX beyond render-facing needs.
- Zircon keeps its current four workspace members and `zircon_runtime` absorption architecture.
- Existing advanced rendering is preserved, but default user-facing profiles must work without VG, hybrid GI, or Solari.
- No long-term compatibility shim is required when old render paths conflict with the explicit profile/module surface.
