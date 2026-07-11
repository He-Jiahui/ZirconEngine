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
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/reference_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests/irradiance_parity.rs
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

`ibl_irradiance_sh.wgsl` and `ibl_irradiance_cube.wgsl` implement the SH9 and optional IEM storage ABI used by real graph dispatch, readback, and artifact cache writeback.

The SH basis is the engine's Y-up ordering from `source_cubemap.rs`: `[L00, L1z, L1y, L1x, L2xz, L2zy, L2(3y^2-1), L2xy, L2(x^2-z^2)]`. GPU coefficient order must remain byte-for-byte compatible with CPU SH evaluation and scene-uniform consumption. The IEM kernel performs cosine-weighted hemisphere sampling and divides by the accumulated cosine weight; it does not apply a second `PI`, because the CPU reference and consumer contract store normalized diffuse irradiance radiance rather than the unnormalized hemisphere integral.

The current SH9 kernel remains a deterministic single-dispatch implementation rather than the final optimized two-stage reduction, but its output is now numerically checked against the CPU exact-solid-angle reference.

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

## 2026-07-11 CPU/GPU Reference Parity

Current-source WGPU verification added three non-ignored reference tests under the dispatch test owner:

- `render_env_prefilter_cpu_gpu_match_16`: all PMREM RGB texels, faces, and mips match the CPU FIS reference within `0.006` after common RGBA16F source quantization.
- `render_env_sh9_matches_cpu_reference`: all SH9 RGB coefficients match within `0.004`; constant input leaves coefficients 1 through 8 below `0.0005`.
- `render_env_iem_matches_sh9_low_frequency`: 64 sphere directions match within `0.055`.

The final run passed `render_env_` 4/4 and the full `ibl_bake_` group 58/58. This closes the Shader Plan 06 small-size GPU/offline PMREM, SH constant-band, and IEM/SH9 low-frequency gates. The existing product records separately own the 8x8 matrix, roughness monotonicity, seam, multi-view, and RenderDoc evidence.
