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

This slice establishes and compiles the scheduler and PMREM command ABI. Production render-graph ownership still must bind the A/B resources and submit each returned operation before EC-M4 can be marked fully complete or used as visual evidence.

## Validation

- Scheduler tests cover initial publication, the exact 16-frame/12-state sequence, stale generations, retry behavior, short mip chains, cancellation, and dispatch expansion.
- PMREM command tests cover face offset serialization, face-count dispatch reduction, invalid ranges, and WGSL ABI text.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min` passes with the Windows target under `E:\cargo-targets`.
- A standalone executable harness compiled the production scheduler source and completed the initial publish, exact 16-frame state sequence, generation invalidation, and cancellation assertions with exit code 0.
- The normal focused lib-test command is currently blocked by unrelated missing diagnostics symbols in `zircon_runtime/src/tests/prelude.rs`; a temporary cfg experiment only established that the crate test target compiles after excluding that owner and was reverted because it selected zero scheduler tests. No EC-M4 visual pass is claimed from this scheduler-only slice.
