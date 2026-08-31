# PFO-4d1e RDG Pass-Local Upload Transaction Plan

## Status

- Date: 2026-08-27
- Scope: Runtime90 PFO-4d1e, compiled render-graph dynamic buffer upload ownership
- Current status: `source_implemented_static_checks_passed_dynamic_validation_pending`
- Evidence boundary: counts below are current-worktree source observations. They are not GPU timing, power, RenderDoc, or product acceptance evidence.

## Problem

The frame-preparation path now merges scene constants, irradiance parameters, shadow-atlas state, reflection-probe parameters, and GPU Scene writes into one neutral `WgpuBufferUploadBatch`. The compiled graph still gives every pass executor a raw `wgpu::Queue`, so graph-time parameter updates can bypass the frame submission transaction.

The current scene-renderer inventory contains 35 production-candidate `queue.write_buffer` calls in 29 files. They mix four different lifetimes:

1. graph-pass frame parameters, such as the three light-grid buffers;
2. persistent or history initialization;
3. dynamic mesh/UI workspaces with their own capacity and retirement policy;
4. constructor/cold-path initialization.

Moving all four categories through one generic helper would hide their ownership differences. PFO-4d1e therefore implements only the graph-pass transport and migrates light-grid as the first measured consumer.

## Architecture Decision

Each `RenderPassGpuExecutionContext` owns a local `WgpuBufferUploadBatch`. Executors may append validated immutable uploads, but cannot submit, poll, or flush them. When an executor returns successfully, its `RecordedGraphPass` carries that batch to `RenderGraphStageExecution`.

The stage owner merges recorded batches in compiled topology order:

- serial recording appends immediately after each successful pass;
- parallel recording returns pass results with each ordered encoder bucket, then appends buckets and passes in the same order used for command-buffer submission;
- no shared `Mutex<WgpuBufferUploadBatch>` is introduced;
- merge complexity is `O(total uploads)` and moves `Vec` storage without cloning payloads or native handles.

The compiled-frame owner retains the existing preparation batch and prepared CPU shadow-state transactions until all graph stages succeed. It then appends the graph batch, performs exactly one `RenderBackend::enqueue_copy_buffer_upload_batch`, records one `FrameBufferUpload` producer ticket, and only then commits shadow-atlas and GPU Scene CPU state.

This ordering gives the following failure behavior:

- pass failure: the pass-local batch is dropped;
- later graph-stage failure: all previously collected graph uploads and frame-preparation uploads are dropped;
- backend admission failure: prepared CPU state remains dirty and is retried;
- ledger failure after backend acceptance remains a fatal frame-transaction error, matching the existing submission contract.

## First Consumer: Light Grid

Current source baseline:

- 3 direct `queue.write_buffer` calls per prepared light-grid pass;
- 3 independently borrowed CPU slices;
- no frame submission ticket or byte accounting at the pass boundary.

Target source contract:

- one exact-capacity immutable payload shared by 3 `WgpuBufferUpload` targets;
- zero direct light-grid `queue.write_buffer` calls;
- the uploads join the compiled frame's single `FrameBufferUpload` ticket;
- CPU packing and graph compilation remain unchanged in this step.

## Non-Goals

- No dynamic performance, power, screenshot, or RenderDoc acceptance claim before managed Windows validation runs.
- No texture-write migration in this step.
- No persistent/history/UI lifetime migration through the pass-local API.
- No removal of raw `queue` from the pass context until its remaining command-recording and texture consumers are classified and migrated.

## Validation Contract

Static checks must prove:

1. the GPU pass context owns and can drain a local batch;
2. `RecordedGraphPass` carries that batch and stage commit appends it;
3. parallel code contains no upload-batch mutex and preserves ordered bucket results;
4. compiled rendering enqueues the merged batch only after graph success and before scene submission;
5. shadow-atlas and GPU Scene prepared state commits only after backend admission and ledger retention;
6. light-grid production code contains zero direct buffer writes and at most three upload targets backed by one payload allocation.

Dynamic validation remains pending for managed Cargo checks/tests, WGPU product rendering, PNG evidence under `docs/tests/runtime/render`, RenderDoc capture, GPU/CPU timings, memory traffic, and power measurements.

## Current Source Result

- `RenderPassGpuExecutionContext` owns and drains one pass-local batch; `RecordedGraphPass` carries it and `RenderGraphStageExecution` merges it without a mutex.
- The compiled frame keeps preparation uploads retryable across graph failure, then merges graph uploads and performs one backend admission before CPU state commit and scene submission.
- Light-grid production direct buffer writes changed from 3 to 0; its three targets share one packed payload.
- The scoped scene-renderer production-candidate inventory changed from 35 writes in 29 files to 32 writes in 28 files.
- Eight exact Rust files pass `rustfmt --edition 2021 --config skip_children=true --check`; scoped whitespace and `git diff --check` checks pass, apart from existing LF/CRLF notices.
- Managed Cargo, real WGPU, screenshots, RenderDoc, timing, memory-traffic, and power acceptance remain pending and are not claimed by this record.

The PFO-4d1f follow-up applies the transport only to verified single-producer targets. TAA resolve,
camera velocity, and bloom are each called exclusively from one compiled RDG context and now return
one complete immutable parameter upload (or an empty batch on disabled/camera-cut paths). Their three
direct writes are removed, reducing the current inventory again to 29 writes in 25 files. Exposure is
intentionally excluded: histogram and resolve currently write the same shared parameter buffer, so its
correct optimization is one frame-owned preparation rather than two deferred pass-local writes.

PFO-4d1g resolves that exclusion at the frame boundary. Exposure parameters are constructed once by
the outer compiled-frame owner and shared by histogram/resolve; the old two producers and two direct
writes are gone. HZB stats reset is an ordered encoder clear rather than a CPU upload. Clustered
lighting, color-LUT bake, DOF prepare, and half-resolution composite return pass-local batches, with
disabled or invalid branches returning empty batches. The fixed exposure adaptation delta remains a
separate P0 correctness issue until authoritative frame timing reaches `ViewportRenderFrame`.

PFO-4d1h removes the remaining post-process buffer-write bypasses. Nine 432-byte full-screen/SSR
parameter buffers are now persistent per-pass slots instead of frame-time allocations; the coarse SSR
mip chain prepares one payload for all recorded mip passes. Reflection-probe and hybrid-GI scene data
share one exact-capacity payload across at most three non-empty ranges, and final pass params append to
the same pass batch. SSAO preparation stays at graph-resource binding but appends to the outer frame
batch, preserving rollback on later binding or graph failure. A fresh source scan reports zero direct
post-process `queue.write_buffer` calls and 13 remaining scene-renderer production candidates in 11
files after excluding test-only paths. Dynamic WGPU, PNG, RenderDoc, timing, memory-traffic, and power
acceptance remain pending.

PFO-4d1i adds delayed CPU-state publication for an RDG-owned persistent workspace. HZB occlusion
parameter preparation now returns a 32-byte upload and revision-qualified commit token only when the
committed args count differs. Tokens move beside pass-local uploads through recorded pass and ordered
stage aggregation; after the one merged upload ticket is admitted and retained, the frame owner commits
them back to the workspace. Graph or admission failure therefore retries the bytes, while stable entries
still skip uploads. The current scene-renderer inventory falls to 12 direct writes in 10 non-test files.

PFO-4d1j applies the same frame-transaction rule to mesh indirect args and compaction metadata,
which are prepared before graph recording but consumed by graph passes. Each of the nine fixed phases
keeps accepted and staged shadows plus buffer revisions; preparation emits shared-payload dirty ranges
and commit tokens without queue authority. The compiled owner appends them before graph execution and
commits only after the merged upload ticket is admitted and retained. A buffer created by a failed frame
therefore receives a full retry instead of inheriting an unaccepted CPU shadow. The exact dirty-range
algorithm remains `O(n)` with a worst-case `O(n)` range count until real profile data supports a bounded
coalescing threshold. The current inventory falls to 11 direct writes in 9 non-test files; dynamic WGPU,
RenderDoc, timing, memory traffic, and power evidence remain pending.

PFO-4d1k replaces the draw-local skinned-palette resource topology with a GPU Scene-owned two-slot
arena. Active joint matrices are packed contiguously, current/previous base and count values live in
the 192-byte instance row, and one arena payload joins `GpuScenePreparedUpload` before the existing
frame batch is admitted. Normal mesh commands now use the frame GPU Scene bind group; no draw retains
a palette buffer or palette-specific bind-group override. The staged slot and span map become previous
history only after scene success, so graph, admission, or scene failure retries the same staged side.
This removes two direct writes from the skinning production module and reduces palette GPU buffers from
`2 * live skinned instances` to two source-owned buffers. Current evidence is source/static only; WGPU,
RenderDoc, timing, allocation slack, memory traffic, and power evidence remain pending.
