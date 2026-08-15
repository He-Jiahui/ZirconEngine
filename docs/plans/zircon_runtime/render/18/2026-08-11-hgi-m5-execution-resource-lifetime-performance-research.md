---
date: 2026-08-11
related_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
related_m5_design: docs/plans/zircon_runtime/render/18/2026-08-10-hgi-m5-sdf-architecture-and-performance-plan.md
doc_type: structural-performance-research
status: structural_slices_source_complete_validation_pending
references:
  - dev/LumenInUE5.5.4WithComputeShader/GenerateProbeTraceTiles.cpp
  - dev/LumenInUE5.5.4WithComputeShader/Res/Shader/UpdateRadianceCache/GenerateProbeTraceTiles.hlsl
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCache.cpp
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/global_sdf/{dispatch,packing,state,trace_bindings}.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/root_output_sources/runtime_prepare_collector.rs
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/performance/01/2026-07-17-renderdoc-toolchain-probe.md
---

# HGI M5 Execution Resource Lifetime Structural Performance Research

## Scope And Evidence Boundary

This is a source-backed design and measurement plan, not a performance result. No current-source
WGPU run, timestamp query, RenderDoc capture, PNG, CPU percentile, GPU duration, power sample, or
percentage improvement exists for this worktree. Dynamic evidence remains coordinator-owned and
must be written under `docs/tests/runtime/render` only after it is produced.

## Implementation Checkpoint

The selected first slice is source-complete. A dedicated
`gpu_resources/probe_trace_tile_generation_pipeline.rs` now owns the trace-tile-generation
bind-group layout, shader module, pipeline layout, and compute pipeline. `HybridGiGpuResources::new`
creates those device-lifetime objects once; `scene_prepare_trace_tiles` now creates only its
frame-dependent bind group and encodes with the retained pipeline. Frame buffers, scene textures,
and `HybridGiGpuPendingReadback` ownership are unchanged.

The test-first source contract was added before the move and then checked after it: it requires the
constructor to create both device objects, requires the frame encoder to bind the retained pipeline,
and rejects either factory definition in the frame owner. A targeted `rustfmt --check` and static
ownership/path check pass. Cargo, WGPU, RenderDoc, readback, PNG, timestamp, and power commands
were deliberately not run in this session; the acceptance protocol below remains pending.

The audit covers the HGI `execute_prepare` path after the completed M5 Global SDF page-table and
influence-index work. It does not reopen the already bounded Global SDF candidate or trace lookup
algorithms. Its goal is to correct resource lifetime ownership before any allocation micro-tuning.

## Current Execution Graph

```text
runtime_prepare_collector
  -> HybridGiGpuResources::execute_prepare
    -> collect_inputs
    -> create_buffers
      -> scene_prepare_resources
        -> scene atlas/depth/trace-tile textures and buffers
        -> encode_probe_trace_tile_generation
    -> queue_params
    -> per-frame completion and radiance-cache bind groups
    -> radiance-cache, completion, and probe-trace dispatches
    -> HybridGiGpuPendingReadback owns all frame resources until completion
```

The collector already prevents unbounded submissions through the shared readback admission limit.
That limit is a correctness boundary: a resource is not reusable merely because the next CPU frame
has started. `HybridGiGpuPendingReadback` deliberately owns the output buffers, scene textures,
upload buffers, sample readback buffers, and trace-tile buffers until the matching GPU observation
is consumed.

## Source Findings

| Finding | Current source evidence | Consequence | Decision |
|---|---|---|---|
| Trace-tile generation pipeline lifetime is wrong | `create_buffers/scene_prepare_trace_tiles.rs` creates a bind-group layout, shader module, pipeline layout, and compute pipeline inside `encode_probe_trace_tile_generation` for every nonempty tile batch. | A stable HGI frame recompiles a device-lifetime pipeline even though shader and layout are invariant. | Fix now as the isolated device-lifetime slice. |
| Frame resource lifetime is intentionally in-flight | `execute.rs` moves output buffers and all scene-prepare resources into `HybridGiGpuPendingReadback`; the collector keeps multiple pending futures. | A naive shared buffer/texture cache could overwrite data before readback, corrupting completion or diagnostics. | Do not cache frame outputs in this slice. |
| Full prepare construction is allocation-heavy | `create_buffers/mod.rs` creates twelve named storage buffers for each prepare call; `scene_prepare_resources.rs` may also create atlas, capture, depth, upload, sample-readback, and trace-tile resources. | This is a likely stable-frame CPU/device churn source, but its safe repair requires a bounded readback-slot ring and a precise scene-generation key. | Measure first, then design a separate ring/cache slice. |
| Fallback texture creation is frame-local | `dispatch_probe_trace_tiles.rs` creates two 1x1 fallback textures and views when a Surface Cache view is missing. | Empty or not-yet-captured scenes can pay device object creation on every probe-trace dispatch. | Include in the later device fallback-resource owner after measurement; do not mix it into the pipeline move. |
| Bind groups are frame-resource dependent | Completion and radiance-cache bind groups bind buffers created by this invocation. | Caching a bind group without a slot/key protocol would bind stale or in-flight resources. | Keep frame-local until a slot-ring owns the buffers. |

## Runtime-Prepare Profiling And Admission Gap

The first source audit found that runtime-prepare compute was outside the existing Plan17
measurement and readback-admission lifetime. At that point,
`scene_renderer_core_render_compiled_scene/render/render.rs` encoded collectors before
`GpuReadbackQueue::prepare_frame` and `GpuPassTimer::begin_frame`. The HGI Global SDF and prepare
dispatches therefore had no timestamp scopes, and a later unavailable readback slot could leave an
encoded update without a completion observation. The existing HGI per-instance backlog prevents
unbounded futures, but it cannot make already encoded work observable after the shared queue has
declined the frame.

The existing `GpuPassTimer` is the correct shared owner. It already reserves timestamps, resolves
through the same `GpuReadbackQueue`, and `FrameProfiler` matches completed timings to
`RenderGraphExecutionRecord` pass-profile names. A second HGI timer or readback ring would make
the in-flight capacity accounting less reliable and would produce incompatible diagnostics.

The next infrastructure slice is therefore deliberately narrow:

1. reserve the shared readback frame before runtime prepare, then begin the existing pass timer
   only when that reservation succeeds;
2. pass the admission result through `RuntimePrepareCollectorContext`; a collector may still
   consume completed CPU observations, but must not encode new work that requires a GPU
   completion when the frame was not admitted;
3. provide an opaque context-owned timestamp scope so collectors cannot create a parallel timer;
   record only logical, dispatched groups such as
   `runtime_prepare.hybrid_gi.global_sdf_build` and
   `runtime_prepare.hybrid_gi.prepare`;
4. retain direct-prepare CPU profile records with the plugin readback packet, then append them to
   `RenderGraphExecutionRecord` before `FrameProfiler` snapshots the frame. This keeps the
   existing name-based GPU merge and makes unsupported/capacity-exhausted timestamp states
   explicit rather than silently dropping the pass; and
5. preserve the current single encoder and resolve path. Error paths must abort a prepared
   readback frame exactly once and defer the active timer rather than resolve a partial frame.

This is observability and correctness infrastructure, not a claim of an optimization. It will be
accepted only when source tests prove the pre-admission ordering, no-admission dispatch gate,
scope/profile-name identity, and frame-profile merge. Managed WGPU validation must then show the
two HGI rows in the same frame profile as graph passes, with a normal PNG/readback and no rejected
readback work being encoded.

## Implemented Source Contract

The implementation now reserves the shared readback frame before runtime prepare and starts the
existing `GpuPassTimer` only for an admitted frame. The admission is passed through
`RuntimePrepareCollectorContext`; HGI and particle collectors may consume ready observations while
rejecting all new GPU work and readback requests when no shared completion slot is available.

HGI records the Global SDF build and radiance-cache prepare groups through context-owned scopes
named `runtime_prepare.hybrid_gi.global_sdf_build` and
`runtime_prepare.hybrid_gi.prepare`. Their CPU records are attached to the advanced-plugin packet
and moved into `RenderGraphExecutionRecord` before graph execution, so the existing frame profiler
can merge them with the matching shared-timer results. A runtime-prepare, resource-binding, graph
materialization, or graph-execution failure aborts the prepared readback frame and defers the active
timer frame before returning the error.

Focused static contracts lock the ordering, admission gate, scope pairing, profile handoff, and
graph-failure timer cleanup. They do not prove WGPU execution or performance. Coordinator-managed
Windows WGPU readback, product PNG, RenderDoc capture, and current-source measurements remain
required for acceptance.

The static count is intentionally limited to what source proves: at least twelve named buffer
allocations per `create_buffers` call before optional scene resources. It is not a measured
allocation count or timing result.

## Reference Comparison

`dev/LumenInUE5.5.4WithComputeShader/GenerateProbeTraceTiles.cpp` constructs the compute
`RenderPass`, bindings, and pipeline in `InitGenerateProbeTraceTilesPass`; its execute function
only supplies indirect dispatch arguments and runs the existing pass. The paired HLSL generates a
bounded tile list per probe, rather than creating shader state at execution time.

Unreal's `LumenRadianceCache.cpp` keeps Radiance Cache state as external graph resources and
registers them for pass use. RDG may allocate transient work resources, but it does not make
pipeline compilation a per-dispatch operation. Zircon deliberately differs by retaining a small
MVP WGPU owner instead of copying RDG; it must still preserve the same lifetime separation:
device shader state, generation-stable resources, frame-slot resources, and readback ownership are
four different domains.

## Chosen First Slice

Move only trace-tile generation shader state to `HybridGiGpuResources`, the existing per-device
HGI resource owner:

```text
HybridGiGpuResources (device lifetime)
  -> trace-tile generation bind-group layout
  -> trace-tile generation compute pipeline

scene_prepare_trace_tiles (frame lifetime)
  -> seed / params / tile / indirect-args buffers
  -> one bind group referencing those frame buffers
  -> encode using the device-owned pipeline

HybridGiGpuPendingReadback (in-flight lifetime)
  -> retains only frame resources required by the submitted observation
```

The new device factory belongs in a dedicated `gpu_resources` child owner, not inside
`create_buffers/scene_prepare_trace_tiles.rs`, because that file owns CPU tile planning and
frame-resource encoding. `HybridGiGpuResources::new` wires the factory next to the existing
completion, probe-trace, and Radiance Cache pipeline owners. No shared foundation, public runtime
DTO, Shader ABI, readback protocol, dispatch count, or fallback route changes in this slice.

## Explicitly Deferred Structural Work

The next candidate is a bounded `HybridGiPrepareFrameResourceRing` per radiance-cache instance:
each slot is reusable only after its matching pending readback completes, grows only for an
observed capacity demand, and owns frame-dependent buffers/bind groups. A separate immutable
scene-prepare artifact may be shared across slots only when an exact scene-generation key proves
that atlas, capture, depth, voxel, and trace-tile inputs are unchanged. Dynamic input, changed
capture data, changed bounds, changed quality, or any pending slot must bypass reuse.

This is deliberately not implemented here. The current framework lacks an authoritative
scene-prepare generation and the dynamic evidence has not yet separated CPU allocation cost from
GPU work. A local HGI cache keyed by mesh count or resource identity would repeat the invalidation
mistake already rejected by the M5 Global SDF design.

## Second Structural Audit: Voxel Fallback Sampling

This audit covers the currently separate voxel fallback, not the Mesh SDF or Global SDF path.
The relevant owners are `scene_prepare_voxel_samples.rs`, `voxel_clipmap_debug.rs`, and
`trace_probe_tiles_aggregate.wgsl`. It was completed before choosing a CPU optimization.

### Source-backed limits and route

- A voxel clipmap has a fixed `4 x 4 x 4` cell layout, or 64 cells. The scene owner admits at
  most eight levels, so the current configuration upper bound is 512 cells before empty cells
  are removed from the GPU descriptor range.
- Each probe trace entry samples at most 16 trace tiles. A tile attempts Surface Cache first,
  then Global SDF, and invokes the voxel path only after both are unavailable or miss.
- When that fallback is selected, `voxel_fallback_tile_sample` scans its complete packed
  voxel-cell descriptor range. Its source-level worst case is therefore
  `O(trace_entries * min(tile_count, 16) * packed_voxel_cells)`. This is bounded in the current
  MVP configuration, but it is not a spatially indexed lookup and must not silently become the
  world-trace primary path.
- The probe diagnostics already record the actual `voxel_candidates` scanned per trace entry.
  They are the dynamic evidence source for deciding whether a GPU voxel-cell index is justified;
  no current run has established that it is a measured bottleneck.

### Confirmed CPU duplication

The scene-prepare resource builder needs three output products per clipmap cell: summed radiance,
dominant-card identity, and dominant-card radiance. The existing production route computes summed
radiance in one mesh-to-cell traversal, then independently computes the dominant entries once for
the node-id product and once again for the RGBA product. All three traversals call the same
material capture and use the same exact AABB cell range. The latter two are not distinct
algorithms; they are two projections of the same dominant-entry calculation.

This is a structural CPU duplication independent of adapter timing. It is safe to remove without
changing quality, radiance-cache revisions, Global SDF page eligibility, or the GPU trace ABI.
The separate clipmap aggregate sample remains separate because it samples each mesh at its
translation while cell products sample at cell centers; deriving one from the other would change
the capture contract.

### Chosen forward design

Introduce a private `SceneVoxelClipmapCellSamples` aggregate in the existing voxel sampling owner.
For one clipmap it makes one mesh/AABB-cell traversal and produces the three already-public
snapshot projections:

```text
one mesh/cell traversal
  -> cell summed RGBA samples
  -> dominant node-id samples
  -> dominant RGBA samples
  -> existing HybridGiScenePrepareResourcesSnapshot fields
```

The aggregate must preserve these invariants:

1. Cell iteration and tie-breaking remain deterministic: higher capture strength wins, then the
   larger node id breaks ties, exactly as before.
2. A mesh missing an authoritative prepared bound remains excluded by the neutral-sideband
   projection; this optimization may not create the old transform-scale fallback.
3. Empty cells, alpha values, persisted radiance overrides, occupancy masks, and the descriptor
   packing order remain byte-for-byte compatible with the existing snapshot contract.
4. The aggregate is frame-local and moves into the existing pending readback owner. It is not a
   cross-frame cache and needs no generation key.

This reduces the repeated mesh/cell traversal and `mesh_capture_radiance` evaluations for those
three cell products from three to one. That is an exact operation-count reduction for the shared
loop, not a claimed CPU-time reduction.

### Source implementation checkpoint

The aggregate is now the only production path in
`execute_prepare/execute/create_buffers/scene_prepare_voxel_samples.rs`. It preallocates each
final projection to the bounded clipmap-cell count and extends it in clipmap order, rather than
retaining a full-frame aggregate collection and projecting it afterward. The three legacy helpers
remain test-only reference implementations. Focused source regressions prove that the aggregate
matches every legacy projection, retains the higher node-id tie-break, and omits a mesh with no
authoritative prepared world bound. Formatting and patch-whitespace checks pass; this remains
source evidence only and does not replace the coordinator-owned dynamic protocol below.

The shared runtime-prepare collector test owner is now folder-backed at
`graphics/runtime_prepare_collector/tests.rs`; the production collector root is 503 lines and
the focused test owner is 346 lines. This keeps WGPU fixture setup separate from admission,
readback, and timestamp-scope behavior. A second current-source audit also reconfirmed the
fail-closed deformation path: active morph targets use conservative prepared morph bounds but
select typed voxel fallback, while skinned objects use the configured Global-SDF clipmap extent
for conservative influence rather than claiming their static base bounds are current pose bounds.

### Measurement and acceptance protocol

The coordinator must profile the same current-source scene before and after this slice with the
existing runtime-prepare timestamp/readback admission path. Record:

1. CPU time for the named HGI prepare group and the scene-prepare voxel sampling subphase once a
   subphase counter is present;
2. trace diagnostics: entry count, `voxel_candidates`, source/fallback distribution, and all
   Global SDF page counters;
3. device creation and upload counters, because this CPU change must not create a second resource
   ring or increase upload bytes;
4. 31 warm samples after a 300-frame settling period, plus a cold frame, on one adapter and scene;
   and
5. a same-revision WGPU readback, inspected PNG beneath `docs/tests/runtime/render`, and a
   RenderDoc capture through `D:\\Tools\\renderdoc`.

Acceptance requires identical scene-prepare readback projections and no pixel/fallback regression.
CPU duration, GPU duration, energy proxy, and platform power may be reported only from those
measurements.

### 2026-08-14 Corrective Decision: Fixed Voxel Cell Lookup

The earlier decision to defer a GPU voxel spatial index no longer holds after the current source
audit. `voxel_fallback_tile_sample` linearly scans every packed voxel-cell descriptor and uses
`abs_diff_u32(descriptor.secondary_id, tile_sample_id)` as its cone relation. A voxel cell index
is `x + 4*y + 16*z`; it is not a spatial distance. Consequently adjacent cells across a z-plane
boundary can be rejected while cells that only happen to be adjacent in the flattened array can be
included. The scan is also proportional to every occupied cell in every active clipmap, rather
than the trace tile's clipmap-local neighborhood.

This is not a micro-optimization decision. The producer has an explicit `4 x 4 x 4` cell grid and
a maximum of eight voxel clipmap levels. The Lumen reference's `TraceVoxels.hlsl` resolves a
three-dimensional cell within the selected Global SDF page, then reads its fixed four-object cell
list from `GlobalDistanceFieldPageObjectGridBuffer`; it does not scan world descriptors for one
cone sample. Zircon must retain its smaller fallback representation but adopt the same
spatial-locality boundary.

The selected repair is a frame-owned fixed lookup buffer built alongside the existing scene
prepare descriptor buffer. It contains at most eight entries, each with a clipmap id followed by
64 descriptor indices in canonical cell-index order; an invalid sentinel represents an empty
cell. The shader resolves the clipmap through at most eight ids and examines the fixed 64 slots of
that clipmap only. It reconstructs `x`, `y`, and `z` before applying the cone relation, so it
preserves multi-cell radiance aggregation without treating the flattened cell id as a metric. The
diagnostic `voxel_candidates` counter becomes the number of valid local descriptors actually
loaded, not the global packed descriptor count.

The buffer is deliberately frame/in-flight owned with the descriptor buffer. It has no cross-frame
cache, generation key, duplicate scene representation, or new Global SDF ownership. Malformed cell
ids, duplicate `(clipmap_id, cell_index)` rows, or more than eight active clipmaps make the lookup
incomplete; dispatch then disables the voxel backend for that frame and retains the existing typed
fallback route rather than sampling a partial index.

The corrected source bound is `O(trace_entries * min(tile_count, 16) * (L + 64))`, where `L <= 8`
is the clipmap-id lookup. The previous bound was
`O(trace_entries * min(tile_count, 16) * packed_voxel_cells)`, with up to 512 cells in the MVP
configuration and incorrect flattened-cell filtering. This is an operation-count and correctness
statement, not a timing, power, or percentage claim.

Focused source tests must prove fixed entry capacity, sentinel handling, duplicate/overflow
fail-closed behavior, ABI/bind-group agreement, absence of the global descriptor loop, and
three-dimensional neighbor selection. The existing coordinator-managed WGPU voxel-cone test,
product PNG, RenderDoc capture, and timestamp sample set remain the required dynamic acceptance
evidence for this changed shader ABI.

## Third Structural Audit: Global SDF Transient Input Accounting

The Global SDF build path already has the correct first-order lifetime split. Per-instance
`GlobalSdfGpuState` owns the device-lifetime page atlas and trace page table; device-lifetime
`GlobalSdfGpuResources` owns the immutable build pipeline. A selected dirty-page batch must retain
its page parameters, object transforms, Mesh SDF payload metadata, Mesh SDF voxel words, candidate
list, completion words, and bind group until the matching readback observation completes. Caching
those frame inputs without an in-flight ownership protocol would recreate the aliasing risk that
the shared readback admission limit prevents.

The source audit nevertheless identified a measurable candidate for a later, separate cache
design. `dispatch_pages` creates seven transient buffers and one bind group for every encoded
page-build batch. `pack_global_sdf_build_inputs` deduplicates objects within the batch, but still
serializes each selected ready Mesh SDF payload and voxel word into that batch's transient storage.
The upload admission cap is `4 * 1024 * 1024` voxel words, or 16 MiB of voxel data before
payload/page metadata. This is an upper bound, not a measured cost and not evidence that a cache
is beneficial in a real scene.

The current-source statistics now expose the decision without changing the shader ABI or resource
ownership. Every encoded build publishes transient buffer and bind-group creation counts; parameter,
page-plus-candidate, Mesh SDF, and completion-buffer upload bytes; and their checked total alongside
the existing CPU phases, page/defer/fallback counters, persistent atlas bytes, and timestamp-profiled
`runtime_prepare.hybrid_gi.global_sdf_build` pass. The deterministic one-page contract uses one
64-voxel Mesh SDF and establishes 7 buffers, 1 bind group, 16 B parameters, 36 B page/candidate
data, 432 B Mesh data, 4 B completion initialization, and 488 B total. This is a byte-layout
regression, not a timing or throughput result. Empty, fallback-only, and readback-backpressured
batches report zero transient creation and upload metrics because they encode no Global SDF work.

The ignored `export_global_sdf_build_wgpu_png` test is the narrow dynamic evidence entry point for
this build owner. It creates the production Global SDF resource set, runs `cs_build_global_sdf`,
requires the completion word and signed inside/outside atlas values, and writes an enlarged signed-
distance slice to `docs/tests/runtime/render`. Its exporter explicitly requests DX12 so the
coordinator can capture the exact test process with `D:\Tools\renderdoc\renderdoccmd.exe`; it is
not a substitute for the collector-to-trace product capture required by the M5 failure handoff.

Lumen supports the same lifetime conclusion but not a direct implementation copy. Its radiance
cache exports long-lived state buffers and atlases as external graph resources after update, while
RDG still allocates bounded indirect/work buffers for an update. Zircon retains its smaller WGPU
owner: device state stays persistent, selected page-build inputs remain frame/in-flight owned, and
a Mesh SDF GPU atlas is not introduced merely because Unreal has external resources.

### Cache Decision Gate

Coordinator-managed Windows measurement must use one identical scene and adapter in four modes:

1. cold initialization with all eligible pages dirty;
2. 300-frame settled static camera, then 31 warm samples;
3. camera-only clipmap scroll with unchanged Mesh SDF resource revisions; and
4. one authoritative Mesh SDF revision invalidation followed by recovery to the same settled frame.

For every sample retain the existing Global SDF CPU phase timings, GPU pass duration, dispatched/
uploaded/deferred/fallback pages, readback backlog, output hash, and the new allocation/upload
breakdown. Capture a same-revision PNG below `docs/tests/runtime/render` and a DX12 RenderDoc
capture through `D:\\Tools\\renderdoc` for the cold and camera-scroll cases. A persistent Mesh SDF
GPU cache may be designed only when these measurements show repeated upload of unchanged Mesh SDF
data across camera-only page rebuilds and identify Mesh bytes or transient object creation as a
material source of observed cost. The result must preserve typed fallback for missing, invalid,
morphing, skinned, or upload-budget-exceeded assets.

If that gate is met, the follow-on design must be a new per-instance owner with immutable content
entries keyed by authoritative asset/revision identity, explicit capacity/eviction accounting, and
in-flight-safe replacement or slot retirement. Per-object world transforms stay transient. It may
not mutate a payload buffer still referenced by a pending Global SDF completion, infer validity from
frame number, or turn typed fallback into an unbounded retry path.

## Test-First Contract

Before the move, add a focused source contract that proves:

1. the trace-tile generation shader module, pipeline layout, and compute pipeline are created by
   the `HybridGiGpuResources` construction path;
2. `scene_prepare_trace_tiles` consumes the device-owned pipeline and does not call
   `Device::create_shader_module`, `create_pipeline_layout`, or `create_compute_pipeline`;
3. tile CPU planning and dispatch dimensions are byte-for-byte unchanged; and
4. the normal prepare path still retains its frame buffers in `HybridGiGpuPendingReadback`.

This proves the lifecycle transfer without pretending it is a benchmark. The coordinator must
later run the normal Rust/WGPU suite on the resulting snapshot.

## Measurement Protocol And Acceptance

Use one current-source scene and adapter in two modes: an empty/no-Surface-Cache fallback scene
and a populated, stable Surface Cache scene. Capture a cold first frame and a warm second frame;
then collect 31 warm samples after a 300-frame settling run. Each sample records CPU prepare phase
time, HGI device-object creation counters, buffer/texture/view/bind-group creation counts, upload
bytes, readback backlog, Global SDF page and fallback counts, frame time, and output pixel hash.

Plan17's timestamp-query facility is the required source for per-pass GPU duration. Until that
facility is active, RenderDoc action counts are only a resource/copy diagnostic, not GPU time.
For the same source revision, coordinator evidence must include a WGPU readback, an inspected PNG
under `docs/tests/runtime/render`, and cold/warm DX12 RenderDoc captures replayed with
`D:\Tools\renderdoc\renderdoccmd.exe` plus the repository audit script. GPU-time and upload-byte
reduction may be used as energy proxies; no power conclusion is allowed without platform power
telemetry.

The first slice is accepted only if stable frames report zero trace-tile-generation
pipeline/layout/shader creation after device initialization, retain identical dispatch geometry and
fallback diagnostics, and show no PNG/readback regression. The frame-ring decision requires the
separate counters above and must not be justified by an assumed percentage improvement.
