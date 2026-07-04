# Skybox IBL PBR Matrix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Plan 11 EL-M1 skybox + IBL reflection path and export a real engine-rendered 8x8 metallic/smoothness PBR matrix PNG under `docs/tests/runtime/shader`.

**Architecture:** Add a formal environment contract in `core/framework/render`, route frame extraction through `EnvironmentExtract`, implement renderer-owned skybox/environment resources under `graphics/scene/scene_renderer/environment`, then make the existing PBR material path sample the environment. The 8x8 matrix is a product test artifact, not an offline drawing.

**Tech Stack:** Rust 2021, `zircon_runtime`, WGPU, WGSL, `image` crate PNG output, existing `SceneRenderer` offscreen path, Plan 11 environment-lighting contracts.

---

## Scope

This plan implements Plan 11 EL-M1 only:

- formal skybox/environment extract;
- procedural skybox as the first environment source;
- IBL-like environment reflection sampling in the standard/fallback PBR material path;
- product screenshot export for the requested 8x8 matrix.

This plan does not implement reflection-probe capture, baked lightmaps, light probe grids, analytic fog, or external cubemap asset import. The new contracts must leave those paths with explicit owners but must not add fake probe/lightmap behavior.

## File Structure

Create:

- `zircon_runtime/src/core/framework/render/environment/mod.rs`: thin module route and curated exports.
- `zircon_runtime/src/core/framework/render/environment/skybox.rs`: `SkyboxSettings`, `SkyboxMode`, `ProceduralSkyParams`, and environment bake-key helpers.
- `zircon_runtime/src/core/framework/render/environment/extract.rs`: `EnvironmentExtract` frame snapshot.
- `zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs`: renderer environment module route.
- `zircon_runtime/src/graphics/scene/scene_renderer/environment/procedural_environment.rs`: CPU/GPU-side deterministic procedural environment sampling helpers and roughness/smoothness mapping tests.
- `zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl`: formal skybox shader using the Plan 11 gradient parameters.
- `zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl`: PBR environment sampling include used by fallback/standard material paths.
- `zircon_runtime/src/graphics/tests/render_product_environment.rs`: environment product tests and ignored PNG export.

Modify:

- `zircon_runtime/src/core/framework/render/mod.rs`: export environment contract types.
- `zircon_runtime/src/core/framework/render/camera.rs`: map `ViewportRenderSettings::preview_skybox` to `EnvironmentExtract` defaults while the public setting name remains stable.
- `zircon_runtime/src/core/framework/render/frame_extract.rs`: add `environment: EnvironmentExtract` to frame snapshots and defaults.
- `zircon_runtime/src/core/framework/render/scene_extract.rs`: make preview environment state derive from `EnvironmentExtract`.
- `zircon_runtime/src/scene/world/render.rs`: fill environment extract from viewport settings.
- `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs`: route preview-sky executor behavior through the formal environment settings until the full executor deletion slice runs.
- `zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/preview_sky_pass.rs`: read skybox state from `EnvironmentExtract` instead of owning standalone fallback-sky policy.
- `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs`: inject environment reflection code into standard PBR shading.
- `zircon_runtime/src/graphics/shader/template/material_surface.rs`: expose the same environment helper to standard material templates if this file currently owns the generated material path.
- `zircon_runtime/src/graphics/tests/project_render.rs`: mount `render_product_environment`.
- `docs/plans/zircon_runtime/render/11-environment-lighting.md`: append status rows for each completed slice and testing stage.
- `docs/zircon_runtime/core/framework/render/shader.md`: record related shader/environment implementation files and tests.

## Milestone 1: Environment Contract And Extraction

Goal: make environment state a formal render-frame contract without changing product visuals yet.

In-scope behaviors:

- `SkyboxSettings::none()` disables skybox and IBL.
- `SkyboxSettings::procedural_default()` represents the current preview-sky gradient as a formal environment source.
- `EnvironmentExtract::default()` is disabled for headless/minimal tests.
- `ViewportRenderSettings::default().preview_skybox == true` still produces the existing default preview sky through `EnvironmentExtract`.
- `IblBakeKey` changes for gradient colors and source revision, but not for intensity or rotation.

Dependencies:

- Existing `FallbackSkyboxKind` and `PreviewEnvironmentExtract` behavior in `camera.rs`, `scene_extract.rs`, and `frame_extract.rs`.

Implementation slices:

- [ ] Add `core/framework/render/environment` contract files and exports.
- [ ] Add unit tests for procedural defaults, disabled defaults, and bake-key stability.
- [ ] Add `environment: EnvironmentExtract` to frame and scene extraction defaults.
- [ ] Route viewport preview sky settings into `EnvironmentExtract` while preserving existing preview output behavior.
- [ ] Update docs and append one status row for each completed slice in this plan.

Testing stage:

- Compile command: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m1-contract-0704 --message-format short --color never`
- Unit command: `cargo test -p zircon_runtime --lib environment --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m1-contract-0704 --message-format short --color never -- --nocapture --test-threads=1`
- Debug loop: if compile fails in upper-layer scene/render tests, inspect `EnvironmentExtract` defaults before adjusting renderer code.
- Exit evidence: compile succeeds; environment unit tests pass; no product screenshot claim is made in this milestone.

## Milestone 2: Formal Skybox Renderer Path

Goal: make the visible skybox consume the formal environment contract.

In-scope behaviors:

- Procedural skybox draws from `EnvironmentExtract.skybox`.
- Disabled skybox leaves the background path unchanged.
- Preview-sky renderer calls no longer own fallback-sky policy; they read the formal environment state.
- The skybox remains an engine render path, not a product-test-only background fill.

Dependencies:

- Milestone 1 environment contract and extraction.

Implementation slices:

- [ ] Create `graphics/scene/scene_renderer/environment` module route and procedural sky shader.
- [ ] Route existing preview-sky executor through `SkyboxSettings` and `ProceduralSkyParams`.
- [ ] Update overlay preview sky pass to read `EnvironmentExtract`.
- [ ] Add product-level tests proving enabled sky differs from disabled sky and keeps visible object pixels.
- [ ] Update Plan 11 status rows and shader module docs.

Testing stage:

- Compile command: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m2-skybox-0704 --message-format short --color never`
- Unit/product command: `cargo test -p zircon_runtime --lib render_product_environment_skybox --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m2-skybox-0704 --message-format short --color never -- --nocapture --test-threads=1`
- Static scan: `git grep -n "FallbackSkyboxKind::ProceduralGradient" -- zircon_runtime/src/core zircon_runtime/src/graphics`
- Debug loop: if the static scan still reports non-test fallback-sky policy owners, either route them through `EnvironmentExtract` in this milestone or document the exact remaining owner in Plan 11 status.
- Exit evidence: skybox product tests pass; static scan has no new policy owner outside the compatibility route being actively retired.

## Milestone 3: PBR Environment Reflection Sampling

Goal: make standard PBR material shading use the environment path for metallic/smoothness response.

In-scope behaviors:

- Smoothness maps to roughness as `roughness = 1.0 - smoothness`.
- Roughness maps to an environment blur level monotonically.
- Metallic controls the reflected environment contribution; metallic 0 remains mostly diffuse, metallic 1 strongly reflects/tints the environment.
- The fallback mesh shader and standard material template consume the same environment helper.

Dependencies:

- Milestone 2 visible skybox source.
- Existing Plan 08 material shader path.

Implementation slices:

- [ ] Add `graphics/shader/includes/zr_environment.wgsl` with documented roughness/smoothness and reflection helpers.
- [ ] Add Rust-side roughness/smoothness mapping tests in `procedural_environment.rs`.
- [ ] Inject environment helper into `fallback_mesh_shader_source.rs`.
- [ ] Inject environment helper into `material_surface.rs` if generated standard material templates bypass fallback source.
- [ ] Add a focused shader/Naga test proving the environment helper is present in material pass source.
- [ ] Update Plan 11 and shader docs with implementation files and tests.

Testing stage:

- Compile command: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never`
- Unit command: `cargo test -p zircon_runtime --lib environment --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1`
- Shader command: `cargo test -p zircon_runtime --lib shader_environment --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1`
- Debug loop: if shader compilation fails, fix the shared include or generated source before changing product-test thresholds.
- Exit evidence: unit and shader tests pass; source assertions prove PBR material source consumes environment helper.

## Milestone 4: 8x8 PBR Matrix Product Artifact

Goal: generate the requested visual proof under `docs/tests/runtime/shader`.

In-scope behaviors:

- 8 columns cover metallic `0.0` through `1.0`.
- 8 rows cover smoothness `0.0` through `1.0`.
- The scene includes visible skybox/environment context.
- The generated PNG is written to `docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png`.
- No same-name PNG is written under repo `target` or external cargo target roots.

Dependencies:

- Milestone 3 environment reflection in PBR shading.

Implementation slices:

- [ ] Add `render_product_environment.rs` product test owner and mount it from `graphics/tests/project_render.rs`.
- [ ] Build a temporary project scene containing 64 sphere or bevelled-tile mesh/material pairs with distinct metallic/smoothness values.
- [ ] Add non-ignored validation test for visible skybox, visible 64-cell matrix, and monotonic image-signal changes along metallic and smoothness axes.
- [ ] Add ignored export test that writes `runtime_pbr_metallic_smoothness_matrix_20260704.png`.
- [ ] Update Plan 11, shader plan, and docs module status with artifact path, hash, dimensions, and validation commands.

Testing stage:

- Compile command: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m4-matrix-0704 --message-format short --color never`
- Product command: `cargo test -p zircon_runtime --lib render_product_environment --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m4-matrix-0704 --message-format short --color never -- --nocapture --test-threads=1`
- Export command: `cargo test -p zircon_runtime --lib graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_metallic_smoothness_matrix_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m4-pbr-matrix-0704 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1`
- Artifact check: `Get-FileHash docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png`
- Target scan: search repo `target`, `E:\cargo-targets`, `F:\cargo-targets`, and `D:\cargo-targets` for the same PNG name.
- Debug loop: if the image is nonblank but monotonic checks fail, inspect material parameter projection and environment helper first, then adjust camera/framing thresholds.
- Exit evidence: product test passes, ignored export writes the PNG, hash/dimensions are recorded, target scan returns zero matches.

## Milestone 5: Documentation And Completion Audit

Goal: make the implementation auditable against Plan 11 and the requested artifact.

In-scope behaviors:

- Plan 11 status table records each completed slice and testing stage.
- Shader/render docs list new environment/PBR files and exact tests.
- The implementation does not claim reflection probes, lightmaps, external cubemap assets, or fog are complete.
- Completion audit maps every user-visible requirement to evidence.

Dependencies:

- Milestones 1-4.

Implementation slices:

- [ ] Update `docs/plans/zircon_runtime/render/11-environment-lighting.md` status rows.
- [ ] Update `docs/plans/zircon_runtime/shader/index.md` or the specific shader subplan if the matrix is accepted as shader-material evidence.
- [ ] Update `docs/zircon_runtime/core/framework/render/shader.md` with related code, plan sources, and tests.
- [ ] Run final status scans for old preview-sky parallel policy, target-hosted artifact names, and PBR matrix artifact metadata.

Testing stage:

- Diff check: `git diff --check -- docs/plans/zircon_runtime/render/11-environment-lighting.md docs/plans/zircon_runtime/shader/index.md docs/zircon_runtime/core/framework/render/shader.md`
- Final focused command set: rerun the most recent successful Milestone 4 product and export commands.
- Completion audit: inspect current PNG, exact source paths, status rows, and command outputs before marking the active goal complete.
- Exit evidence: docs checks pass; Milestone 4 evidence remains current; no requirement is left with missing or indirect evidence.

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | Add `core/framework/render/environment` contract files and exports | 已验证 | 2026-07-04 | `zircon_runtime/src/core/framework/render/environment/{mod.rs,extract.rs,skybox.rs}`；`zircon_runtime/src/core/framework/render/mod.rs`；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m1-contract-0704 --message-format short --color never` 通过 |
| M1 | Add unit tests for procedural defaults, disabled defaults, and bake-key stability | 已验证 | 2026-07-04 | `cargo test -p zircon_runtime --lib core::framework::render::environment --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m1-contract-0704 --message-format short --color never -- --nocapture --test-threads=1`：6 passed |
| M1 | Add `environment: EnvironmentExtract` to frame and scene extraction defaults | 已验证 | 2026-07-04 | `RenderFrameExtract`、`SceneViewportRenderPacket`、`RenderFrameExtract::from_snapshot`、`RenderFrameExtract::to_scene_snapshot` 已接线；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m1-contract-0704 --message-format short --color never` 通过 |
| M1 | Route viewport preview sky settings into `EnvironmentExtract` while preserving preview output | 已验证 | 2026-07-04 | `zircon_runtime/src/scene/world/render.rs` 使用 `EnvironmentExtract::from_preview_skybox_enabled`，`PreviewEnvironmentExtract::from_environment` 保留旧 preview 字段输出；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m1-contract-0704 --message-format short --color never` 通过 |
| M1 | Update docs and append completed slice status rows | 已完成 | 2026-07-04 | `docs/superpowers/plans/2026-07-04-skybox-ibl-pbr-matrix.md`；`docs/plans/zircon_runtime/render/11-environment-lighting.md` |
| M2 | Create renderer environment module route and procedural sky shader | 已验证 | 2026-07-04 | `zircon_runtime/src/graphics/scene/scene_renderer/environment/{mod.rs,procedural_environment.rs}` 与 `environment/shaders/skybox_procedural.wgsl`；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m2-skybox-0704 --message-format short --color never` 通过 |
| M2 | Route preview sky pass and scene uniform through `EnvironmentExtract` | 已验证 | 2026-07-04 | `overlay/passes/preview_sky_pass.rs` 读取 `frame.environment().skybox.mode`；`SceneUniform` 上传 sky/env 参数；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m2-skybox-0704 --message-format short --color never` 通过 |
| M3 | Add shared `zr_environment.wgsl` include and roughness/smoothness helper coverage | 已验证 | 2026-07-04 | `zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl`；`cargo test -p zircon_runtime --lib graphics::shader::template --locked --jobs 1 --target-dir E:\cargo-targets\zircon-env-m3-pbr-0704 --message-format short --color never -- --nocapture --test-threads=1` 通过，21/21 |
| M3 | Inject environment reflection into fallback, standard forward, and deferred PBR paths | 已验证 | 2026-07-04 | `fallback_mesh.wgsl`、`zr_shading_standard_pbr.wgsl`、`deferred_lighting.wgsl` 调用 `zr_environment_pbr_indirect(...)`；`deferred::lighting_pipeline` 13/13、`fallback_mesh_shader_source` 14/14、`graphics::shader::template` 21/21 均通过 |
| M4 | Add 8x8 metallic/smoothness PBR matrix export scene and validation | 已验证 | 2026-07-04 | `zircon_runtime/src/graphics/tests/project_render/project_scenes.rs::export_runtime_shader_pbr_metallic_smoothness_matrix_png` 生成 64 个标准 PBR 球并启用 procedural skybox/environment；导出测试通过 1/1 |
| M4 | Write requested PNG under docs and confirm no target copy | 已验证 | 2026-07-04 | `docs/tests/runtime/shader/runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png`，1280x960，109556 bytes，SHA256 `E883A3BDF657025EAD16A7F39B1F8BE5D7FFCDA1FDEF0243A8636A05C217030D`；repo `target`、`E:\cargo-targets`、`F:\cargo-targets`、`D:\cargo-targets` 同名扫描 0 |
| M5 | Update Plan 11, shader plan, SH04 plan, and module docs | 已完成 | 2026-07-04 | `docs/plans/zircon_runtime/render/11-environment-lighting.md`、`docs/plans/zircon_runtime/shader/index.md`、`docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md`、`docs/zircon_runtime/core/framework/render/shader.md` 已记录完成项、测试命令、PNG path/hash 与后续未完成范围 |
| M5 | Update priority structure/review finding docs | 已完成 | 2026-07-04 | `docs/plans/engine-code-structure-convention.md` 记录 folder-backed environment/render shader owner 与 target artifact scan；`docs/plans/engine-code-review-findings-2026-06.md` 记录真实 PBR/environment 合约验证而非离线合成图 |
| M5 | Final artifact audit | 已验证 | 2026-07-04 | `runtime_shader_pbr_metallic_smoothness_matrix_skybox_20260704.png` 位于 `docs/tests/runtime/shader`，1280x960，SHA256 `E883A3BDF657025EAD16A7F39B1F8BE5D7FFCDA1FDEF0243A8636A05C217030D`；本计划明确 cubemap prefilter、reflection probes、lightmaps/probes、fog、RenderDoc capture 与更广 product/perf sweep 仍未由本轮完成 |
