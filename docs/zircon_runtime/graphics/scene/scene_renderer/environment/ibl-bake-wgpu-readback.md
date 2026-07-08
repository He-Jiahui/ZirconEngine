---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_ibl_bake_artifact_sections.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba16float_region.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_writeback.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
plan_sources:
  - user: 2026-07-06 continue real HDRI cubemap/PMREM correction and Shader 06 execution
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - cargo fmt --package zircon_runtime (passed)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-readback-0706 CARGO_INCREMENTAL=0 cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --message-format short --color never (first run exposed a test-only expect_err Debug bound; rerun passed after explicit match)
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-ibl-wgpu-readback-0706 CARGO_INCREMENTAL=0 cargo test -p zircon_runtime --lib ibl_bake_wgpu_readback --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (Cargo wrapper timed out after 904s while compiling)
  - E:\cargo-targets\zircon-ibl-wgpu-readback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_readback --nocapture --test-threads=1 (passed 4/4; 7022 filtered)
  - git diff --check -- touched IBL readback/doc files (passed with existing LF/CRLF warning only)
  - target/cargo-target shader/HDRI PNG scan (COUNT=0)
  - E:\cargo-targets\zircon-ibl-runtime-writeback-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_runtime_writeback --nocapture --test-threads=1 (passed 2/2; downstream runtime cache writeback bridge)
  - E:\cargo-targets\zircon-ibl-wgpu-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::backend::render_backend::read_ibl_bake_artifact_sections::tests::readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip --exact --nocapture --test-threads=1 (passed 1/1; backend PMREM texture readback seam guard)
  - E:\cargo-targets\zircon-ibl-live-pmrem-readback-seam-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams --exact --nocapture --test-threads=1 (passed 1/1; live PMREM graph output readback/cache seam guard)
  - E:\cargo-targets\zircon-ibl-live-iem-readback-0708\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe graphics::scene::scene_renderer::environment::ibl_bake_runtime_writeback::tests::runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance --exact --nocapture --test-threads=1 (passed 1/1; live IEM graph output readback/cache directional guard)
doc_type: module-detail
---

# IBL Bake WGPU Readback

## Purpose

`ibl_bake_wgpu_readback.rs` is the renderer-local acquisition bridge between materialized render graph outputs and the backend artifact readback helpers. The backend already knows how to read `Rgba16Float` cube mip chains and SH9 buffers, but it should not depend on scene-renderer resource names or render graph transient lifetime. This module owns that mapping.

The bridge is part of Shader 06 EC-M2 / Render 11 EL-M1. It moves the live GPU bake path closer to the planned three-source artifact flow:

- asset-derived bake artifact,
- runtime cache artifact from previous readback,
- live WGPU compute output read back into the same artifact payload layout.

## Behavior Model

`ibl_bake_wgpu_readback_resources_from_graph_resources(...)` takes an `IblBakeArtifactDescriptor` and a `RenderGraphExecutionResources` view. It examines `descriptor.contents()` and only requires graph outputs that the descriptor says will be written into the artifact.

- `PMREM` requires the owned transient texture named `environment.ibl.pmrem`.
- `SH9` requires the storage buffer named `environment.ibl.irradiance_sh9`.
- `IEM` requires the owned transient texture named `environment.ibl.irradiance_cube`.

The function returns `IblBakeArtifactWgpuReadbackResources`, which is the backend-facing readback packet consumed by `read_ibl_bake_artifact_wgpu_sections(...)`. Missing resources fail before backend readback begins, and the error includes the exact render graph resource name and expected role.

`read_ibl_bake_artifact_wgpu_sections_from_graph_resources(...)` is the thin integration entry for future scheduler code. It builds the backend readback packet, then calls the backend reader to produce `IblBakeArtifactReadbackSections`.

## Design and Rationale

The module lives in `graphics/scene/scene_renderer/environment` because resource-name knowledge belongs to the environment graph plan, not to the backend. The backend remains generic over raw WGPU textures and buffers. This mirrors the existing dispatch split:

- `ibl_bake_graph_plan.rs` declares graph resources and stable names.
- `ibl_bake_wgpu_dispatch.rs` records compute work into those graph resources.
- `ibl_bake_wgpu_readback.rs` rehydrates those graph resources into backend readback resources before transient backings are released.
- `ibl_bake_runtime_writeback.rs` consumes those sections when runtime compute was actually required and writes the artifact into the asset-layer runtime cache.
- `asset/artifact/ibl_bake_artifact_runtime_writeback.rs` remains the cache writer after sections become a validated artifact payload.

The bridge intentionally does not read uninitialized GPU data in its acquisition tests. This module's first responsibility is ownership and resource-lifetime correctness. Backend content correctness now has a separate WGPU texture readback seam guard: it writes a known seam-stress PMREM into a `Rgba16Float` cube texture, reads the texture and SH9 buffer through the same backend readback packet this bridge creates, verifies exact artifact bytes, then reapplies the payload and checks cube-edge seam statistics.

## Edge Cases and Constraints

- SH9-only descriptors are allowed by the readback bridge because the SH9 buffer is self-contained once a scheduler has an explicit request descriptor. This differs from the current opt-in executor request inference, which still needs PMREM metadata to recover face size and mip count.
- IEM descriptors use a single-mip irradiance cube texture with `SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE` and six array layers.
- Missing PMREM/SH9/IEM graph resources fail by name. This is important because runtime graph writeback must run before `release_transient_backings_into_pool(...)`.
- The bridge does not choose the production cache root or inject scheduler work into the product renderer. Runtime cache writeback from already-scheduled graph outputs is handled by `ibl_bake_runtime_writeback.rs`.

## Test Coverage

The focused unit tests materialize an IBL bake render graph on an offscreen WGPU backend, then verify that full `PMREM_SH9_IEM` descriptors resolve all required readback resources and SH9-only descriptors do not require PMREM or IEM. Negative coverage verifies that an empty graph resource set reports the missing PMREM resource by stable graph name. IEM descriptor coverage locks the single-mip, six-face irradiance cube shape.

The validation commands listed in the header are the scoped checks for this slice. The first `cargo check` run caught a test-only `expect_err` issue because the backend readback resource packet intentionally does not implement `Debug`; the test was corrected to an explicit `match`, then the same scoped check passed.

EC-M3x adds backend WGPU PMREM texture content coverage beneath this graph-resource bridge. `readback_sections_preserve_pmrem_seams_after_wgpu_texture_roundtrip` writes the synthetic seam-stress PMREM artifact bytes into a real WGPU cube texture using padded rows, reads the texture back through `read_ibl_bake_artifact_wgpu_sections(...)`, requires exact payload byte equality, and verifies applied mid/rough seam metrics remain stable. The direct final lib-test binary passed 1/1 in 1.81s after Cargo wrapper attempts timed out during compile/link.

EC-M3y extends that coverage to live compute-produced PMREM graph output. `runtime_graph_writeback_reads_pmrem_graph_output_and_preserves_readback_seams` records the actual PMREM WGPU graph passes against a synthetic seam-stress source cubemap, reads the graph-owned `environment.ibl.pmrem` texture through this bridge, writes the runtime cache blob, resolves the next dispatch from cache with zero runtime compute, decodes the PMREM payload, and verifies mid/rough seam metrics are reduced relative to mip 0. The direct final lib-test binary passed 1/1 in 3.28s after the Cargo wrapper timed out during the cold compile.

EC-M3z extends the same live graph-output coverage to IEM. `runtime_graph_writeback_reads_iem_graph_output_and_preserves_directional_irradiance` records the actual irradiance-cube WGPU graph pass against a low-frequency directional source cubemap, reads the graph-owned `environment.ibl.irradiance_cube` texture through this bridge, writes the runtime cache blob, resolves the next dispatch from cache with zero runtime compute, decodes the IEM payload, and verifies non-black directional variation plus normalized correlation against the CPU cosine-convolution reference. The direct final lib-test binary passed 1/1 in 8.14s after the Cargo wrapper timed out during the cold compile.

## Open Issues

This bridge closes graph-resource acquisition for WGPU IBL artifact readback, and the backend now guards PMREM texture readback byte layout plus seam preservation. The downstream runtime graph writeback bridge reads scheduled SH9, live PMREM, and live IEM graph output before transient release and proves runtime-cache second resolve `dispatch=0` in focused tests. Remaining Shader 06 EC-M2/EC-M3 gates are scheduler-owned product dispatch proof, production cache-root selection, optimized SH9 reduction, BRDF/IEM product binding, product second-launch `dispatch=0`, screenshot/source-reference gates, RenderDoc/product capture, 4K/16K offline bake, importer/staged artifact parity, and full CI.
