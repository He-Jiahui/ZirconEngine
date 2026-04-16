---
related_code:
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/deferred.rs
  - zircon_graphics/src/scene/scene_renderer/history.rs
  - zircon_graphics/src/scene/scene_renderer/mesh.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/tests/project_render.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
implementation_files:
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/deferred.rs
  - zircon_graphics/src/scene/scene_renderer/mesh.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/tests/project_render.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
plan_sources:
  - user: 2026-04-16 continue the remaining M4 runtime work without stopping
  - user: 2026-04-16 continue the next task and finish as much of the remaining roadmap as possible
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m4-runtime-shader-resource-paths.md
tests:
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M4 Deferred Runtime Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把内建 deferred pipeline 从 compile skeleton 推进为真实 `GBuffer -> deferred lighting -> shared post/history/overlay` runtime 路径，并且继续通过 `RenderServer`、pipeline asset 与 quality/capability 边界驱动。

**Architecture:** 保持 `RenderPipelineAsset`、`RenderServer`、`RenderPipelineCompileOptions` 的公共边界不变，只在 `zircon_graphics::scene::SceneRenderer` 内部补一条 capability-safe 的 deferred runtime。Forward+ 继续保留现有 mesh shader 直写 `scene_color` 的路径，Deferred 则固定改走材质解码的 GBuffer 几何 pass 和 fullscreen deferred lighting，再复用已经落地的 SSAO、clustered lighting、history resolve、post-process 和 overlay。

**Tech Stack:** Rust, wgpu, zircon_render_server, zircon_scene, zircon_graphics integration tests

---

## File Map

- `zircon_graphics/src/tests/project_render.rs`
  - 新增真实失败用例，证明 built-in deferred 不能继续沿用 forward shader 直写路径。
- `zircon_graphics/src/backend/render_backend/mod.rs`
  - 为 offscreen target 增加 deferred 所需的 GBuffer 纹理。
- `zircon_graphics/src/scene/scene_renderer/deferred.rs`
  - 新增 deferred geometry/deferred lighting 运行时资源与 WGSL shader。
- `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
  - 基于 `CompiledRenderPipeline` 的 feature 集在 runtime 中分支 forward/deferred 路径。
- `zircon_graphics/src/scene/scene_renderer/mesh.rs`
  - 暴露 mesh draw 的透明/不透明分类信息，供 deferred 几何与透明补绘共用。
- `zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs`
  - 把 preview sky、mesh scene content 继续拆细，允许 deferred 路径只复用背景和透明 mesh，而不回退整条 forward base-scene pass。
- `zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs`
  - 支持按 draw 列表子集录制 mesh 内容。
- `docs/assets-and-rendering/srp-rhi-render-server-architecture.md`
  - 记录 deferred runtime 真实资源链和与 forward 的边界差异。
- `docs/assets-and-rendering/index.md`
  - 更新目录概览，标注 deferred runtime 已不再只是 skeleton。

## Task 1: Lock The Deferred Runtime Gap With A Failing Integration Test

**Files:**
- Modify: `zircon_graphics/src/tests/project_render.rs`

- [x] Step 1: Add a test that renders the same fullscreen quad through built-in forward and built-in deferred.
  - Material contract:
    - material base color is red
    - custom project shader outputs green
    - texture is fully white to avoid texture noise
  - Forward expectation: frame stays green-dominant because forward uses the project shader.
  - Deferred expectation: frame becomes red-dominant because deferred must decode material/base-color through GBuffer instead of executing the project shader fragment.

- [x] Step 2: Run the focused test and verify it fails before any production change.
  - Run: `cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked`
  - Expected: FAIL because deferred currently routes through the same forward-style `record_scene_content(...)` path and still comes out green.

## Task 2: Add Deferred Runtime Resources And Routing

**Files:**
- Modify: `zircon_graphics/src/backend/render_backend/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/mesh.rs`

- [x] Step 1: Extend `OffscreenTarget` with a dedicated `gbuffer_albedo` render target.
  - Requirements:
    - sampled in deferred lighting
    - cleared/rewritten per frame
    - kept internal to `zircon_graphics`

- [x] Step 2: Teach runtime feature detection to distinguish deferred execution from forward execution.
  - `CompiledRenderPipeline` should continue to be the only runtime decision input.
  - No new public API on `RenderServer`.

- [x] Step 3: Make mesh draw classification explicit so deferred can write opaque draws into GBuffer and forward-render transparent draws later.
  - Keep forward path behavior unchanged.

## Task 3: Implement Real GBuffer Geometry And Deferred Lighting Passes

**Files:**
- Create: `zircon_graphics/src/scene/scene_renderer/deferred.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs`

- [x] Step 1: Add a fixed deferred geometry pipeline that writes material albedo/tint into `gbuffer_albedo`.
  - It must bind:
    - scene uniform
    - model uniform
    - texture bind group
  - It must not compile or execute project fragment shaders.

- [x] Step 2: Add a fullscreen deferred lighting pipeline that reads:
  - `gbuffer_albedo`
  - `normal`
  - background clear/sky color
  - scene lighting uniform
  - It writes lit opaque color into `scene_color`.

- [x] Step 3: Split scene content recording so deferred can reuse:
  - preview sky / clear color as background
  - transparent forward mesh rendering after deferred lighting
  - overlay/gizmo rendering after post-process

- [x] Step 4: Route `SceneRenderer::render_frame_with_pipeline(...)` like this:
  - Forward+: existing normal-prepass + forward scene color + shared SSAO/cluster/history/post/overlay
  - Deferred:
    - preview sky or clear color background
    - normal prepass
    - GBuffer opaque geometry
    - shared SSAO and clustered-light compute
    - deferred lighting fullscreen pass to `scene_color`
    - transparent mesh forward pass into `scene_color`
    - shared post-process/history resolve to `final_color`
    - overlay pass into `final_color`

## Task 4: Validate The Deferred Runtime Path And Update Docs

**Files:**
- Modify: `docs/assets-and-rendering/srp-rhi-render-server-architecture.md`
- Modify: `docs/assets-and-rendering/index.md`

- [x] Step 1: Re-run the focused deferred integration test and confirm green -> red divergence now passes.
  - Run: `cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked`

- [x] Step 2: Run the existing bridge and compile coverage to ensure the new runtime path did not break pipeline selection or stats plumbing.
  - Run: `cargo test -p zircon_graphics pipeline_compile --locked`
  - Run: `cargo test -p zircon_graphics render_server_bridge --locked`

- [x] Step 3: Run crate-level validation.
  - Run: `cargo test -p zircon_graphics --lib --locked`
  - Run: `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`

- [x] Step 4: Update architecture docs.
  - Document the new runtime resource chain:
    - `final_color` temporarily hosts preview-sky background before post-process
    - `gbuffer_albedo` stores opaque material decode
    - deferred lighting shades into `scene_color`
    - transparent and overlays remain separate late passes

## Completion Gate

- Built-in deferred no longer reuses forward project fragment shaders for opaque geometry.
- Deferred viewport output is observably different from forward output in the new integration test.
- Deferred runtime still stays behind `RenderServer` and `RenderPipelineAsset`; no `wgpu` type leaks upward.
- Shared SSAO / clustered lighting / history resolve resource chain continues to work after deferred lighting is inserted.
- Docs and validation are updated in the same change.
