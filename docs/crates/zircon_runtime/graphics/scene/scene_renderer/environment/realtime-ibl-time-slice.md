---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_gpu_timestamps.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_capture_wgpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_sh.wgsl
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
plan_sources:
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs
  - zircon_runtime/tests/runtime_shader_pbr_realtime_ibl_export.rs
---

# Realtime IBL time slicing

## Owner and scope

`realtime_ibl_time_slice.rs` owns the frame-to-frame scheduling contract for procedural-sky IBL rebakes. It does not own sky rendering, render-graph resource allocation, GPU submission, artifact readback, or cache persistence. Those remain with the existing environment capture, IBL compute, and runtime writeback owners.

The scheduler follows the real-time sky capture sequence in Unreal's `ReflectionEnvironmentRealTimeCapture.cpp` while reusing Zircon's GGX FIS PMREM and SH9 kernels. It never introduces a second filtering algorithm.

## Buffer and publication contract

- Slot A/B form a ready/work pair.
- The renderer samples only the ready slot while all slices write the work slot.
- A completed generation cycle atomically publishes the work slot and swaps A/B.
- A parameter change increments the generation, discards unfinished work, and preserves the published environment.
- A late GPU completion carrying an older generation token is `Stale` and cannot publish.
- A failed GPU slice is `Retry`; state and published resources remain unchanged.
- Re-requesting the currently published key cancels obsolete pending work without rebuilding it.

The first environment has no valid ready slot, so it is emitted as one complete batch. Later rebakes use time slicing.

## Default ticket schedule

With the default two capture faces per frame and eight PMREM mips, one update
cycle occupies 21 physical batches. Each batch contains one operation:

| Batches | Work | Count |
|---|---|---:|
| 1-3 | Capture sky faces 0-1, 2-3, 4-5 | 3 |
| 4-10 | Generate source cubemap mips 1-7 | 7 |
| 11-13 | PMREM mip 0, faces 0-1, 2-3, 4-5 | 3 |
| 14-20 | PMREM mips 1-7, all faces | 7 |
| 21 | Project diffuse SH9 and publish | 1 |

There is no `CaptureCloud` scheduler operation. A shorter mip chain reduces
the source-mip and PMREM rows without emitting zero-length dispatches.

## Parameterized PMREM dispatch

`RealtimeIblFrameBatch::prefilter_dispatch_slices()` expands a mip range into one dispatch descriptor per mip. `ibl_bake_wgpu_prefilter_command_for_slice(...)` then derives a command from the normal artifact-bake command plan and changes only:

- `dispatch_groups.z` to the requested face count;
- PMREM uniform word 5 to `first_face`;
- readback copies to empty, because partial slices cannot be persisted as a complete artifact.

`ibl_prefilter.wgsl` computes the cubemap face as `first_face + global_id.z`. The storage view remains the complete six-layer mip view, so the global face index is used consistently for direction reconstruction and `textureStore`.

## Frame integration requirements

The render owner must call `request_rebake` when procedural sky parameters change, call `begin_frame` at most once per frame number, execute operations in order, and call `complete_frame` only after submission success or failure is known. Source mip generation and SH9 remain parameterized operations in the same batch contract.

`RealtimeIblRuntime` now owns these calls for both compiled and direct scene render paths. A first procedural environment records the complete batch before scene draws and samples the work slot in that same command buffer. Later updates keep the scene bind group on the published ready slot while slices write the other slot.

## Render graph A/B contract

`realtime_ibl_graph_plan.rs` maps one `RealtimeIblFrameBatch` to render-graph passes without changing the scheduler state machine. Both slots are imported every frame as persistent external resources. Texture names are view aliases over one allocation per texture, not independent textures:

| Slot resource/view | A name pattern | B name pattern |
|---|---|---|
| Source full sampled cube | `environment.realtime_ibl.a.source.sampled` | `environment.realtime_ibl.b.source.sampled` |
| Source single-mip sampled cube | `environment.realtime_ibl.a.source.sampled.mipN` | `environment.realtime_ibl.b.source.sampled.mipN` |
| Source single-mip storage array | `environment.realtime_ibl.a.source.storage.mipN` | `environment.realtime_ibl.b.source.storage.mipN` |
| PMREM full sampled cube | `environment.realtime_ibl.a.pmrem.sampled` | `environment.realtime_ibl.b.pmrem.sampled` |
| PMREM single-mip storage array | `environment.realtime_ibl.a.pmrem.storage.mipN` | `environment.realtime_ibl.b.pmrem.storage.mipN` |
| Diffuse SH9 buffer | `environment.realtime_ibl.a.sh9` | `environment.realtime_ibl.b.sh9` |

The ready slot is imported for downstream sampling but is never declared as a write target. Every capture, source-mip, PMREM, and SH9 write targets only the work slot selected by the scheduler batch. Publication therefore remains an explicit post-submit operation instead of becoming an accidental graph alias.

The pass sequence is generated directly from the batch operations:

- `CaptureSky` writes the requested face range of the work source cubemap.
- `GenerateSourceMips` expands into one pass per destination mip. Each pass reads only the previous mip through a single-mip `Cube` view and writes only the destination through a single-mip `D2Array` storage view. This keeps sampled and writable subresources non-overlapping under WGPU validation.
- Each PMREM mip slice becomes a separate pass. It reads the work source cubemap, writes the work PMREM cubemap, and preserves `first_face` plus `face_count` in its workload.
- `ProjectDiffuseSh9` reads the work source cubemap, writes the work SH9 buffer, and depends on the last generated operation.

All passes use the async-compute queue contract and explicit dependencies. `RealtimeIblGpuResources` allocates exactly two source cubes, two PMREM cubes, and two SH9 buffers, then binds graph aliases to views of those allocations. The allocations and every A/B view remain resident, but graph materialization binds only names present in the current `CompiledRenderGraph::resource_lifetimes()`. A time-sliced graph intentionally omits unused face/mip aliases; binding all resident aliases would create stale external-resource bindings outside the compiled lifetime set. `RealtimeIblWgpuRecorder` records the graph pass order: dedicated procedural capture and source-mip kernels feed the existing GGX PMREM and SH9 pipeline cache. SH9 reuse clears the work buffer before projection.

The graph contract itself does not publish the slot. `SceneRendererCore` prepares and records it before scene passes, submits one command buffer, and only then gives the generation token back to `RealtimeIblRuntime::complete_submission`. Failed recording returns the batch as `Retry`; stale generations cannot publish.

## Scene sampling contract

Procedural real-time IBL uses scene source kind `4`. This is distinct from imported source cubemap kind `3` so the material path can select the generated PMREM and the matching generated SH9 coefficients from one published A/B slot.

- binding 1 samples the selected A/B source cube, including skybox output;
- binding 4 samples the same slot's PMREM cube for standard-PBR reflection;
- binding 6 reads the same slot's 144-byte SH9 allocation as a fixed uniform buffer for diffuse irradiance;
- the scene uniform publishes source size 128, PMREM size 128, and 8 PMREM mips;
- the first complete batch samples the work slot because it is written before draws in the same command buffer;
- later batches sample only the ready slot and swap the bind group after successful publication;
- intensity and rotation are final-sampling parameters and stay outside the runtime bake key, matching the offline artifact identity; capture stores unrotated, unscaled radiance so scene sampling applies each parameter exactly once.
- the procedural source can optionally carry a directional sun disk. Its direction, radiance color, intensity, and angular radius participate in the bake identity, are captured into the source cube before mip/PMREM/SH9 work, and are also drawn by the analytical skybox consumer. Final sky intensity and Y rotation still stay outside the bake identity.

The SH9 A/B buffers carry both `STORAGE` and `UNIFORM` usage. The compute pass writes the work slot through its storage binding; after successful publication, the scene bind group selects that same allocation through binding 6. Offline source cubemaps use a renderer-owned uniform buffer populated from their artifact SH9 coefficients. `SceneUniform` therefore contains no embedded SH array, and realtime diffuse lighting never performs a CPU readback or synchronous queue wait.

## SH9 parallel reduction

The SH9 projection uses one `8x8x1` workgroup and one `1x1x1` dispatch. All 64 invocations divide the existing face-major, exact-solid-angle cubemap sample set by striding over linear sample indices. Each invocation accumulates all nine RGB coefficients locally, writes one lane into workgroup memory, and participates in a six-step tree reduction before lane zero applies normalization and the diffuse band factors.

This preserves the offline and realtime SH9 coefficient ABI and the same cubemap integration math. It replaces the previous implementation where only global invocation zero traversed every sample while thousands of dispatched invocations returned immediately. The structure follows Unreal's `ComputeSkyEnvMapDiffuseIrradianceCS` single-group reduction, while retaining Zircon's exact cubemap sample set so offline CPU/GPU parity does not split into a second approximation.

The single-group extent is one contract across all owners. The shader plan, offline render-graph workload, metadata-only compute executor, WGPU command encoding, and realtime recorder all report `1x1x1` for SH9. The optional irradiance-cube kernel remains a spatial `4x4x6` dispatch; the two kernels must not share one inferred extent merely because both consume the same 32x32 sample size.

## Validation

- Current scheduler tests define the exact 21-batch default sequence, stale generations, retry behavior, short mip chains, cancellation, and dispatch expansion. The managed lib-test validation is currently blocked before test execution by the open Runtime74 UI compile failure.
- PMREM command tests cover face offset serialization, face-count dispatch reduction, invalid ranges, and WGSL ABI text.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min` passes with the Windows target under `E:\cargo-targets`.
- The standalone executable harness below is historical evidence for the earlier 16-frame topology; it does not attest the current 21-batch schedule.
- At the scheduler-only checkpoint, the normal focused lib-test command was blocked by unrelated missing diagnostics symbols in `zircon_runtime/src/tests/prelude.rs`; a temporary cfg experiment only established that the crate test target compiled after excluding that owner and was reverted because it selected zero scheduler tests. No EC-M4 visual pass was claimed from that scheduler-only slice.
- The SceneRendererCore integration check passed on Windows: `cargo check -p zircon_runtime --lib --no-default-features --features core-min` finished successfully with the target under `E:\cargo-targets\zircon-shader06-ecm4-runtime-20260712`.
- The final-source graphics integration export compiled on Windows. The exact contract test passed 1/1, the direct-SH9 ignored 8x8 product export passed 1/1 in 50.10 seconds, and the ignored five-view regression product export passed 1/1 in 94.07 seconds.
- The following 2026-07 measurements are historical baselines for the earlier 16-batch topology; they do not attest the current 21-batch schedule.
- The 1600x1200 direct-SH9 PBR result, CPU wall times, and 17 nonzero WGPU timestamp-query samples are recorded below `docs/tests/runtime/shader`. Initial full publication measured 4.981760 ms GPU; the sixteen sliced batches averaged 0.321728 ms and peaked at 4.472832 ms in the SH9 projection slice.
- The historical export renders a presentation-only frame after state 11 publishes the work slot. This prevents the accepted screenshot from accidentally sampling the previous ready slot; the final image differs from the retained pre-SH9 image by RGB channel MAE 27.999296.
- A five-view perspective product export covers front, pitch +/-120 degrees, and yaw +/-120 degrees. The directional sun gives the otherwise azimuthally symmetric procedural gradient a stable orientation marker; the product test requires the highlight in every view, requires the orbit to move it to the viewport edge within one raster pixel, and requires every rotated frame to differ from front. The 800x600 per-view PNGs and 4000x600 single-row contact sheet are stored only below `docs/tests/runtime/shader`.
- The 2026-07-14 DX12 closeout run measured the complete update at 4.363264 ms GPU, all sliced updates at 0.361920 ms average, and the heaviest/final SH9 slice at 1.819648 ms. The maximum slice is 41.7% of the full update and passes the product gate requiring less than 75%.
- The historical `ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9=1` capture targeted state 11. The current product contract targets the terminal 21st `ProjectDiffuseSh9` batch; a fresh RDC is required before making a current capture claim.
