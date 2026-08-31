# PFO-4d1s GPU Scene Sideband Upload Transaction

Status: `source_implemented_static_checks_passed_dynamic_wgpu_validation_pending`

Date: 2026-08-27

## Structural Review

GPU Scene primitive, instance, light, scene-count, and skinned-palette data already follow one
`prepare -> append -> backend accept -> commit` transaction. Morph payloads and virtual-geometry
resident rows do not: mesh build gives them raw `wgpu::Queue`, performs driver-visible writes before
the rest of the scene has been prepared, and immediately replaces their committed CPU shadows.

That split is a correctness and ownership defect. If later mesh synchronization, render-graph
construction, upload admission, or scene submission fails, the buffers may not belong to an
accepted frame while the CPU shadows claim that the data was published. A retry can consequently
produce no upload. Virtual-geometry scene-count parameters also read the old committed shadow, so
deferring the resident-row commit without passing this frame's staged counts would publish an
internally inconsistent frame.

The existing changed-row scan is not the current algorithmic bottleneck: it is a single forward
scan and emits maximal contiguous changed runs in `O(n)` time. No gap threshold, hash table, sort,
or per-row allocation is justified without profile evidence. The structural fix must retain that
bound and move only immutable payload/range ownership.

## Reference Alignment

Unreal's `GPUScene.cpp` allocates `FRDGScatterUploader` instances from `FRDGBuilder`, fills them in
setup work, and closes them through `PrimitiveUploadBuffer.End(GraphBuilder, ...)` and matching
instance/lightmap calls. Light data likewise uses a graph-allocated async scatter uploader and
`ResizeAndUploadTo*`. GPU Scene upload work therefore stays registered with the render graph rather
than issuing an unrelated queue mutation and publishing CPU state before the graph owner accepts
the work.

`dev/LumenInUE5.5.4WithComputeShader` does not own this GPU Scene data-plane boundary. Its explicit
D3D12 upload and command-list code is not an interface template for bypassing Zircon's neutral
frame upload transaction.

## Design

1. Move the existing maximal changed-run scan into `GpuSceneBufferUploadBatchBuilder`; remove the
   raw queue write helpers.
2. Replace the immediate morph and virtual-geometry upload methods with preparation objects that
   own an immutable `WgpuBufferUploadBatch`, the report, and move-only next-shadow commit data.
3. Buffer growth and scene bind-group rebuild remain preparation-time resource work. Committed CPU
   shadows change only after the prepared scene upload has left local ownership and the frame owner
   has accepted the combined batch.
4. Extend `GpuScenePreparedUpload` with typed morph/VG attachments. Appending transfers batch
   ownership and accounts bytes once; commit applies all core and sideband shadows together.
5. Pass the staged virtual-geometry page and cluster counts into core GPU Scene preparation so
   remap parameters describe the same prospective frame rather than the previous committed frame.
6. Mesh build prepares VG first, assigns morph slots while preparing morph data, synchronizes the
   core scene using the staged counts, and then attaches both preparations. It no longer receives a
   raw queue or adds sideband byte totals out of band.
7. Tests must cover successful commit, stable no-upload frames, changed-row byte counts, shrink
   reuse, and abort/retry behavior where dropping preparation leaves committed shadows unchanged.
8. Each sideband owner permits exactly one outstanding preparation. A move-only reservation stays
   alive after attachment and is released only by successful commit or drop. The core preparation
   and both sidebands retain the same unforgeable `GpuScene` owner identity; attachment has no
   caller-supplied scene parameter, and `append_to` validates the target before the batch leaves
   local ownership. Commit repeats the target validation. This prevents both an older frame from
   publishing after physical-buffer replacement and an A-sideband/B-core cross-scene splice.

## Performance Measurement Plan

Dynamic validation remains a later Windows WGPU milestone. When available, capture at least stable,
single-row-change, and full-change scenes at 1K and 10K morph/VG rows. Record preparation CPU
p50/p95/p99, emitted range count and bytes, frame upload packet count, native buffer-write/copy
count in RenderDoc, GPU frame time, VRAM peak, and process/GPU power where tooling exposes it.

Acceptance requires linear preparation growth, zero sideband-specific submission ticket, zero
stable-frame sideband upload, exact one-row byte counts, and no material regression against the
pre-change frame-time distribution. No performance or power claim may be made from source counts.

## Source Acceptance Boundary

Focused source checks require zero production `queue.write_buffer` in GPU Scene morph/VG, zero raw
queue parameter in their mesh-build wrappers, one shared prepared GPU Scene upload owner, explicit
prospective VG counts, and abort/retry tests. Scoped rustfmt and diff checks are required. Cargo,
real WGPU execution, product PNG, RenderDoc, profile, VRAM, and power remain pending until the
managed Windows validation lane is available.

## Completed Source Work

1. Morph and virtual-geometry resident uploads now return typed preparation objects containing an
   immutable batch, report, and move-only shadow commit token. Their mesh-build wrappers no longer
   accept `wgpu::Queue`; both preparations attach to the existing `GpuScenePreparedUpload`.
2. The maximal contiguous changed-run scan moved into `GpuSceneBufferUploadBatchBuilder` and keeps
   its single forward `O(n)` traversal. The three raw queue helper functions and both immediate
   sideband upload APIs were removed without adding thresholds, hashes, sorting, or per-row maps.
3. Virtual-geometry page/cluster counts are carried as prospective frame state into direct or
   staging core preparation. Remap parameters therefore describe the same prepared resident set,
   while committed shadows and counts change together only after scene success.
4. Every growable sideband buffer retains a `require_full_upload` intent after physical replacement.
   A dropped frame cannot make a new empty buffer look synchronized merely because the next frame
   returns to the old committed shadow; successful commit alone clears the intent.
5. Code review identified two ownership holes: independently prepared frames could outlive a later
   physical-buffer replacement, and a caller-supplied scene argument could disguise an A-sideband
   as belonging to a B-core frame. Morph and VG now each hold a single-outstanding reservation
   through the combined frame, while core and sideband preparations retain one unforgeable scene
   identity. Foreign attachment, pre-backend ownership transfer, and commit targets are validated;
   drop releases the reservation for retry.
6. Focused source counts passed after review repair: production raw GPU Scene/build
   `queue.write_buffer` `0`; sideband raw queue parameters `0`; old immediate APIs/helpers `0`;
   typed product attachments `2`; prospective-count markers `4`; combined-frame abort/retry tests
   `2`; overlapping-preparation rejection tests `2`; foreign-scene attachment rejection tests `2`.
   Scoped rustfmt check and diff check passed.
7. The master PFO plan and Render03 contract were updated. The generated GPU Scene module document
   remains pending because an external process keeps
   `docs/zircon_runtime/graphics/scene/gpu_scene/mod.md` write-locked; this does not block the next
   source slice and must be retried before milestone acceptance.
8. Cargo, real WGPU execution, product PNG, RenderDoc, 1K/10K profile, VRAM, and power were not run,
   so this source slice makes no runtime performance or power claim.
