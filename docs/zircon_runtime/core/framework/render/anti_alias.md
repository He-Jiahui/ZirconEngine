---
related_code:
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

## Out Of Scope

M8A does not implement MSAA render targets, TAA history jitter/resolution, SMAA, CAS, DLSS, editor material UI, `.zmaterial` schema work, or plugin-provider arbitration. Unsupported modes must degrade through `AntiAliasFallbackReport`; they must not appear as silently successful product paths.
