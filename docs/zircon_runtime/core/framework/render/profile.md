---
related_code:
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_image/src/image.rs
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_core_pipeline/src/lib.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/solari/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
implementation_files:
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/image/mod.rs
  - zircon_runtime/src/core/framework/render/mesh/mod.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/mod.rs
  - zircon_runtime/src/core/framework/render/light/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/sprite/mod.rs
  - zircon_runtime/src/core/framework/render/post_process/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/solari/mod.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - user: 2026-05-21 continue M10 profile freeze acceptance checklist
  - user: 2026-05-21 continue Bevy default render profile completion gates
  - user: 2026-05-08 continue ZirconEngine Bevy-Level Rendering Completion Plan M1
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - dev/bevy/Cargo.toml
  - dev/bevy/docs/cargo_features.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - cargo test -p zircon_runtime render_profile --locked
  - cargo check -p zircon_app --locked --all-targets
doc_type: module-detail
---

# Runtime Render Profile Contracts

## Purpose

`zircon_runtime::core::framework::render::profile` owns the neutral product-profile vocabulary for Bevy-level rendering. It turns Bevy's Cargo feature profiles and collections into runtime-selectable Zircon product bundles instead of copying Bevy's compile-time feature model.

The contract is intentionally data-only: it defines product names, feature dependencies, backend capability gates, validation errors, and the config key used by `zircon_app` to store the active bundle. Concrete renderer execution stays in `zircon_runtime::graphics`, `zircon_runtime::rhi`, `zircon_runtime::rhi_wgpu`, and `zircon_runtime::render_graph`.

## Bevy Evidence

Bevy profiles and collections drive the shape:

- `dev/bevy/crates/bevy_internal/src/default_plugins.rs:32-62` orders the default render-facing plugins as render initialization, image, mesh, camera, light, pipelined rendering, core pipeline, post-process, anti-aliasing, sprite/sprite render, UI/UI render, PBR, and later optional tooling.
- `dev/bevy/Cargo.toml:133-151` defines `default`, `2d`, `3d`, and `ui` profiles.
- `dev/bevy/Cargo.toml:198-261` defines `common_api`, `2d_api`, `2d_bevy_render`, `3d_api`, `3d_bevy_render`, `ui_api`, and `ui_bevy_render` collections.
- `dev/bevy/docs/cargo_features.md:10-52` explains that profiles are high-level product groups and collections compose those profiles.

Zircon deliberately diverges by making these runtime product choices rather than Cargo features, because the app host already selects `EntryProfile`, `RuntimeTargetMode`, plugin manifests, and export profiles at runtime.

## Data Model

`RenderProductProfile` currently names these product choices:

- `Headless`
- `CommonRenderApi`
- `Render2d`
- `Render3d`
- `Ui`
- `DefaultRender`
- `AdvancedRender`
- `SolariExperimental`

`RenderProductFeature` names the neutral capabilities that a profile bundle can require, such as camera, image, mesh, material, shader, sprite, light, PBR, core pipeline, UI render, render target, post-process, anti-aliasing, Virtual Geometry, Hybrid GI, and Solari.

`RenderProfileBundle` stores the selected profile, implied product profiles, and enabled product features. Its constructors are the canonical bundle definitions:

- `headless()` activates no render products.
- `render_2d()` includes camera/image/mesh/material/shader/sprite/core pipeline and post-process.
- `render_3d()` includes camera/image/mesh/material/shader/light/PBR/core pipeline/post-process/AA.
- `ui()` includes UI render/core pipeline/render target plus common API features.
- `default_render()` includes `CommonRenderApi`, `Render2d`, `Render3d`, and `Ui` without advanced features.
- `advanced_render()` adds Virtual Geometry and Hybrid GI on top of default rendering.
- `solari_experimental()` adds Solari capability requirements on top of advanced rendering.

## Default Render Ordering

Bevy's `DefaultPlugins` sequence matters because the renderer is not just one plugin. It initializes render infrastructure, then image and mesh assets, then camera and light contracts, then core pipeline and post-process passes, then anti-aliasing, sprite/UI render, and PBR. Zircon maps that order into profile features and module contracts instead of trying to make one monolithic renderer module cover every product surface.

The current Zircon ordering contract is:

| Bevy default plugin slice | Zircon product feature or module | Current owner | Notes |
| --- | --- | --- | --- |
| `RenderPlugin` | `RenderProductFeature::CorePipeline` plus concrete graphics submit/present | `render::profile`, `render::core_pipeline`, `zircon_runtime::graphics` | Framework profile names the product need; graphics owns concrete extract/prepare/queue/submit/present. |
| `ImagePlugin` | `RenderProductFeature::Image` | `render::image` plus texture asset projection | Image descriptor, sampler, usage, and fallback vocabulary are explicit; loader and GPU upload milestones stay outside this module. |
| `MeshPlugin` | `RenderProductFeature::Mesh` | `render::mesh` plus mesh/model asset projection | Topology, bounds, counts, 2D/3D suitability, and Virtual Geometry payload presence are explicit. |
| `CameraPlugin` | `RenderProductFeature::Camera` | `render::camera` and `render::camera_ordering` | Camera target, viewport, clear, HDR/exposure/MSAA, render layers, inactive extraction, and ordering contracts are explicit. |
| `LightPlugin` | `RenderProductFeature::Light` | `render::light` | Neutral light snapshots and readiness counts are explicit; shadows, clustered lights, and area-light shading remain PBR milestones. |
| `PipelinedRenderingPlugin` | no standalone Zircon feature yet | future graphics scheduling milestone | Zircon currently keeps pipelined execution as a concrete graphics concern, not a profile feature. |
| `CorePipelinePlugin` | `RenderProductFeature::CorePipeline` | `render::core_pipeline` plus concrete pipeline assets | Core2d/Core3d phase families and deterministic phase queues are explicit. |
| `PostProcessPlugin` | `RenderProductFeature::PostProcess` | `render::post_process` | Stack and pass graph validation are explicit; effect breadth remains incremental. |
| `AntiAliasPlugin` | `RenderProductFeature::AntiAlias` | `render::anti_alias` | DefaultRender requires screen-space AA capability; concrete FXAA fallback exists, while SMAA/TAA/CAS/DLSS remain gaps. |
| `SpritePlugin` / `SpriteRenderPlugin` | `RenderProductFeature::Sprite` | `render::sprite` | Non-particle Core2d sprite product path is explicit; Mesh2d drawing and batching remain future work. |
| `UiPlugin` / `UiRenderPlugin` | `RenderProductFeature::UiRender` and `RenderProductFeature::RenderTarget` | runtime UI plus graphics UI pass | Runtime UI render is a DefaultRender product surface; broader UI widgets/focus/picking/a11y are owned by UI milestones. |
| `PbrPlugin` | `RenderProductFeature::Pbr` | `render::material`, `render::light`, concrete graphics material/runtime pipeline | Runtime PBR material/light stats and fallback readiness exist; full Bevy PBR lighting and shader reflection remain separate. |

This table is the baseline for deciding whether a future render change belongs in the neutral framework contract, asset projection, app profile wiring, or concrete graphics renderer. Advanced Virtual Geometry, Hybrid GI, and Solari are deliberately absent from `DefaultRender`; they remain opt-in products layered after this default surface.

## M10K Default Profile Completion Gates

Bevy's default product surface has two separations that Zircon must keep visible:

- `dev/bevy/Cargo.toml:134-151` defines the default experience as `2d`, `3d`, `ui`, and audio. This render plan excludes audio, but it still must make 2D, 3D, and UI rendering independently complete enough to ship as defaults.
- `dev/bevy/Cargo.toml:198-261` separates `common_api`, `2d_api`, `2d_bevy_render`, `3d_api`, `3d_bevy_render`, `ui_api`, and `ui_bevy_render`. API/descriptor readiness is therefore not the same thing as renderer readiness.
- `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77` places `RenderPlugin`, image/mesh/camera/light, pipelined rendering, core pipeline, post-process, AA, sprite, UI render, and PBR before later optional tooling. Default rendering is a product bundle, not a single flagship renderer feature.

The Zircon promotion gate for `RenderProfileBundle::default_render()` is now:

| DefaultRender slice | Required Bevy-level evidence before calling the slice complete | Evidence that does not close the slice |
| --- | --- | --- |
| `CommonRenderApi` | Stable camera, image, mesh, material, and shader descriptors; asset projection; validation errors that name the missing resource family. | Concrete GPU execution alone, because Bevy separates common API collections from renderer collections. |
| `Render2d` | Core2d phase selection, camera/target selection, sprite product path, material/image fallback reporting, and later Mesh2d/SpriteMesh evidence. | Particle, gizmo, VG, HGI, or Solari graph execution. |
| `Render3d` | Core3d phase selection, StandardMaterial/PBR baseline, directional/point/spot/ambient/rect light readiness, post-process, and AA. | One material struct, one ambient uniform, or advanced GI output without clustered lights, shadows, probes, and PBR pipeline breadth. |
| `Ui` | Profile-controlled UI render pass, target/order stats, and integration with runtime UI extraction. | Widget, picking, or accessibility work alone; those are UI milestones unless their render pass is proven. |
| Presentation | Headless/offscreen, runtime window present, editor viewport present, render-to-texture, and screenshot/capture paths must be classified separately. | A primary-surface blit that silently falls back for all target families. |
| Diagnostics | Default submit must expose enough product diagnostics to explain missing assets, degraded lights/materials, culled passes, present failures, and future timing/allocator data. | RenderDoc markers or aggregate frame counters without pass/product attribution. |
| Advanced separation | `AdvancedRender` and `SolariExperimental` may depend on `DefaultRender`, but their success cannot be cited as evidence that 2D, 3D, UI, presentation, or diagnostics are complete. | Any acceptance statement that uses Virtual Geometry, Hybrid GI, or Solari to replace missing default product behavior. |

This gate turns the original product concern into a repeatable milestone rule: every future M10 acceptance entry must name which `DefaultRender` slice it advances, which Bevy source surface it follows, and which remaining gaps stay outside the accepted claim.

## M10O Profile Freeze Acceptance Checklist

Profile freeze is the first M10 dependency gate. It does not mean the renderer is complete; it means the product vocabulary, default bundle, advanced separation, and app-level storage are fixed enough that later M10 slices can target stable names.

| Requirement | Bevy source pressure | Zircon source evidence | Completion gate |
| --- | --- | --- | --- |
| `DefaultRender` is a default-rendering bundle, not the whole engine default. | `dev/bevy/docs/cargo_features.md:22-52` separates Bevy `default`, `2d`, `3d`, `ui`, `common_api`, and renderer collections. | `zircon_runtime/src/core/framework/render/profile.rs:78-87` builds `DefaultRender` from `CommonRenderApi`, `Render2d`, `Render3d`, and `Ui`. | Keep audio, platform, input, scene, and UI widget/picking ownership outside this profile gate. |
| `DefaultRender` excludes advanced products. | Bevy default plugin order includes normal render/PBR slices before optional advanced tooling in `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43-77`. | `zircon_runtime/src/core/framework/tests.rs:1050-1063` asserts no `AdvancedRender`, `SolariExperimental`, `VirtualGeometry`, `HybridGlobalIllumination`, or `Solari` feature in default render. | No M10 acceptance statement may cite VG/HGI/Solari as default 2D/3D/UI proof. |
| Required feature validation stays explicit. | Bevy collections split API and renderer collections, so renderer readiness must be named rather than implied. | `profile.rs:149-176` validates required profiles and backend capabilities; `framework/tests.rs:1066-1106` rejects missing 2D sprite, 3D PBR, and UI render target dependencies. | Later M10 slices must add or update tests when they change required feature sets. |
| App bootstrap stores the selected profile before runtime modules depend on it. | Bevy plugin groups are configured before app execution. | `zircon_app/src/entry/entry_config.rs:203-207` defaults runtime/editor to `DefaultRender` and headless to `Headless`; `zircon_app/src/entry/tests/profile_bootstrap.rs:446-473` covers runtime default and explicit headless storage under `RENDER_PROFILE_CONFIG_KEY`. | `cargo check -p zircon_app --locked --all-targets` remains the app-profile promotion check. |
| Advanced provider registration is opt-in. | Bevy Cargo features and plugin groups make optional renderer products explicit. | `profile_bootstrap.rs:219-300` covers default render not linking VG/HGI providers and advanced/Solari profiles linking their providers only when selected. | Feature-gated advanced-provider tests stay separate from default profile acceptance. |
| Backend capability gates are not silently bypassed. | Bevy render features are gated by renderer/platform support. | `profile.rs:194-219` maps advanced, AA, and Solari features to backend capabilities; profile module tests cover default AA and Solari capability failures. | Missing capability must be structured `RenderProfileValidationError`, not a fallback renderer success. |

M10.1 can be called profile-frozen only when fresh validation has run for `cargo test -p zircon_runtime render_profile --locked` and `cargo check -p zircon_app --locked --all-targets`, or when a docs-only slice explicitly states that it has not promoted the gate. This document currently records the gate and prior evidence; it does not by itself claim fresh Cargo validation.

## Validation

`RenderProfileBundle::validate()` checks product dependencies before a bundle is accepted. The M1 hard requirements are:

- `Render2d` requires camera, image, mesh, material, shader, sprite, and core pipeline.
- `Render3d` requires camera, image, mesh, material, shader, light, PBR, core pipeline, post-process, and AA.
- `Ui` requires UI render, core pipeline, and render target.

`validate_capabilities(...)` additionally maps product features to backend capability gates through existing neutral `RenderCapabilityKind` values. `VirtualGeometry`, `HybridGlobalIllumination`, and `Solari` remain explicit opt-in paths and are not part of `DefaultRender`.

M9A adds a neutral advanced runtime plan under `render::advanced`. `AdvancedProfileRuntimePlan` evaluates `RenderProfileBundle`, backend capabilities, and provider availability into per-feature reports for Virtual Geometry and Hybrid GI. These reports distinguish `NotRequested`, `Ready`, and `Degraded`, and they record both missing provider and missing backend capability details. Advanced capability validation now includes concrete backend requirements such as storage buffers, indirect draw, and buffer readback rather than relying only on the flagship VG/HGI capability toggles.

M9B adds the neutral Solari contract under `render::solari`. `SolariExperimental` now requires Bevy Solari's runtime GPU capability set: inline ray query, acceleration structures, buffer binding arrays, texture binding arrays, sampled-texture/storage-buffer non-uniform indexing, and partially bound binding arrays. `RenderQualityProfile::with_solari(true)` resolves submit-time profile participation to `SolariExperimental`; `SolariSettings::experimental_enabled()` remains a separate runtime gate so tests and app profile selection can distinguish requested-but-disabled from provider or backend failures.

## App Wiring

`EntryConfig` now carries a `RenderProfileBundle`. Runtime and editor profiles default to `RenderProfileBundle::default_render()`, while headless defaults to `RenderProfileBundle::headless()`. The app host stores the chosen bundle in `CoreRuntime` config under `RENDER_PROFILE_CONFIG_KEY` before module activation so runtime modules and later plugin groups can query the active render product surface without coupling to `zircon_app` internals.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers bundle defaults, dependency rejection for 2D/3D/UI, and advanced capability rejection when backend capability summaries do not satisfy the bundle.

`zircon_app/src/entry/tests/profile_bootstrap.rs` covers app bootstrap storage for default runtime rendering and explicit headless bundle selection.

Fresh M1 validation evidence from 2026-05-08:

- `cargo test -p zircon_runtime render_profile --locked`: passed with `5 passed; 0 failed; 1047 filtered out` for render-profile tests; integration binaries with zero matching tests also completed without failures.
- `cargo check -p zircon_app --locked --all-targets`: passed with warning-only output after lower shared compile blockers in asset animation export/fallback, UI surface pooling serialization, ECS query marker lifetimes, and the active state foundation settled.

Milestone testing commands are the M1 gates from the Bevy-level rendering plan: `cargo test -p zircon_runtime render_profile --locked` and `cargo check -p zircon_app --locked --all-targets`.
