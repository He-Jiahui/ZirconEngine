---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/reports.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/reports.rs
plan_sources:
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\scene_renderer\environment\ibl_bake_compute_executor.rs zircon_runtime\src\graphics\scene\scene_renderer\environment\ibl_bake_graph_plan.rs zircon_runtime\src\graphics\scene\scene_renderer\environment\mod.rs zircon_runtime\src\graphics\scene\scene_renderer\graph_execution\render_pass_execution_context\gpu\reports.rs zircon_runtime\src\graphics\scene\scene_renderer\graph_execution\render_pass_executor_registry.rs zircon_runtime\src\graphics\scene\scene_renderer\graph_execution\render_pass_executor_registry\tests\registry_contracts.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never
  - cargo test -p zircon_runtime --lib ibl_bake_compute_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-shader-plan-check-0706 --message-format short --color never
  - cargo test -p zircon_runtime --lib ibl_bake_compute_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-shader-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1 (passed 4/4)
  - cargo fmt --package zircon_runtime
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-compute-executor-wgpu-0706 CARGO_INCREMENTAL=0 cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --message-format short --color never (passed)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-compute-executor-wgpu-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_compute_executor --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (passed 6/6; 6984 filtered)
  - cargo test -p zircon_runtime --lib ibl_bake_wgpu_request --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-request-owner-coremin-0706 --color never -- --nocapture --test-threads=1 (passed 2/2)
  - cargo test -p zircon_runtime --lib ibl_bake_request --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-request-owner-coremin-0706 --color never -- --nocapture --test-threads=1 (passed 2/2)
  - E:\cargo-targets\zircon-ibl-runtime-writeback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_runtime_writeback --nocapture --test-threads=1 (passed 2/2; downstream graph output cache writeback)
doc_type: module-detail
---

# IBL Bake Compute Executor

## Purpose

`ibl_bake_compute_executor.rs` is the opt-in executor registration and graph-to-WGPU bridge for Plan 11 / Shader 06 IBL bake passes. It connects the IBL bake graph plan to `RenderPassExecutorRegistry`, the existing `RenderGraphExecutionRecord` audit path, and the renderer-local WGPU dispatch helper.

The module belongs beside `ibl_bake_graph_plan.rs` because both files describe the renderer-side bake contract: graph resources first, executor ids second, and live WGPU dispatch only after the graph can prove source/output ownership.

## Contract

- `ibl_bake_compute_executor_registrations()` returns explicit registrations for `environment.ibl_prefilter`, `environment.ibl_irradiance_sh`, and `environment.ibl_irradiance_cube`.
- `RenderPassExecutorRegistry::with_environment_ibl_bake_compute_executors()` registers those executors only when an owner opts in. The builtin product registry still excludes the IBL bake ids.
- Each executor verifies that the pass reads `environment.ibl.source_cubemap` and writes its expected output resource.
- PMREM dispatch groups are derived from the compiled graph resource metadata plus the `.mipN` pass-name suffix. A PMREM pass without that suffix fails closed instead of guessing a mip.
- SH9 uses the fixed 4x4x6 reduction shape; IEM uses its 32x32x6 texture shape.
- `RenderPassGpuExecutionContext::push_compute_dispatch_record(...)` accepts the prebuilt dispatch record so resource accesses can be preserved in the execution record.
- When a GPU execution context is present, graph metadata contributes only the requested artifact contents: PMREM, SH9, IEM, or a combination of those resources.
- The bake key, source face size, and mip count come from the current frame `EnvironmentExtract::source_cubemap_ibl_bake_request(...)`, which delegates to `SourceCubemapEnvironment::ibl_bake_artifact_request(...)`.
- SH9-only and IEM-only graphs no longer need a PMREM output texture just to infer the source shape. They still need a current source-cubemap frame environment and at least one declared IBL output resource.

The 2026-07-06 compile-maintenance update keeps the PMREM negative test aligned with current constants by using the executor id as the invalid pass name when checking the required `.mipN` suffix. This changes only the test fixture input; executor registration ids and runtime dispatch planning behavior are unchanged.

## Boundary

This is not the complete production GPU bake scheduler yet. With a GPU context and enough graph metadata it can enter the live WGPU helper, which creates bind groups, storage views, compute pipelines, and dispatch commands for the matched pass. The source cubemap frame environment is now the request shape owner. Downstream graph-output runtime cache writeback now lives in `ibl_bake_runtime_writeback.rs`, but this executor still does not own scheduler queue storage, production cache-root selection, product graph injection, product dispatch=0 proof, or RenderDoc/product capture.

The default renderer path does not register the IBL bake executors. That is intentional until the PMREM/SH9/IEM kernels and readback queue are implemented.

## Verification

Counted evidence:

- Targeted `rustfmt --edition 2021 --check` passed for the IBL executor/graph files, registry file, registry test file, and GPU report helper.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never` passed.
- `cargo test -p zircon_runtime --lib ibl_bake_compute_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-graph-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1` passed 3/3 with 6902 filtered.
- Current per-mip PMREM rerun: `cargo test -p zircon_runtime --lib ibl_bake_compute_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-shader-plan-check-0706 --message-format short --color never -- --nocapture --test-threads=1` passed 4/4 with repository-existing warnings.
- Current WGPU-executor request inference check: `cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-compute-executor-wgpu-0706` passed.
- Current focused executor rerun: `cargo test -p zircon_runtime --lib ibl_bake_compute_executor --no-default-features --features core-min --locked --jobs 1` with the same target dir passed 6/6 with 6984 filtered.
- Current explicit request-owner rerun: `cargo test -p zircon_runtime --lib ibl_bake_wgpu_request --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-request-owner-coremin-0706 --color never -- --nocapture --test-threads=1` passed 2/2, covering full PMREM+SH9+IEM request rebuild and SH9-only rebuild without PMREM metadata.
- Current source request API rerun: `cargo test -p zircon_runtime --lib ibl_bake_request --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-request-owner-coremin-0706 --color never -- --nocapture --test-threads=1` passed 2/2, covering `SourceCubemapEnvironment` and `EnvironmentExtract` request shape ownership.

The later PMREM cosine-tail focused lib-test build also compiled this executor test module after the stale pass constant reference was removed; that run is counted only as compile coverage for this maintenance edit, not as a new executor functional run.

Open gates remain: production scheduler queue ownership around these requests; production cache-root injection; staged/importer artifact production; product second-launch dispatch=0 evidence; RenderDoc/product capture; optimized SH9 reduction; strict roughness/SSIM/seam screenshots; and full CI. The renderer-side graph resource acquisition bridge lives in `ibl_bake_wgpu_readback.rs`, and the runtime cache writeback closeout bridge lives in `ibl_bake_runtime_writeback.rs`; neither makes the executor production-scheduled by itself.
