---
related_code:
  - dev/bevy/crates/bevy_anti_alias/src/lib.rs
  - dev/bevy/crates/bevy_anti_alias/src/fxaa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/smaa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs
  - dev/bevy/crates/bevy_anti_alias/src/contrast_adaptive_sharpening/mod.rs
  - dev/bevy/examples/3d/anti_aliasing.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mode.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/fallback.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/anti_alias.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
implementation_files:
  - zircon_runtime/src/core/framework/render/anti_alias/mod.rs
  - zircon_runtime/src/core/framework/render/anti_alias/mode.rs
  - zircon_runtime/src/core/framework/render/anti_alias/settings.rs
  - zircon_runtime/src/core/framework/render/anti_alias/fallback.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/post_process/effect.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/anti_alias.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias/fxaa.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/params/post_process_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/new/create_pipeline_bundle/post_process_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process.wgsl
plan_sources:
  - user: 2026-05-18 continue Render M8A anti-alias product surface
  - user: 2026-05-20 continue Bevy-level render anti-alias evidence mapping
  - user: 2026-05-22 continue M10 post-process and anti-alias breadth checklist
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - cargo test -p zircon_runtime --locked render_product_anti_alias --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked render_product_post_process --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked render_product_pipeline --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib builtin_registry_covers_product_postprocess_executor_ids --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib capability_validation --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Anti-Alias Product Surface

## Purpose

`zircon_runtime::core::framework::render::anti_alias` owns the neutral M8A anti-alias contract. It turns `RenderProductFeature::AntiAlias` from profile vocabulary into a per-view product setting with explicit resolution and fallback reporting.

The framework layer only describes requested and effective modes. Concrete pixels remain in `zircon_runtime::graphics`, where the first implemented path is FXAA inside the postprocess product surface.

## Bevy Evidence

The Bevy reference is deliberately broad. `dev/bevy/crates/bevy_anti_alias/src/lib.rs:23-33` wires `FxaaPlugin`, `SmaaPlugin`, `TemporalAntiAliasPlugin`, `CasPlugin`, and optional `DlssPlugin` behind one `AntiAliasPlugin`.

`dev/bevy/crates/bevy_anti_alias/src/fxaa/mod.rs:57-108` defines the per-camera `Fxaa` component, sensitivity settings, and Core2d/Core3d post-process scheduling after tonemapping. `fxaa/mod.rs:191-218` prepares a per-view specialized FXAA pipeline.

`dev/bevy/crates/bevy_anti_alias/src/smaa/mod.rs:1-31` documents SMAA's quality/performance tradeoffs and current Bevy limitations, and `smaa/mod.rs:137-196` makes the three-pass shape explicit: edge detection, blending-weight calculation, and neighborhood blending. `smaa/mod.rs:290-359` registers LUTs, pipeline preparation, temporary textures, bind groups, and Core2d/Core3d post-process scheduling.

`dev/bevy/crates/bevy_anti_alias/src/taa/mod.rs:47-72` registers TAA extraction, jitter, pipeline, and history-texture preparation, then schedules TAA in Core3d early post-process. `taa/mod.rs:101-115` requires temporal jitter, mip bias, depth prepass, and motion-vector prepass; `taa/mod.rs:152` rejects TAA when MSAA is enabled. `dev/bevy/crates/bevy_anti_alias/src/contrast_adaptive_sharpening/mod.rs:40-122` defines camera-facing CAS sharpening settings and schedules CAS after FXAA for Core2d/Core3d.

`dev/bevy/examples/3d/anti_aliasing.rs` is the practical product target: it lets one camera switch between no AA, MSAA, FXAA, SMAA, TAA, and optional DLSS. Zircon's current product surface names the same families, but only FXAA is a concrete renderer path.

## Product Surface

`AntiAliasMode` names the product vocabulary: `Off`, `Auto`, `Fxaa`, `Msaa`, `Taa`, `Smaa`, `Cas`, and `Dlss`. The modes are intentionally broader than the first concrete implementation so authoring code can request future modes without the renderer silently claiming support.

`AntiAliasSettings` currently stores one mode and defaults to `Auto`. `RenderViewExtract` carries the settings beside the camera and core pipeline. Camera snapshots with `msaa_samples > 1` map to `AntiAliasMode::Msaa { samples }`; otherwise the view defaults to `Auto`.

`AntiAliasFallbackReport` records `requested_mode`, `effective_mode`, and an optional `AntiAliasFallbackReason`. `Auto` resolving to FXAA is reported as `AutoResolvedToFxaa`, while unsupported SMAA/CAS/DLSS, unsupported MSAA sample counts, unsupported TAA, missing TAA history, and unsupported FXAA each have distinct reasons.

## Capability Resolution

`RenderCapabilitySummary` now carries screen-space and future AA capability fields:

- `supports_fxaa`
- `supports_smaa`
- `supports_taa`
- `supports_cas`
- `supports_dlss`
- `max_supported_msaa_samples`

The current WGPU summary reports FXAA support when offscreen rendering is available, keeps SMAA/TAA/CAS/DLSS disabled, and reports `max_supported_msaa_samples = 1`. That makes default 3D resolve `Auto -> Fxaa` without pretending MSAA or temporal modes are implemented.

`RenderCapabilityKind::ScreenSpaceAntiAlias` and `RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias` are the validation hook for default AA. `RenderProfileBundle::default_render()` requires this capability through its `AntiAlias` feature, and `RenderQualityProfile::with_anti_alias(false)` disables the built-in AA render feature for profiles that intentionally want no AA pass.

## Graph And Renderer Flow

`BuiltinRenderFeature::AntiAlias` contributes a `fxaa` pass at `RenderPassStage::PostProcess` with executor id `post.fxaa`. Default Forward+ and Deferred include the feature after the base postprocess feature and before runtime UI, so the compiled graph order is `post-process -> fxaa -> runtime-ui`.

Runtime submission resolves the requested view settings against backend capabilities and actual history availability before it builds the effective postprocess stack. When the effective mode is FXAA, `PostProcessStackDescriptor::from_extract_settings_with_anti_alias(...)` makes `final-composite` write `postprocess.final-composited` and adds an `fxaa` node that writes the public `final-color`. The compiled-scene resource import aliases both names to the frame target while the current concrete shader remains a single WGPU postprocess pass.

The concrete shader gets an `anti_alias` uniform lane. When enabled, it samples the current scene color around the fragment and blends high-contrast edge pixels before color grading. This is a first FXAA product path, not an MSAA render-target implementation and not a temporal AA history path.

## Stats And Diagnostics

`RenderStats` reports:

- `last_anti_alias_fallback`
- `last_anti_alias_graph_executed_pass_count`

The fallback report proves what the renderer actually used for the submitted frame. The graph count proves the FXAA executor participated in the product graph. Existing postprocess stats also expose the `fxaa` node in `last_post_process_graph_executed_nodes` when the effective graph enables FXAA.

## Bevy Gap Classification

| Bevy AA family | Zircon product state | Completion requirement |
| --- | --- | --- |
| FXAA | Concrete DefaultRender path. `Auto` resolves to FXAA when the backend supports offscreen rendering, and the post-process graph records the `fxaa` node. | Add Bevy-style sensitivity controls if user-facing tuning becomes part of the profile surface. |
| MSAA | Named through `AntiAliasMode::Msaa { samples }` and camera MSAA projection, but current capability reports max supported samples as `1`. Unsupported counts degrade through `AntiAliasFallbackReport`. | Add multisampled render targets, resolve/writeback policy, and sorted-camera writeback behavior before claiming Bevy parity. |
| SMAA | Named but unsupported. It degrades to FXAA or Off with `UnsupportedSmaa`. | Add three-pass temporary resources, LUT handling, presets, and Core2d/Core3d graph nodes. |
| TAA | Named but unsupported. Missing history and unsupported temporal capability are distinct fallback reasons. | Add temporal jitter, mip-bias, depth/motion-vector prepass ownership, history reset, and Core3d early post-process scheduling. |
| CAS | Named but unsupported. It degrades with `UnsupportedCas`. | Add camera-facing sharpening settings and schedule CAS after the chosen screen-space AA pass. |
| DLSS | Named but unsupported and treated as optional capability-gated provider work. | Add explicit backend/provider capability checks before exposing it as anything other than a degraded optional mode. |

## M10.6 Promotion Gate

M10.6 uses this document as the anti-alias side of the post-process/AA breadth gate. The accepted state is intentionally narrow: FXAA is concrete, `Auto` can resolve to FXAA, unsupported families degrade through structured fallback reports, and the graph records the FXAA node. That is not equivalent to Bevy's complete `AntiAliasPlugin`, which installs FXAA, SMAA, TAA, CAS, and optional DLSS as distinct camera-facing products.

| AA family | Current Zircon evidence | Promotion requirement |
| --- | --- | --- |
| FXAA | Concrete graph pass and submit stats exist for DefaultRender. | Keep it as the accepted screen-space fallback and add sensitivity controls only when they become user-facing product settings. |
| MSAA | Camera MSAA requests are projected into `AntiAliasMode::Msaa`, but capability summary reports only one sample today. | Add multisampled render targets, resolve/writeback policy, sorted-camera writeback diagnostics, and interaction rules with TAA before accepting MSAA parity. |
| SMAA | `AntiAliasMode::Smaa` names the family and degrades with `UnsupportedSmaa`. | Add three graph passes, temporary edge/blend textures, area/search LUT handling, quality presets, missing-LUT diagnostics, and Core2d/Core3d pass-order tests. |
| TAA | `AntiAliasMode::Taa` names the family and distinguishes missing history from unsupported temporal capability. | Add temporal jitter, mip bias, depth and motion-vector prepass ownership, history reset behavior, MSAA conflict diagnostics, and Core3d early-postprocess ordering. |
| CAS | `AntiAliasMode::Cas` names the family and degrades with `UnsupportedCas`. | Add camera-facing sharpening settings, denoise mode, a graph node after the chosen AA pass, and pass-order diagnostics. |
| DLSS | `AntiAliasMode::Dlss` names an optional provider-backed family and degrades with `UnsupportedDlss`. | Keep it capability/provider gated; do not expose it as accepted behavior without backend/provider checks and fallback diagnostics. |

Promotion requires `cargo test -p zircon_runtime --locked render_product_anti_alias`, post-process graph coverage, pipeline/pass-order coverage, and `cargo check -p zircon_runtime --lib --locked` in a quiet validation window. The 2026-05-26 M10W run passed those focused current-checkout gates while keeping MSAA, SMAA, TAA, CAS, and DLSS as explicit gaps.

2026-05-26 M10W validation evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked render_product_anti_alias --jobs 1 --message-format short --color never`: PASS, 3 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked render_product_post_process --jobs 1 --message-format short --color never`: PASS, 9 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_ui_graph_pass_order --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never`: PASS, 39 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.

## Out Of Scope

M8A does not implement MSAA render targets, TAA history jitter/resolution, SMAA, CAS, DLSS, editor material UI, `.zmaterial` schema work, or plugin-provider arbitration. Unsupported modes must degrade through `AntiAliasFallbackReport`; they must not appear as silently successful product paths.
