---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - cargo fmt --package zircon_runtime (passed)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-runtime-writeback-0706 CARGO_INCREMENTAL=0 cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --message-format short --color never (passed)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-runtime-writeback-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_runtime_writeback --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (Cargo wrapper timed out while compiling; background compile completed)
  - E:\cargo-targets\zircon-ibl-runtime-writeback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_runtime_writeback --nocapture --test-threads=1 (passed 2/2; 7053 filtered)
  - rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\scene_renderer\environment\ibl_bake_runtime_writeback.rs (passed)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-live-pmrem-readback-seam-0708 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams --no-default-features --features core-min --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1 (Cargo wrapper timed out after 1204s while compiling; background compile completed)
  - E:\cargo-targets\zircon-ibl-live-pmrem-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams --exact --nocapture --test-threads=1 (passed 1/1; 7310 filtered; test body 3.28s)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-live-iem-readback-0708 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance --no-default-features --features core-min --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1 (Cargo wrapper timed out after 1204s while compiling; background compile completed)
  - E:\cargo-targets\zircon-ibl-live-iem-readback-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance --exact --nocapture --test-threads=1 (passed 1/1; 7312 filtered; test body 8.14s)
doc_type: module-detail
---

# IBL Bake Runtime Writeback

## Purpose

`ibl_bake_runtime_writeback.rs` is the renderer-local closeout bridge for runtime IBL artifact production. Before render graph transient backings are released, it registers materialized graph outputs with the product diagnostic tail. After a successful scene submission, it retains only a CPU pending aggregator and reservation; the next frame's single backend completion poll delivers the sections before the asset-layer runtime cache writer runs.

This keeps ownership split by layer:

- `ibl_bake_graph_plan.rs` owns stable graph resource names and readback roots.
- `ibl_bake_wgpu_dispatch.rs` writes PMREM, SH9, or IEM outputs into graph resources.
- `ibl_bake_wgpu_readback.rs` maps those graph resources to backend product diagnostic section requests.
- `ibl_bake_runtime_writeback.rs` decides whether the dispatch report needs runtime compute output, prepares the CPU-only pending aggregation, commits it only after scene submission succeeds, and calls the asset runtime writeback helper after all callbacks terminate.
- `asset/artifact/ibl_bake_artifact_runtime_dispatch.rs` owns runtime cache hit/miss and second-dispatch `dispatch=0` semantics.

## Contract

`IblBakeRuntimeGraphWritebackQueue::prepare(...)` takes the backend product diagnostic owner, an explicit `IblBakeArtifactRequest`, an `IblBakeArtifactCacheStore`, the reservation, and current `RenderGraphExecutionResources`. The queue is bounded to four pending artifacts and rejects a duplicate request before registering new GPU copies.

The descriptor is rebuilt from the request. The graph resource shape is not used as the request owner. This preserves the Shader 06 rule that the current frame source cubemap owns bake key, face size, mip count, and required contents.

If `dispatch.requires_runtime_compute()` is false, `prepare(...)` returns no pending writeback without touching graph resources. This avoids false failures on asset-derived or runtime-cache hits where no transient bake resources need to exist.

If runtime compute is required, `prepare(...)` registers requested graph outputs through `prepare_ibl_bake_artifact_wgpu_readback_from_graph_resources(...)`. `commit_submitted(...)` moves that state into the queue only after the scene ticket exists. At the next frame begin, the backend's sole completion poll first routes all section callbacks; `poll_completed()` then assembles ready sections and calls `write_ibl_bake_artifact_runtime_dispatch_readback(...)`. A current descriptor writes the `.zribl` runtime cache blob. A stale descriptor is skipped by the asset layer. No product writeback method owns `wgpu::Buffer`, `map_async`, `device.poll`, `queue.submit`, or a separate command buffer.

`write_ibl_bake_runtime_cache_from_graph_resources(...)` remains under `#[cfg(test)]` as the synchronous historical fixture path. It is not part of the product completion model.

## Verification

Current 2026-08-27 source evidence confirms `artifact registration -> output writeback -> diagnostic tail -> query tail -> scene submit`, then `scene submit -> pending ownership commit -> transient retirement`. The following frame performs the sole backend completion poll before `poll_completed()` and typed query collection. The production writeback segment contains no native buffer, map, submit, command-buffer extraction, or device poll. Scoped formatting and diff checks pass; Cargo, real WGPU, PNG, RenderDoc, profiler, and power validation for this cutover remain pending.

The focused tests cover both control paths:

- A cache-hit dispatch skips graph readback even with an empty `RenderGraphExecutionResources`, proving cache/asset hits do not accidentally require transient GPU resources.
- A runtime-compute SH9 graph dispatch writes the SH9 storage buffer through WGPU, reads it before transient release, writes the runtime cache blob, then resolves the same request a second time with `environment_compute_dispatch_count() == 0`.
- A runtime-compute PMREM graph dispatch builds a synthetic seam-stress source cubemap, records every live PMREM compute graph pass, reads the produced `Rgba16Float` graph output through the runtime cache writeback bridge, resolves the next dispatch from runtime cache with `environment_compute_dispatch_count() == 0`, decodes the PMREM payload, and verifies mid/rough cube-edge luma seam energy is reduced from the base mip.
- A runtime-compute IEM graph dispatch builds a low-frequency directional source cubemap, records the live irradiance-cube WGPU graph pass, writes the produced `environment.ibl.irradiance_cube` output through the runtime cache bridge, resolves the next dispatch with `environment_compute_dispatch_count() == 0`, decodes the IEM payload, and verifies its normalized luma response remains correlated with the CPU cosine-convolution reference.

The SH9, PMREM, and IEM tests use the offscreen WGPU backend, a materialized IBL bake graph, live compute dispatch helpers, and the real asset runtime cache store under temporary directories. The PMREM test binds all source cubemap mips so roughness-dependent shader sampling validates the same source/PMREM mip contract used by the runtime renderer. The IEM test intentionally compares normalized directional response instead of exact bytes because the WGPU path uses the planned 64-sample cosine hemisphere shader while the CPU reference uses direct solid-angle convolution.

## Open Issues

This module closes the renderer-local asynchronous GPU output to runtime cache writeback owner path for already-scheduled SH9, PMREM, and IEM graph outputs. It does not yet choose the production cache root, close product consumption/second-launch `dispatch=0` evidence, produce current-source RenderDoc captures, or close the full Shader 06 screenshot/SSIM/seam, performance/power, and 4K/16K offline bake gates.
