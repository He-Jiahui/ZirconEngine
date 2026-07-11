---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/irradiance_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/irradiance_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
plan_sources:
  - user: 2026-07-06 implement WGPU-to-render-pipeline design from docs/plans/zircon_runtime/render
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - cargo test -p zircon_runtime --lib ibl_bake_wgpu_dispatch --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-wgpu-dispatch-0706 --message-format short --color never -- --nocapture --test-threads=1 (passed 4/4; 6984 filtered)
  - E:\cargo-targets\zircon-ibl-wgpu-dispatch-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_dispatch --nocapture --test-threads=1 (passed 4/4; log docs/tests/runtime/render/plan11_ibl_wgpu_dispatch_direct_tests_20260706.out.log)
  - cargo fmt --package zircon_runtime
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --message-format short --color never (passed)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_wgpu_dispatch --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (passed 6/6; 6986 filtered)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_wgpu_pipeline_cache --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (passed 1/1; 6992 filtered)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-dispatch-coverage-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_wgpu_dispatch --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (passed 6/6; 6987 filtered)
  - E:\cargo-targets\zircon-ibl-final-mip-average-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_dispatch --nocapture --test-threads=1 (passed 7/7; 6995 filtered; includes final 1x1 PMREM face-average readback)
  - E:\cargo-targets\zircon-ibl-wgpu-readback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_readback --nocapture --test-threads=1 (passed 4/4; readback resource bridge)
doc_type: module-detail
---

# IBL Bake WGPU Dispatch

## Purpose

`ibl_bake_wgpu_dispatch.rs` is the renderer-local WGPU compute pipeline, command-encoding, and focused graph-context bridge for Plan 11 / Shader 06 IBL baking. It sits after the pure command plan and live bind-group helper, and before the future production scheduler/readback/cache writer.

The module can still create a direct `wgpu::ComputePipeline` from the command plan's `wgsl_source`, `pipeline_label`, and the shared `IBL_BAKE_COMPUTE_ENTRY_POINT` for low-level tests. The graph-context path now requires the renderer-owned `IblBakeWgpuPipelineCache`, so live PMREM/SH9/IEM dispatches reuse shader modules, pipeline layouts, and compute pipelines instead of compiling per pass.

## Dispatch Encoding

`encode_ibl_bake_wgpu_compute_dispatch(...)` validates `command.dispatch_groups` before beginning a WGPU compute pass. A zero group is rejected as a renderer-local error instead of being left to backend validation. Valid commands encode:

- compute pass label from `command.pipeline_label`,
- pipeline binding,
- bind group 0,
- `dispatch_workgroups(x, y, z)` from the command plan.

The returned `IblBakeWgpuEncodedDispatch` is deliberately small: it records the pipeline label and dispatched groups for focused tests and later executor audit wiring.

## Graph Context Bridge

`record_ibl_bake_wgpu_pass_for_request(...)` is the focused bridge from `RenderPassExecutionContext` into the live WGPU helper. It selects the matching command from pass name and executor id, validates declared source/output access, resolves the imported source cubemap view and materialized output resource, creates params/sampler/bind group objects, obtains the compute pipeline from the renderer-owned IBL pipeline cache, encodes the dispatch into the pass encoder, and pushes a `RenderGraphComputeDispatchRecord`.

Current focused coverage locks all three graph-context output shapes through materialized render graph resources:

- PMREM mip0 resolves the owned Cube transient as a `texture_storage_2d_array<rgba16float, write>` view.
- SH9 resolves the owned transient storage buffer.
- IEM resolves its owned Cube transient as a single-mip `texture_storage_2d_array<rgba16float, write>` view.

Production scheduling, async readback draining, cache writeback, and product dispatch=0 proof remain follow-up gates. The renderer-local readback resource acquisition step now lives in `ibl_bake_wgpu_readback.rs`, so dispatch stays focused on command encoding and audit records.

## Verification

The focused WGPU test creates an offscreen backend, a source cubemap view, PMREM storage texture output, SH9 storage buffer output, params buffers, source sampler, bind groups, two direct compute pipelines, and one command encoder. It encodes and submits both the PMREM storage-texture path and SH9 storage-buffer path, then polls the device to completion.

The second test rejects a zero dispatch group before opening a WGPU pass.

The graph-context tests attach an `IblBakeWgpuPipelineCache` to the test GPU context and verify that materialized PMREM mip0, SH9, and IEM graph passes resolve source/output resources, record one compute dispatch audit entry, and submit successfully. The negative graph-context test still rejects a PMREM pass name that lacks the required `.mipN` suffix before GPU resource lookup.

The final PMREM mip test builds an asymmetric 16x16x6 source cubemap with five mip levels, dispatches PMREM mip4 into an `Rgba16Float` 1x1x6 storage output, reads the output back, and asserts every face receives the same nonzero value. This locks the WGPU shader's cmft-style final 1x1 six-face average instead of allowing the roughest mip to preserve per-face direction noise.

The reference-parity child tests exercise the production compute kernels rather than a test-only approximation:

- `render_env_prefilter_cpu_gpu_match_16` compares every RGB texel of all six faces and every fixed PMREM mip against the CPU FIS reference, with RGBA16F quantization applied to the common source.
- `render_env_sh9_matches_cpu_reference` compares all nine RGB coefficients against the CPU exact-solid-angle projection and verifies that a constant environment populates only band zero.
- `render_env_iem_matches_sh9_low_frequency` samples 64 Fibonacci-sphere directions and compares the cosine-convolved IEM against SH9 evaluation.

These tests exposed two production shader defects. The SH kernel used a Z-up coefficient ordering while the CPU/environment consumer contract is Y-up, and the IEM kernel multiplied its normalized cosine-weighted average by an additional `PI`. The shader now uses the CPU coefficient basis exactly and returns the normalized weighted average without the extra energy factor.

Current-source verification on 2026-07-11 passed `render_env_` 4/4, the complete `ibl_bake_` group 58/58, `source_cubemap::tests::` 16/16, `environment_brdf_lut::tests::` 4/4, and `ibl_bake_runtime_writeback::tests::` 4/4. The build used an isolated validation root because the shared root lock temporarily resolved `gpu-allocator` and `wgpu-hal` to incompatible `windows` crate versions; the isolated lock selected WGPU 29.0.4 with a consistent Windows ABI and did not modify the shared lock file.

## Open Issues

This helper closes shader-module creation, compute-pipeline creation, command-encoder dispatch, renderer-lifetime IBL pipeline cache use, focused PMREM/SH9/IEM graph-context dispatch/audit coverage, final PMREM 1x1 six-face average readback, CPU/GPU PMREM texel parity, SH9 coefficient parity, IEM/SH9 low-frequency parity, and runtime cache writeback coverage. Product second-launch dispatch=0, strict roughness/SSIM/seam evidence, multi-view captures, and RenderDoc evidence are recorded by Shader Plan 06. Remaining work outside this helper is optimized/two-stage SH9 reduction, 4K/16K offline bake coverage, editor-facing capture controls, and full shared-workspace CI after dependency-lock convergence.

## 2026-07-07 Test Owner Split

Status `runtime_15_production_file_budget_ui_ibl_project_owner_split_static_passed_cargo_check_offline_locked_blocked` moves the dispatch helper's large private test body out of `ibl_bake_wgpu_dispatch.rs` and into `ibl_bake_wgpu_dispatch/tests.rs`. The production file keeps command encoding, graph-context bridging, and pipeline-cache dispatch logic; the child test file owns the focused WGPU dispatch, zero-dispatch rejection, graph-context materialization, and final PMREM mip readback regressions. No compatibility module, shim, or re-export was introduced.

Verification passed scoped rustfmt, standalone structure-convention `production_file_budget` 104/104, and no-default-features runtime tests offline cargo check with warnings only. The locked Cargo gate is blocked by current non-slice `Cargo.lock` drift.
