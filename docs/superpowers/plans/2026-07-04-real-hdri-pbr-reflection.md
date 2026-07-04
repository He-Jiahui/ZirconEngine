# Real HDRI PBR Reflection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Download a real CC0 HDRI, route it through the engine environment/PBR path, and export a screenshot showing real-scene reflections under `docs/tests/runtime/shader`.

**Architecture:** Keep the full texture/cubemap asset pipeline out of this narrow slice because Plan 13 cubemap assets are still open. Add a small sampled equirectangular environment table to the existing environment extract/scene uniform, decode the downloaded HDR in the product test, and let skybox/PBR WGSL sample that real HDR-derived environment data.

**Tech Stack:** Rust 2021, `zircon_runtime`, WGPU/WGSL, `image` HDR decoder, Poly Haven CC0 HDRI, existing project-render offscreen screenshot path.

---

## Scope

This plan implements a real-HDRI validation path only:

- Download one CC0 HDRI from Poly Haven and record source/license/hash.
- Decode the HDR in a test/export path.
- Downsample it into a fixed low-resolution equirectangular sample table carried by `EnvironmentExtract`.
- Sample that table in skybox and PBR reflection WGSL.
- Export a real HDR reflection proof image under `docs/tests/runtime/shader`.

This plan does not implement the full Plan 13 cubemap asset importer, GPU `texture_cube` binding, GGX prefilter mip chain, reflection probe capture/blending, or SH irradiance bake.

## Files

- Modify `zircon_runtime/src/core/framework/render/environment/skybox.rs`: add sampled HDR/equirectangular environment settings and bake-key coverage.
- Modify `zircon_runtime/src/core/framework/render/environment/extract.rs`: expose constructors for sampled HDR environments.
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs`: add sampled environment table fields.
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs`: fill sampled HDR table and params.
- Modify `zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl`: mirror the uniform layout.
- Modify `zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl`: sample the HDR-derived equirectangular table.
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl`: make the visible skybox use the same sampled environment when selected.
- Modify `zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl`, fallback shader copies, and any test-only scene uniform WGSL structs that mirror scene layout.
- Modify `zircon_runtime/src/graphics/tests/project_render/project_scenes.rs`: add ignored real-HDRI export and validation helpers.
- Add downloaded HDR under `docs/tests/runtime/shader/assets/` and exported PNG under `docs/tests/runtime/shader/`.
- Update Plan 11, shader plan/module docs, and this plan status table.

## Milestone 1: Real HDRI Environment Samples

Implementation slices:

- [x] Add sampled equirectangular environment contract and defaults.
- [x] Add scene uniform fields and WGSL sampling helper.
- [x] Decode/download Poly Haven HDRI and build the sampled table in the export test.
- [x] Export a real-HDRI reflection screenshot and assert visible HDR-derived color variation.
- [x] Update docs/status rows and run validation.

Testing stage:

- `cargo fmt --package zircon_runtime -- --check`
- `cargo test -p zircon_runtime --lib core::framework::render::environment --locked --jobs 1 --target-dir E:\cargo-targets\zircon-real-hdri-reflection-0704 --message-format short --color never -- --nocapture --test-threads=1`
- `cargo test -p zircon_runtime --lib graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_real_hdri_reflection_png --locked --jobs 1 --target-dir E:\cargo-targets\zircon-real-hdri-reflection-0704 --message-format short --color never -- --ignored --exact --nocapture --test-threads=1`
- Hash and size checks for downloaded HDR and exported PNG.
- Same-name target scan for the exported PNG.

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | Download Poly Haven `lakes` 1K HDRI | 已验证 | 2026-07-04 | 下载 `https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/lakes_1k.hdr` 到 `docs/tests/runtime/shader/assets/polyhaven_lakes_1k.hdr`; API 记录 `size=1464859`, `md5=b615491d315a3d4e23bb09c2c96c9e03`;本地校验 `size=1464859`, MD5 `B615491D315A3D4E23BB09C2C96C9E03`, SHA256 `FAF3ECE79216E568A29F0D8FC176A795C66EB9C312C3CF3EE18D9AC04A71DECB` |
| M1 | Add sampled equirectangular environment contract and uniform layout | 已验证 | 2026-07-04 | `SampledEquirectangularEnvironment`、`SkyboxMode::SampledEquirectangular`、scene uniform `environment_sample_params`/`environment_samples`、`zr_scene_runtime.wgsl`/fallback/deferred/skybox WGSL layout 已接线；`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-real-hdri-reflection-0704 --message-format short --color never` 通过 |
| M1 | Add real HDRI reflection export path | 已验证 | 2026-07-04 | `export_runtime_shader_pbr_real_hdri_reflection_png` 解码 `polyhaven_lakes_1k.hdr`,下采样为 16x8 equirectangular environment table,用真实 HDRI 环境渲染 8x8 PBR 矩阵；direct generated binary `E:\cargo-targets\zircon-real-hdri-reflection-0704-server\debug\deps\zircon_runtime-0a7825d39d44b0c4.exe graphics::tests::project_render::project_scenes::export_runtime_shader_pbr_real_hdri_reflection_png --ignored --exact --nocapture --test-threads=1` 通过 1/1,输出 `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_reflection_20260704.png`,1280x960,132232 bytes,SHA256 `958E3B200EC56BCA16BF9596B1F05D872179F51CEB9A64925E10FC2D41792DEE`;同名 PNG 在 repo `target`、`E:\cargo-targets`、`F:\cargo-targets`、`D:\cargo-targets` 扫描 0 |
| M1 | Final docs/status verification | 已验证 | 2026-07-04 | `cargo fmt --package zircon_runtime -- --check` 通过；scoped `git diff --check` 通过；direct binary 复跑 `export_runtime_shader_pbr_real_hdri_reflection_png` 通过 1/1,5.38s；PNG 与 HDR hash 复核一致,同名 target 扫描仍为 0 |
