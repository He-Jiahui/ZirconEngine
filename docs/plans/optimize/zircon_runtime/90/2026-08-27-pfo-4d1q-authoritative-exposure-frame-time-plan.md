# PFO-4d1q Authoritative Exposure Frame Time

Status: `source_implemented_static_checks_passed_dynamic_exposure_validation_pending`

Date: 2026-08-27

## Problem

Exposure parameter preparation currently uploads a fixed `1 / 60` delta every frame. The runtime
already advances one `FrameTimeSnapshot`, but its outer-frame index and raw real delta stop at scene
update and never enter `RenderFrameExtract` / `ViewportRenderFrame`. Exposure therefore adapts at a
different rate when frame cadence changes and continues to invent time for synthetic frames.

## Reference Alignment

Unreal's `FSceneViewFamily` carries a shared `FGameTime` for every view in one engine frame, and
`PostProcessEyeAdaptation.cpp` assigns `Parameters.DeltaWorldTime` from
`View.Family->Time.GetDeltaWorldTimeSeconds()`. The relevant architectural rule is one frame-family
time receipt shared by all views, rather than a post-process pass reading wall time or assuming a
refresh rate. Zircon's existing outer `FrameTimeSnapshot` is the corresponding authority.

## Design

1. add a lightweight `RenderFrameTiming` DTO to `RenderFrameExtract`, containing outer frame index
   and raw real delta seconds;
2. default synthetic/snapshot extracts to frame 0 / delta 0, so absence of authoritative time is
   explicit and does not imply 60 FPS;
3. the dynamic session stores the latest accepted outer-frame timing after `tick_time`; cached scene
   extracts remain keyed only by scene/view content, and timing is overwritten after cache lookup;
4. every camera frame shares the same `Arc<RenderFrameExtract>` and therefore the same timing receipt;
5. exposure preparation accepts the extract delta and removes its fixed-rate constant;
6. no pass reads `Instant`, no second clock is created, and no time field is added to cache identity.

## Acceptance Boundary

Source tests must cover timing sanitization/defaults, cache-overlay wiring, the one exposure parameter
producer, and absence of the fixed adaptation constant. Focused format and diff checks belong to this
slice. Cargo, WGPU, exposure image sequences, RenderDoc, profile, and power remain deferred.

## Completed Source Work

1. `RenderFrameTiming` now carries the outer frame index and sanitized raw-real delta through the
   neutral `RenderFrameExtract` contract. Scene and snapshot producers explicitly default to frame
   zero/delta zero when no runtime clock receipt exists.
2. `RuntimeDynamicSession::tick_frame` stores timing immediately after the sole `tick_time` call.
   `current_extract` applies the editor camera and then overlays this timing on the cached extract;
   neither timing type nor field enters `RuntimeFrameExtractCacheKey`.
3. Compiled scene rendering passes the extract delta into the sole exposure-parameter preparation
   owner. The fixed `EXPOSURE_ADAPTATION_DELTA_SECONDS` constant and pass-local queue writes are absent.
4. Focused `rustfmt --edition 2021 --config skip_children=true --check` and scoped
   `git diff --check` passed. Fresh source counts are: fixed exposure delta constants `0`, session
   timing overlays `1`, outer tick captures `1`, exposure extract-delta consumers `1`, timing mentions
   in the scene cache `0`, and pass-local `Instant` uses `0`. Eight explicit frame-extract
   initializers were audited and none omit timing or a struct-update source.
5. No Cargo/WGPU/product run was performed in this implementation slice. Exposure cadence PNG
   sequences, RenderDoc capture, CPU/GPU profile, memory, and power evidence remain pending and are
   not inferred from these source counts.
