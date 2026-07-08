---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_sh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_cube.wgsl
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_command_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_prefilter.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_sh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_cube.wgsl
plan_sources:
  - user: 2026-07-06 real HDRI PMREM mip blur and cmft/Unreal cubemap filtering correction
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - E:\cargo-targets\zircon-ibl-final-mip-average-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_dispatch --nocapture --test-threads=1
doc_type: module-detail
---

# IBL Bake Shader Plan

## Purpose

`ibl_bake_shader_plan.rs` is the renderer-local contract for the Plan 11 / Shader 06 GPU IBL bake kernels. It does not submit WGPU commands directly. It describes which WGSL kernel is used for each requested artifact family, which resources the kernel consumes, which storage output it writes, and which fixed dispatch extent should be used by the later command encoder.

The PMREM kernel writes one cubemap mip per dispatch. This is a WGPU constraint rather than a style choice: WGSL has no `texture_storage_cube`, and a storage texture view cannot dynamically select arbitrary mip levels from inside the shader. The graph and command plans therefore model PMREM as per-mip work that writes a `texture_storage_2d_array<rgba16float, write>` view with six array layers.

The follow-up WGPU command/readback plan lives in `ibl_bake_wgpu_command_plan.rs` and is documented in `ibl-bake-wgpu-command-plan.md`. That plan now owns bind-group layout entries, per-mip storage texture view descriptors, storage-buffer output descriptors, artifact-aware readback copy planning, and the WGSL source needed by the pipeline helper. `RenderGraphExecutionResources::owned_texture_view_with_descriptor(...)` can create the PMREM/IEM D2Array storage view from an owned Cube transient backing, `ibl_bake_wgpu_binding.rs` can create live params buffers, source samplers, bind group layouts, and bind groups, `ibl_bake_wgpu_pipeline_cache.rs` owns renderer-lifetime shader/pipeline reuse, and `ibl_bake_wgpu_dispatch.rs` can encode PMREM/SH9/IEM graph-context dispatches. Production scheduling, readback buffer ownership, cache writeback, and asynchronous readback draining are still open.

## PMREM Algorithm Contract

`shaders/ibl_prefilter.wgsl` mirrors the current CPU PMREM bridge in `source_cubemap/pmrem.rs` for the algorithm pieces that can be represented in a single per-mip shader:

- roughness-to-mip mapping uses the Unreal constants `ROUGHEST_MIP = 1.0` and `ROUGHNESS_MIP_SCALE = 1.2`,
- low roughness uses 32 samples, mid roughness uses 64 samples, and high roughness uses 128 samples,
- Hammersley samples are centered with `(index + 0.5) / sample_count`,
- GGX sampling applies the Unreal `E.y *= 0.995` grazing-angle guard,
- filtered importance sampling chooses the source cubemap mip from sample PDF,
- source texel solid angle includes the Unreal `* 2.0` scale,
- `roughness >= 0.99` uses cosine hemisphere convolution from the source mip pyramid instead of ordinary previous-PMREM downsampling,
- the final 1x1 mip performs a cmft-style six-face average by evaluating the cosine prefilter at each face axis and writing the same average color to all six output layers.

The CPU PMREM bridge performs its final six-face average after per-face filtering. The WGPU path keeps the same observable contract without a read/write feedback pass: only the final mip dispatch takes the six face axes, filters each from the source cubemap mip pyramid, averages them, and stores that shared radiance into every 1x1 face layer. This avoids using the destination PMREM texture as both sampled input and storage output in one pass.

## Other Kernels

`ibl_irradiance_sh.wgsl` and `ibl_irradiance_cube.wgsl` establish the SH9 and optional IEM storage ABI. They are intentionally kept behind the renderer-local IBL bake modules until real command encoding, async readback, and artifact cache writeback are connected. The current SH9 kernel is a deterministic starter contract, not the final optimized two-stage reduction.

## Verification

The focused verification for the current PMREM shader-plan slice was:

```powershell
$env:CARGO_TARGET_DIR='E:\cargo-targets\zircon-ibl-shader-plan-check-0706'; $env:CARGO_INCREMENTAL='0'; cargo test -p zircon_runtime --lib ibl_bake_shader_plan --no-default-features --features core-min --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1
```

Result after the final-mip update: 5/5 passed. The tests parse all three WGSL kernels through Naga, assert PMREM per-mip dispatch/resource binding, assert high-roughness sample count 128, and lock the PMREM WGSL contract markers for cosine convolution, PDF-driven source-lod selection, 2x texel solid-angle scale, centered Hammersley samples, the Unreal grazing-angle guard, and the final 1x1 six-face average branch.

The focused WGPU dispatch verification was also rerun directly from the generated test binary:

```powershell
E:\cargo-targets\zircon-ibl-final-mip-average-0706\debug\deps\zircon_runtime-9b53a08da5f6e0e7.exe ibl_bake_wgpu_dispatch --nocapture --test-threads=1
```

Result: 7/7 passed, 6995 filtered, including `final_pmrem_mip_writes_common_six_face_average`. That test writes an asymmetric source cubemap, dispatches final PMREM mip4, reads back `Rgba16Float` 1x1x6, and asserts all six faces match and contain nonzero radiance.

This verification does not claim production scheduling, async readback dequeue, runtime cache writeback, product second-launch dispatch=0, RenderDoc capture, screenshot SSIM/seam gates, or full workspace CI. The output-view bridge is covered separately by `materialization_exposes_owned_cube_storage_texture_array_views`, the live bind-group helper is covered by `ibl_bake_wgpu_binding`, and the live pipeline/command-encoding plus graph-context helper is covered by `ibl_bake_wgpu_dispatch`.
