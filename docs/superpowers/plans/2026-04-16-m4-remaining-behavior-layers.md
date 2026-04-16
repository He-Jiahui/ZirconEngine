---
related_code:
  - zircon_render_server/src/types.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/bloom_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/reflection_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_runtime_feature_flags.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/bloom.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/scene/scene_renderer/particle.rs
  - zircon_graphics/src/runtime/offline_bake.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
implementation_files:
  - zircon_render_server/src/types.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/bloom_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/reflection_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_runtime_feature_flags.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/bloom.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/scene/scene_renderer/particle.rs
  - zircon_graphics/src/runtime/offline_bake.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
plan_sources:
  - user: 2026-04-16 continue and complete M4
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m4-runtime-shader-resource-paths.md
  - docs/superpowers/plans/2026-04-16-m4-deferred-runtime-execution.md
tests:
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - cargo test -p zircon_graphics bloom_quality_profile_spreads_bright_pixels_when_enabled --locked
  - cargo test -p zircon_graphics color_grading_extract_tints_scene_after_post_process --locked
  - cargo test -p zircon_graphics offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering --locked
  - cargo test -p zircon_graphics particle_rendering_draws_billboard_sprites_in_transparent_stage --locked
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M4 Remaining Behavior Layers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 M4 里 deferred/SSAO/history 之后剩余的行为层补齐成可运行 baseline：`bloom/color grading`、`reflection probes`、`baked lighting integration + offline bake`、`particle rendering`。

**Architecture:** 保持 `RenderServer -> CompiledRenderPipeline -> SceneRenderer` 的公共边界不变。后处理族继续收敛到 `scene_renderer/post_process/`，`reflection probes / baked lighting` 通过新的 lighting extract 和 post-process runtime 参数接入，`particle rendering` 作为透明阶段独立 runtime pass，`offline bake` 则作为 `zircon_graphics` 的 CPU 基线任务产出可回灌到 `RenderFrameExtract` 的 baked/probe 数据。

**Tech Stack:** Rust, wgpu, zircon_render_server, zircon_scene, zircon_graphics integration tests

**Status:** Completed on 2026-04-16. The remaining M4 behavior layers now execute through the same `RenderServer -> CompiledRenderPipeline -> SceneRenderer` path as the earlier deferred/SSAO/history work, with passing coverage for bloom, color grading, offline bake + baked/probe integration, particle rendering, pipeline compile, render server bridge, `project_render`, and `validate-matrix`.

---

## File Map

- `zircon_render_server/src/types.rs`
  - 扩展 quality profile 的 feature toggles，暴露 `bloom / color_grading / reflection_probes / baked_lighting / particle_rendering` 开关。
- `zircon_scene/src/render_extract.rs`
  - 扩展 `LightingExtract / PostProcessExtract / ParticleExtract` 的真实 runtime 数据。
- `zircon_scene/src/components.rs`
  - 暴露和 extract 对齐的公共 snapshot 类型。
- `zircon_graphics/src/feature/mod.rs`
  - 新增内建 feature：`Bloom`、`ColorGrading`、`ReflectionProbes`、`BakedLighting`。
- `zircon_graphics/src/pipeline/mod.rs`
  - 把这些 feature 放入 built-in Forward+/Deferred pipeline，并维持 deterministic stage/pass order。
- `zircon_graphics/src/runtime/server/mod.rs`
  - 把新增 quality profile 开关映射到 `RenderPipelineCompileOptions.disabled_features`。
- `zircon_graphics/src/scene/scene_renderer/post_process/*`
  - 新增 bloom 纹理/缓冲、reflection probe 缓冲、baked lighting 与 color grading 参数，并更新 WGSL shader。
- `zircon_graphics/src/scene/scene_renderer/particle.rs`
  - 实现 billboard 粒子透明阶段 runtime pass。
- `zircon_graphics/src/runtime/offline_bake.rs`
  - 实现 CPU baseline bake 任务，生成 baked lighting + reflection probe 输出。
- `zircon_graphics/src/tests/m4_behavior_layers.rs`
  - 新增这一批行为层的离屏集成测试，避免继续膨胀 `project_render.rs`。

## Task 1: Extend Extract And Feature Contracts

**Files:**
- Modify: `zircon_render_server/src/types.rs`
- Modify: `zircon_scene/src/components.rs`
- Modify: `zircon_scene/src/render_extract.rs`
- Modify: `zircon_scene/src/lib.rs`
- Modify: `zircon_graphics/src/feature/mod.rs`
- Modify: `zircon_graphics/src/pipeline/mod.rs`
- Modify: `zircon_graphics/src/runtime/server/mod.rs`

- [x] Step 1: Add neutral-by-default extract data types.
  - `RenderBloomSettings`
  - `RenderColorGradingSettings`
  - `RenderReflectionProbeSnapshot`
  - `RenderBakedLightingExtract`
  - `RenderParticleSpriteSnapshot`

- [x] Step 2: Thread them into `LightingExtract / PostProcessExtract / ParticleExtract`.
  - Legacy snapshot compatibility keeps these fields default/empty when unavailable.

- [x] Step 3: Add built-in features and quality toggles.
  - `BuiltinRenderFeature::{Bloom, ColorGrading, ReflectionProbes, BakedLighting}`
  - `RenderFeatureQualitySettings` booleans for the same family plus `particle_rendering`

- [x] Step 4: Put new features into built-in Forward+/Deferred pipelines and wire quality-profile disable paths through `RenderPipelineCompileOptions`.

## Task 2: Lock Post-Process Behavior With Failing Tests

**Files:**
- Create: `zircon_graphics/src/tests/m4_behavior_layers.rs`
- Modify: `zircon_graphics/src/tests/mod.rs`

- [x] Step 1: Write a failing bloom test.
  - Test name: `bloom_quality_profile_spreads_bright_pixels_when_enabled`
  - Render a small bright quad on a dark background.
  - Assert bloom-on creates more bright neighboring pixels than bloom-off.

- [x] Step 2: Run the bloom test and confirm it fails.
  - Run: `cargo test -p zircon_graphics bloom_quality_profile_spreads_bright_pixels_when_enabled --locked`

- [x] Step 3: Write a failing color grading test.
  - Test name: `color_grading_extract_tints_scene_after_post_process`
  - Apply non-neutral grading settings and assert a measurable channel shift.

- [x] Step 4: Run the color grading test and confirm it fails.
  - Run: `cargo test -p zircon_graphics color_grading_extract_tints_scene_after_post_process --locked`

## Task 3: Implement Bloom And Color Grading Runtime

**Files:**
- Modify: `zircon_graphics/src/backend/render_backend/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/scene_runtime_feature_flags.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/resources/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl`

- [x] Step 1: Add `bloom_texture` to `OffscreenTarget`.
- [x] Step 2: Add a bloom fullscreen pass that extracts/softens bright pixels from `scene_color`.
- [x] Step 3: Extend the final post-process shader to consume bloom and color grading parameters.
- [x] Step 4: Re-run bloom/color grading tests and make them pass.

## Task 4: Lock Reflection Probes, Baked Lighting, And Offline Bake With Failing Tests

**Files:**
- Modify: `zircon_graphics/src/tests/m4_behavior_layers.rs`

- [x] Step 1: Write a failing bake/probe integration test.
  - Test name: `offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering`
  - Bake a scene extract, re-apply the bake output, and assert render output changes compared with the unbaked frame.

- [x] Step 2: Run the bake/probe test and confirm it fails.
  - Run: `cargo test -p zircon_graphics offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering --locked`

## Task 5: Implement Reflection Probes, Baked Lighting, And Offline Bake Baseline

**Files:**
- Modify: `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/scene_runtime_feature_flags.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/scene_post_process_resources.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl`
- Create: `zircon_graphics/src/runtime/offline_bake.rs`
- Modify: `zircon_graphics/src/runtime/mod.rs`
- Modify: `zircon_graphics/src/lib.rs`

- [x] Step 1: Add a runtime reflection-probe buffer encoded from extract data into projected screen-space probe influence.
- [x] Step 2: Add baked-lighting parameters to the final post-process composite.
- [x] Step 3: Implement `offline_bake` CPU baseline that emits:
  - `RenderBakedLightingExtract`
  - `Vec<RenderReflectionProbeSnapshot>`
- [x] Step 4: Re-run the bake/probe integration test and make it pass.

## Task 6: Lock Particle Rendering With A Failing Test

**Files:**
- Modify: `zircon_graphics/src/tests/m4_behavior_layers.rs`

- [x] Step 1: Write a failing particle render test.
  - Test name: `particle_rendering_draws_billboard_sprites_in_transparent_stage`
  - Submit `RenderFrameExtract` with particle sprites and assert frame gains visible additive pixels vs particle feature disabled.

- [x] Step 2: Run the particle test and confirm it fails.
  - Run: `cargo test -p zircon_graphics particle_rendering_draws_billboard_sprites_in_transparent_stage --locked`

## Task 7: Implement Particle Runtime Pass

**Files:**
- Create: `zircon_graphics/src/scene/scene_renderer/particle.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/core/mod.rs`

- [x] Step 1: Add a CPU-built billboard vertex stream from `RenderParticleSpriteSnapshot`.
- [x] Step 2: Render it in transparent stage with additive alpha blending into `scene_color`.
- [x] Step 3: Re-run the particle test and make it pass.

## Task 8: Validate And Document The Remaining M4 Slice

**Files:**
- Modify: `docs/assets-and-rendering/srp-rhi-render-server-architecture.md`
- Modify: `docs/assets-and-rendering/index.md`
- Modify: `docs/superpowers/plans/2026-04-16-m4-remaining-behavior-layers.md`

- [x] Step 1: Update architecture docs to record the new extract fields, quality gates, post-process resources, particle pass, and offline bake baseline.
- [x] Step 2: Run compile/runtime regression coverage.
  - Run: `cargo test -p zircon_graphics pipeline_compile --locked`
  - Run: `cargo test -p zircon_graphics render_server_bridge --locked`
  - Run: `cargo test -p zircon_graphics --lib --locked`
- [x] Step 3: Run validator.
  - Run: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`

## Completion Gate

- Bloom and color grading have real runtime shader/resource paths.
- Reflection probes and baked lighting participate in real frame execution and can be produced by an offline bake baseline task.
- Particle sprites render through a dedicated transparent-stage runtime path.
- All new behavior families remain capability/profile gated through `RenderPipelineCompileOptions` and `RenderQualityProfile`.
- Docs and crate-level validation are updated in the same change.
