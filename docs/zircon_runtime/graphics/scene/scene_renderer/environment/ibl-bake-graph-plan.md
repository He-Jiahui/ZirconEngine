---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\scene_renderer\environment\ibl_bake_graph_plan.rs zircon_runtime\src\graphics\scene\scene_renderer\environment\mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never
  - cargo test -p zircon_runtime --lib ibl_bake_graph_plan --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1 (blocked before target tests by current UI UiStateFlags focused/hovered compile drift)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/mod.rs (2026-07-06 graph-execution compute-audit facade re-export: passed as part of docs/tests/runtime/text/runtime_text_spacing_cache_layout_rustfmt_check_20260706.log SHA256 E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855)
  - cargo test -p zircon_runtime text_measure_cache_reuses_shaped_runs_between_measure_and_layout --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-shaped-cache-0706 --message-format short --color never -- --nocapture --test-threads=1 (2026-07-06 compiled lib-test graph-execution/IBL test imports after the facade re-export and passed the filtered text regression 1/1; log docs/tests/runtime/text/runtime_text_spacing_cache_layout_focused_test_20260706.log SHA256 1DCD85CCAA299AF05AEA2CDCEEA331023D408473CF297356B8AA6114F16DE40A)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-shader-plan-check-0706 --message-format short --color never
  - cargo test -p zircon_runtime --lib ibl_bake_graph_plan --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-shader-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1 (passed 4/4)
doc_type: module-detail
---

# IBL Bake Render Graph Plan

## Purpose

`ibl_bake_graph_plan.rs` is the scene-renderer-local declaration point for Plan 11 / Shader 06 EC-M2s IBL bake graph work. It turns an `IblBakeArtifactRequest` into RenderGraph resources and async-compute pass metadata without registering a production executor or enabling frame-time bake execution.

The module is intentionally scoped under `graphics::scene::scene_renderer::environment` because it describes renderer graph topology: source cubemap input, derived PMREM texture output, SH9 buffer output, optional irradiance cubemap output, and the executor ids that future WGPU compute owners must implement.

## Graph Contract

- `append_ibl_bake_artifact_graph_plan(...)` imports `environment.ibl.source_cubemap` as a required external texture and marks it persistent for the graph boundary.
- PMREM requests create `environment.ibl.pmrem` as one `Rgba16Float` cubemap with `STORAGE`, `SAMPLED`, and `COPY_SRC` usage, plus readback marking for artifact acquisition.
- PMREM work is represented as one pass per mip: `env.ibl_prefilter.mip0`, `env.ibl_prefilter.mip1`, and so on up to the request mip count.
- SH9 requests create `environment.ibl.irradiance_sh9` as a storage/readback buffer sized to the artifact SH9 layout.
- IEM requests create `environment.ibl.irradiance_cube` as a 32x32x6 `Rgba16Float` cubemap with storage, sampling, copy-source, and readback usage.
- `IblBakeGraphPlan::passes` is appended in artifact order: all PMREM mip passes, SH9, then optional irradiance cubemap. The compiled graph may topologically interleave independent SH9/IEM work with PMREM mips, so tests check names and dependencies instead of compiled pass indices.
- Each pass reads the external source cubemap and writes exactly one storage output.

## Workloads

The pass metadata uses fixed labels and executor ids so later executor registration can bind real WGPU compute kernels without changing the artifact/graph contract:

- `env.ibl_prefilter.mipN` / `environment.ibl_prefilter` / `zircon-env-ibl-prefilter`
- `env.ibl_irradiance_sh` / `environment.ibl_irradiance_sh` / `zircon-env-ibl-irradiance-sh`
- `env.ibl_irradiance_cube` / `environment.ibl_irradiance_cube` / `zircon-env-ibl-irradiance-cube`

All three workloads use an 8x8x1 workgroup contract. PMREM dispatch is mip scoped: mip0 of a 128 face uses `[16, 16, 6]`, mip7 uses `[1, 1, 6]`, and every mip pass writes a WGPU D2Array storage view over the six cube faces. SH9 and IEM use fixed 32x32x6 small-grid dispatches suitable for the later reduction/downsample compute implementations.

The explicit PMREM mip dependency chain exists because the current RenderGraph resource model tracks whole textures, not subresources. WGPU will bind one storage view per mip, but the graph still sees the same cubemap as the output resource for every PMREM pass.

## Scheduling Boundary

This module does not execute compute work. It does not add default frame-path registration, WGPU bind groups, readback queue integration, cache writeback, or product second-launch dispatch proof. It is a dormant graph-plan contract that lets the next slice attach concrete WGPU command encoding and readback scheduling without reworking resource naming or artifact section layout.

## Verification

Counted evidence for the current slice:

- `rustfmt --edition 2021 --check` passed for `ibl_bake_graph_plan.rs` and `environment/mod.rs`.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never` passed.

The current per-mip PMREM contract rerun uses `E:\cargo-targets\zircon-ibl-shader-plan-check-0706`: `cargo test -p zircon_runtime --lib ibl_bake_graph_plan --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-shader-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1` passed 4/4 with repository-existing warnings.

On 2026-07-06, a runtime text focused rerun exposed a structural import gap in the compute audit facade while compiling the `zircon_runtime` lib-test target. `graph_execution/mod.rs` now re-exports `RenderGraphComputeWorkloadAuditStatus` from `render_graph_execution_record`, matching the existing `RenderGraphComputeDispatchRecord` facade and letting the IBL compute executor tests import audit status through the graph-execution boundary. This is a facade/wiring repair only: it does not add default IBL executor registration, WGSL kernels, GPU dispatch, readback scheduling, or product capture evidence.

Open gates remain: real WGPU bind group/storage view creation, PMREM/SH9/IEM compute command encoding, async scheduling and readback queue, cache writeback dispatch from GPU outputs, importer/staged artifact production, product second-launch dispatch=0 evidence, RenderDoc/product captures, strict roughness/SSIM/seam screenshots, optimized SH9 reduction, and full CI.
