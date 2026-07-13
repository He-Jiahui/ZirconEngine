# Realtime IBL time slicing

## Owner and scope

`realtime_ibl_time_slice.rs` owns the frame-to-frame scheduling contract for procedural-sky IBL rebakes. It does not own sky rendering, render-graph resource allocation, GPU submission, artifact readback, or cache persistence. Those remain with the existing environment capture, IBL compute, and runtime writeback owners.

The scheduler follows the real-time sky capture sequence in Unreal's `ReflectionEnvironmentRealTimeCapture.cpp` while reusing Zircon's GGX FIS PMREM and SH9 kernels. It never introduces a second filtering algorithm.

## Buffer and publication contract

- Slot A/B form a ready/work pair.
- The renderer samples only the ready slot while all slices write the work slot.
- A completed 12-state cycle atomically publishes the work slot and swaps A/B.
- A parameter change increments the generation, discards unfinished work, and preserves the published environment.
- A late GPU completion carrying an older generation token is `Stale` and cannot publish.
- A failed GPU slice is `Retry`; state and published resources remain unchanged.
- Re-requesting the currently published key cancels obsolete pending work without rebuilding it.

The first environment has no valid ready slot, so it is emitted as one complete batch. Later rebakes use time slicing.

## Twelve logical states

With the default two capture faces per frame, one cycle occupies 16 physical frames:

| State | Work | Physical frames |
|---|---|---:|
| 0 | Capture sky faces 0-1, 2-3, 4-5 | 3 |
| 1 | Capture cloud faces 0-1, 2-3, 4-5 | 3 |
| 2 | Generate source cubemap mips | 1 |
| 3-5 | PMREM mip 0, faces 0-1, 2-3, 4-5 | 3 |
| 6 | PMREM mip 1, all faces | 1 |
| 7 | PMREM mip 2, all faces | 1 |
| 8 | PMREM mip 3, all faces | 1 |
| 9 | PMREM mips 4-5, all faces | 1 |
| 10 | PMREM mips 6-last, all faces | 1 |
| 11 | Project diffuse SH9 and publish | 1 |

Unavailable high-mip states emit no zero-length GPU dispatch, but still advance the logical schedule.

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

- `CaptureSky` and `CaptureCloud` write the requested face range of the work source cubemap.
- `GenerateSourceMips` expands into one pass per destination mip. Each pass reads only the previous mip through a single-mip `Cube` view and writes only the destination through a single-mip `D2Array` storage view. This keeps sampled and writable subresources non-overlapping under WGPU validation.
- Each PMREM mip slice becomes a separate pass. It reads the work source cubemap, writes the work PMREM cubemap, and preserves `first_face` plus `face_count` in its workload.
- `ProjectDiffuseSh9` reads the work source cubemap, writes the work SH9 buffer, and depends on the last generated operation.

All passes use the async-compute queue contract and explicit dependencies. `RealtimeIblGpuResources` allocates exactly two source cubes, two PMREM cubes, and two SH9 buffers, then binds graph aliases to views of those allocations. The allocations and every A/B view remain resident, but graph materialization binds only names present in the current `CompiledRenderGraph::resource_lifetimes()`. A time-sliced graph intentionally omits unused face/mip aliases; binding all resident aliases would create stale external-resource bindings outside the compiled lifetime set. `RealtimeIblWgpuRecorder` records the graph pass order: dedicated procedural capture and source-mip kernels feed the existing GGX PMREM and SH9 pipeline cache. The V1 cloud pass repeats the analytical sky capture because no separate cloud model is present; it does not introduce a different cubemap orientation or filtering path. SH9 reuse clears the work buffer before projection.

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

## Validation

- Scheduler tests cover initial publication, the exact 16-frame/12-state sequence, stale generations, retry behavior, short mip chains, cancellation, and dispatch expansion.
- PMREM command tests cover face offset serialization, face-count dispatch reduction, invalid ranges, and WGSL ABI text.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min` passes with the Windows target under `E:\cargo-targets`.
- A standalone executable harness compiled the production scheduler source and completed the initial publish, exact 16-frame state sequence, generation invalidation, and cancellation assertions with exit code 0.
- At the scheduler-only checkpoint, the normal focused lib-test command was blocked by unrelated missing diagnostics symbols in `zircon_runtime/src/tests/prelude.rs`; a temporary cfg experiment only established that the crate test target compiled after excluding that owner and was reverted because it selected zero scheduler tests. No EC-M4 visual pass was claimed from that scheduler-only slice.
- The SceneRendererCore integration check passed on Windows: `cargo check -p zircon_runtime --lib --no-default-features --features core-min` finished successfully with the target under `E:\cargo-targets\zircon-shader06-ecm4-runtime-20260712`.
- The final-source graphics integration export compiled on Windows. The exact contract test passed 1/1, the direct-SH9 ignored 8x8 product export passed 1/1 in 50.10 seconds, and the ignored five-view regression product export passed 1/1 in 94.07 seconds.
- The 1600x1200 direct-SH9 PBR result, CPU wall times, and 17 nonzero WGPU timestamp-query samples are recorded below `docs/tests/runtime/shader`. Initial full publication measured 4.981760 ms GPU; the sixteen sliced batches averaged 0.321728 ms and peaked at 4.472832 ms in the SH9 projection slice.
- The export renders a presentation-only frame after state 11 publishes the work slot. This prevents the accepted screenshot from accidentally sampling the previous ready slot; the final image differs from the retained pre-SH9 image by RGB channel MAE 27.999296.
- A five-view perspective product export covers front, pitch +/-120 degrees, and yaw +/-120 degrees. The directional sun gives the otherwise azimuthally symmetric procedural gradient a stable orientation marker; the product test requires the highlight in every view, requires the orbit to move it to the viewport edge within one raster pixel, and requires every rotated frame to differ from front. The 800x600 per-view PNGs and 4000x600 single-row contact sheet are stored only below `docs/tests/runtime/shader`.
