---
date: 2026-08-24
related_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
related_runtime90_plan: docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
doc_type: current-source-architecture-review-and-measurement-plan
status: current_source_review_complete_runtime_baseline_pending
coordination_owner: docs/plans/zircon_runtime/render/17
---

# WGPU Diagnostic Submission Architecture And Measurement Plan

## Status

This record is a current-source architecture review and measurement plan. It is not an accepted
milestone or a performance result. M5's buffer/color-texture diagnostic readback foundation, M6's
neutral submission packet, and the first submission-bound timestamp/pipeline-statistics query
path are implemented at source level, but managed Cargo validation, product execution, RenderDoc
capture, GPU timestamps, power telemetry, PNG, and RDC evidence have not run in this work item.
No frame-time, throughput, energy, or image-quality result is claimed.

The production diagnostic owner now has one device-generation-local `DiagnosticReadbackTracker`,
one submission-ticket binding, checked staging layout, bounded request/byte admission, and a
single device-poll boundary. Buffer/texture copies and timestamp/pipeline-statistics queries share
that tracker, so query requests cannot double the per-frame or in-flight diagnostic quota. The
WGPU query service owns only query sets, resolve/staging buffers, map callbacks, and ticket-ordered
delivery; it cannot submit or poll. Legacy timer/statistics owners have not been removed because
the product raw path has not migrated to the neutral packet route.

The product startup path now derives its device request policy before device creation. When
`SceneRendererStartupOptions::with_gpu_timing()` is selected, it requests `GpuTimestamp` and
`PipelineStatistics` as optional features through `RenderBackend::new_offscreen_with_policy`;
the default startup policy remains the empty MVP baseline. This corrects the former ordering bug
where legacy timers were created after an MVP-only device had already been negotiated. It does not
move any product submission, poll, resource, or readback owner to the neutral route and is not a
performance result.

The legacy `GpuPipelineStatisticsTimer` remains a product owner pending hard cut, but now follows
the same native WGPU ABI as the packet path: one query-set index per recorded scope and five
resolved `u64` counters per index. Its query-set capacity, resolve range, staging byte count, and
decoder now use those two quantities separately. This is a correctness repair to the existing raw
owner, not a duplicate diagnostic service or a product migration claim.

The production neutral command path now encodes `BufferToTextureCopy` and
`TextureToBufferCopy` for color textures. Both operations are advertised as `Native` only because
their WGPU encoding, resource usage validation, checked copy extent, buffer range, and padded-row
rules are implemented together. They remain command copies rather than public CPU readback:
`supports_buffer_readback` remains false. M11 later added an explicit neutral texture-copy aspect:
only `Depth32Float` texture-to-buffer work through `DepthOnly` is admitted, while unsupported
depth/stencil directions remain rejected rather than acquiring an implicit aspect. This is
source-level status only; no managed execution receipt exists.

S1 now also has a ticket-qualified CPU-to-GPU color-texture update path. `RenderDevice::write_texture`
accepts a mip/layer-or-slice `TextureCopyRegion`, source row pitch, and bytes; the deterministic
contract and `WgpuRenderDevice` both validate the region, `COPY_DST` usage, color-only aspect, row
pitch, and effective source length. WGPU queue texture writes accept tight multiline rows, unlike
encoded buffer-to-texture copies, so the two layout rules are intentionally separate. The
submission service retains only bytes through the final effective pixel, charges that amount to the
existing upload budget, keeps the native texture alive through the same ticket, and serializes the
write with command submissions under its sole queue lock. The added deterministic and real-WGPU
regression definitions cover row padding, ticket lifecycle, and tight rows, but have not received
managed execution validation. No raw product texture upload has been migrated, and this is not a
performance or product-frame result.

M6 now provides `zr_rhi::RhiSubmissionPacket`: an immutable collection of command lists qualified
by `DeviceId`, `DeviceGeneration`, and one `RenderQueueClass`. `RenderDevice::enqueue_command_list`
is a convenience path that constructs a one-list packet, while both the deterministic contract
backend and `production::WgpuRenderDevice` accept a multi-list packet as one logical submission
ticket. The WGPU owner retains a `Vec<wgpu::CommandBuffer>` for the ticket and joins it only at its
single native submit service. This removes the previous API-level pressure to synthesize one ticket
per command list, but it is not a product cutover or a measured batching claim.

The same source increment adds `BeginComputePass { label }` / `EndComputePass` and diagnostic
render/compute variants to the neutral command ABI. A labelled compute scope may contain several
dispatches and has the same balanced debug-group discipline as a render pass; the production
backend encodes one native compute pass for that scope, while a legacy unscoped dispatch remains
valid for simple command lists. `DiagnosticQueryPlan` now belongs to `zr_rhi`: the graph compiler
assigns dense pass IDs and bounded query ranges before it creates an immutable packet; WGPU then
encodes begin/end timestamps at encoder scope, begins/ends pipeline-statistics queries inside the
native pass, resolves both sets into the same packet, and maps the result after normal submission
polling. Unsupported optional query features and budget rejection emit bounded `Unavailable` or
`OverBudget` query deliveries without cancelling the rendering packet. These are source-level
claims only; no runtime timing conclusion follows from them.

Indirect command foundation source completion, 2026-08-24: the neutral `CommandList` contract
now has seven operations: `DrawIndirect`, `DrawIndexedIndirect`, `MultiDrawIndirect`,
`MultiDrawIndexedIndirect`, `MultiDrawIndirectCount`, `MultiDrawIndexedIndirectCount`, and
`DispatchComputeIndirect`. The shared argument validator requires `BufferUsage::INDIRECT`, a
four-byte-aligned offset, and the exact WGPU argument range (16 bytes per non-indexed draw, 20
bytes per indexed draw, and 12 bytes per compute dispatch). The count-buffer validator requires
one aligned four-byte `u32`, while the argument range covers all `max_count` tightly packed
draws. Deterministic admission and the production WGPU encoder consume these shared validators;
production submission retains every argument and count buffer through its exact ticket. Raster
indirect commands are admitted only when the originating adapter reports
`DownlevelFlags::INDIRECT_EXECUTION`; the count-buffer variants also require the negotiated
`MULTI_DRAW_INDIRECT_COUNT` feature. The production capability receipt rejects the corresponding
operation before encoding when either prerequisite is absent. `INDIRECT_FIRST_INSTANCE` remains a
separate negotiated capability, so product argument generation must not write a nonzero
first-instance field without it. This lays the RHI prerequisite for Plan 02's fixed-count and
compaction replay paths and Plan 16's compute-generated dispatches, but it does not migrate the
remaining raw product WGPU calls. Deterministic regression definitions cover recording,
usage/alignment/range rejection, explicit compute-pass dispatch, and count-buffer range
rejection. Managed Cargo/GPU execution, product frame migration, RenderDoc/PNG/RDC artifacts,
performance or power results, milestone acceptance, and coordinator submission remain pending.

Pipeline-statistics ABI correction: one diagnostic statistics scope owns one native query-set
index, whose resolve payload is five `u64` counters. The neutral plan therefore counts native
statistics queries separately from resolved counter values; WGPU sizes the resolve and staging
buffers from the latter and resolves only the former. This prevents a fivefold query-index range
with an undersized native resolve buffer. The new neutral and optional-feature WGPU regression
definitions cover this contract, but have not yet received managed execution validation.

M7 source completion, 2026-08-24: the neutral RHI now owns generational `TextureViewHandle` and
`TextureViewDesc` objects with explicit dimensions and mip/layer ranges. Typed sampled-texture and
sampler binding declarations encode in the private production WGPU registry, retain both view and
parent texture through submission tickets, and reject parent destruction while any logical view is
live. The deterministic owner uses a per-parent view count, so texture destruction is O(1) rather
than scanning all views. M8 source completion, 2026-08-24: `StorageTextureBindingDesc` now carries
write-only access, exact format, and view dimension; deterministic and production WGPU owners admit
only the current runtime-compute subset (`Rgba8Unorm` and `Rgba16Float`, D1/D2/D2Array/D3,
single-sampled `TextureUsage::STORAGE` parents) and reject format or dimension mismatches at
layout/bind-group boundaries. Read/read-write storage and depth/stencil aspect views remain
explicitly deferred. Deterministic and optional real-WGPU test definitions cover valid binding,
range rejection, parent teardown, filterability, storage-format mismatch, unsupported sRGB
storage, and a WGSL write-only-storage compute dispatch through the neutral submission path.
This is source completion only: managed Cargo, product-frame execution, RenderDoc, PNG/RDC
capture, performance, and power evidence remain pending, so no milestone acceptance or
coordinator submission is claimed.

M9 source completion, 2026-08-24: `TextureDesc::view_formats` now declares alternate
shader-visible formats at allocation time and `TextureViewDesc::format` selects one at view
creation time. The neutral MVP admits only the portable WGPU sRGB pairs
`Rgba8Unorm`/`Rgba8UnormSrgb` and `Bgra8Unorm`/`Bgra8UnormSrgb`; repeated parent formats,
duplicate alternates, incompatible reinterpretations, and undeclared view formats fail before a
native object is created. The production WGPU registry maps the declared list to
`TextureDescriptor::view_formats` and the selected view format to `TextureViewDescriptor::format`.
Bind-group validation now uses the effective view format, so the runtime mip-generation shape can
sample an sRGB view while the same parent exposes a linear `Rgba8Unorm` storage view; attempting
to bind the sRGB view to that storage layout is rejected. This decision follows WGPU 29's
sRGB-only view reinterpretation rule, Bevy's `TextureSrgbViewFormats` mapping, and the Lumen
reference's distinct backing/SRV format intent without adding a backend-specific typeless resource
API. Deterministic and optional production-WGPU test definitions cover the valid dual-view route,
native sampled binding, undeclared view rejection, and all three invalid declaration forms. These
definitions have not received managed execution validation. No raw-product migration, product
frame, RenderDoc capture, PNG/RDC artifact, performance, energy, or milestone acceptance is
claimed, and no coordinator submission is due.

M10 source completion, 2026-08-24: `TextureViewDesc` now carries the neutral
`TextureViewAspect` (`All`, `DepthOnly`, `StencilOnly`). Shared deterministic/production view
validation admits `DepthOnly` only for a depth format and `StencilOnly` only for a depth-stencil
format; color textures remain `All`. Typed binding derives sample type from the selected aspect:
the explicit depth view binds as `Depth`, the stencil view binds as `Uint`, and an `All`
depth-stencil view is rejected as an attachment-only view rather than being falsely treated as a
shader resource. The production registry maps this directly to WGPU's `TextureAspect`; this
matches Bevy's separate combined/depth/stencil view ownership and the Lumen reference's backing
depth-stencil resource with an independently chosen SRV format. At M10, plane/YUV aspects and
depth/stencil copy were deliberately deferred; M11 subsequently added an explicit copy aspect for
the constrained `Depth32Float` source path only. Deterministic
and optional real-WGPU test definitions cover depth/stencil binding, combined-view rejection, and
both invalid-aspect diagnostics. These definitions have not received managed execution validation.
No raw-product migration, product frame, RenderDoc capture, PNG/RDC artifact, performance, energy,
or milestone acceptance is claimed, and no coordinator submission is due.

M11 source completion, 2026-08-24: `TextureCopyRegion` now carries a distinct neutral
`TextureCopyAspect` (`All`, `DepthOnly`, `StencilOnly`) rather than reusing view intent. The
linear-copy contract admits color texture copies only with `All` and admits `Depth32Float`
texture-to-buffer copies only with `DepthOnly`; the latter must copy the complete mip extent in
X/Y, matching WGPU's depth-stencil partial-copy restriction. Directional validation is shared by
the deterministic command executor, WGPU command encoder, and ticket-qualified queue-upload
entry: WGPU's portable depth formats remain write-prohibited, `Depth24Plus` remains non-copyable,
and `Depth24PlusStencil8` stencil-plane support remains deferred until the deterministic owner has
an independent per-aspect storage layout. This choice follows WGPU 29's linear-copy validation,
which permits a `Depth32Float` source but forbids it as a destination. Direct command-copy test
definitions cover the valid explicit-depth path and default-`All` rejection in deterministic and
optional production-WGPU backends. The diagnostics receipt service remains color-only; it does
not promise a depth-to-CPU conversion. These definitions have not received managed execution
validation. No product frame, RenderDoc/PNG/RDC artifact, performance/energy result, milestone
acceptance, or coordinator submission is claimed.

Texture migration prerequisite source completion, 2026-08-24: the neutral command ABI now has
`TextureToTextureCopy`, with independent source and destination `TextureCopyRegion` values.
Deterministic execution and the production WGPU encoder share one validation rule: source and
destination are distinct, single-sampled, same-dimension, same-format color textures; both use
the `All` aspect and have equal copy extents. The production registry retains both textures through
the one submission ticket. The deterministic render/compute state machine stays in
`command_validation.rs`; its four transfer validators and executors have one folder-backed owner
in `command_validation/copy_commands.rs`. This is sufficient for the existing physical mip-tail replacement path
without reintroducing a direct texture-copy submit, but it deliberately does not migrate the raw
product texture owner, permit self copies, reinterpret formats, or add depth/stencil transfers.
The deterministic subregion preservation/rejection regressions and optional production-WGPU
diagnostic-readback regression are source test definitions only. Managed Cargo, product-frame
execution, RenderDoc, PNG/RDC artifact, performance/energy data, milestone acceptance, and a
coordinator submission remain pending.

Device-context handoff prerequisite source completion, 2026-08-24: the neutral production owner
now accepts one opaque `WgpuRenderDeviceContext` carrying the negotiated `Instance`, `Adapter`,
`Device`, `Queue`, and UI shared-image registry. `WgpuRenderDevice` retains that generation state,
verifies the profile's complete neutral adapter facts, actual native enabled-feature receipt,
negotiated device limits, and fixed WGPU single-serialized queue topology before installing its
WGPU error supervisor, and returns `RhiError::NativeContextAdapterMismatch`,
`RhiError::NativeContextRequestedFeaturesMismatch`,
`RhiError::NativeContextDeviceLimitsMismatch`, or `RhiError::NativeContextQueueTopologyMismatch`
on a mismatch. It can derive a
same-generation `WgpuUiSurfaceContext` without publishing raw device or queue getters. This is a
construction boundary needed for the Runtime90 product hard cut; it does not yet move `RenderBackend`,
`ResourceStreamer`, scene passes, raw texture objects, or legacy product submissions onto the
neutral owner. The production test helper and a source contract define the handoff, but managed
Cargo, product execution, RenderDoc/PNG/RDC, performance/energy data, milestone acceptance, and
coordinator submission remain pending.

Render-pass attachment subresource prerequisite source completion, 2026-08-24: neutral
`RenderPassTextureViewDesc` now reaches the production WGPU encoder with its declared mip level
and array layer intact. The backend materializes one `D2` native view over exactly that mip and
one array layer for `D2`, `D2Array`, and `Cube` textures; it does not publish the native view to
the render graph. Shared attachment validation rejects `D1` and `D3` before native encoding,
while retaining the existing bounds, extent, sample-count, resolve, format, and duplicate-binding
checks. Deterministic coverage defines the portable rejection contract, and an optional
production-WGPU regression clears then diagnostic-readbacks mip 1, array layer 1 from a two-layer
target. These are source test definitions only: no managed Cargo or GPU execution, product frame,
RenderDoc/PNG/RDC artifact, performance/energy result, milestone acceptance, or coordinator
submission is claimed.

MVP neutral offscreen-frame source completion, 2026-08-24: `WgpuMvpOffscreenTriangle` is a
reusable production owner for the minimal clear-and-triangle frame. Its construction retains only
neutral color/depth texture, shader, and pipeline handles; the raster pipeline uses a
`Depth24Plus` `LessEqual` depth state and its one graphics `CommandList` clears color and depth
together. Its ticket is issued by `WgpuRenderDevice` rather than a raw queue. The color output
remains a neutral texture handle for the device-owned diagnostic path or later graph composition,
and destruction follows pipeline, shader, layout, depth texture, then color texture dependency
order. This creates no
wrapper around the raw scene renderer and does not make the legacy product renderer use the new
owner yet. Its optional production-WGPU source regression checks ticket completion and pixel
readback; managed Cargo, application integration, product execution, RenderDoc/PNG/RDC,
performance/energy data, milestone acceptance, and coordinator submission remain pending.

Neutral MVP product bootstrap source completion, 2026-08-24: `NeutralMvpRenderer` now performs
the existing cold-path adapter selection and negotiated device request, then transfers the newly
created `Instance`, `Adapter`, `Device`, and `Queue` directly into `WgpuRenderDeviceContext`.
It creates `WgpuMvpOffscreenTriangle` only after that one neutral device owner has installed its
supervisor, and every frame returns the device-owned submission ticket. Its narrow public facade
can capture one rendered frame as tightly packed RGBA8 bytes only through a serialized,
request-qualified diagnostic readback; its deadline starts before capture-state contention and is
checked around submission waits and nonblocking completion pumps, although no synchronous backend
call can be claimed as a hard real-time bound. Timed-out requests retain their exact
request/frame identity until a later call reaps that same delivery, and rejection/cancellation
closes the active diagnostic frame. It exposes no native device or queue. The graphics-gated
`zircon_neutral_mvp_capture` binary converts that result to the fixed repository artifact path
`docs/tests/runtime/render/plan17_wgpu_neutral_mvp_triangle_current.png`, but this source addition
has not been executed. The bootstrap neither constructs `RenderBackend` nor retains a raw
queue/device or installs a second supervisor; `GraphicsError::Rhi` retains a typed RHI construction
failure rather than converting it to a text validation error. This is the first product-facing
minimum path, not a migration claim for the legacy scene renderer, surface presenter, or resource
streamer. Managed Cargo, application entry selection, product execution, RenderDoc/PNG/RDC,
performance/energy data, milestone acceptance, and coordinator submission remain pending.

### Product hard-cut inventory and sequencing, 2026-08-24

Before changing any product texture algorithm, the current graphics source was mechanically
scanned with files named `*test*` and paths under `tests/` excluded. This is structural inventory,
not profiling: it found 616 `wgpu::Device`, 213 `wgpu::Queue`, 1,605 `wgpu::Texture`, 713
`wgpu::TextureView`, and 158 `wgpu::CommandEncoder` references. The same scan found 35
`.submit(` occurrences across 23 non-test files, 64 `.write_texture(` occurrences across 34
files, 65 `.create_texture(` occurrences across 45 files, 242 `backend.device`/`backend.queue`
occurrences across 50 files, and 65 `GpuTextureResource` occurrences across 18 files. Inline
test-only branches in production-named files can remain in these counts, so they are an upper
bound on product call sites rather than an execution sample or a performance result.

The immediate texture path spans `RenderBackend` native ownership, `ResourceStreamer` construction
and revision/mip residency, fallback and output textures, material/bindless lookup, scene/UI
sampling, and compiled-frame recording. `RenderBackend` currently installs the one raw WGPU fault
supervisor itself; creating a second `WgpuRenderDevice` alongside it would replace those callbacks
and split queue responsibility. Therefore the next product migration unit must replace, in one
cut, the frame owner plus the resources and binding model it records. It cannot be a wrapper around
`GpuTextureResource`, an additional queue facade, or a texture-only conversion.

The Lumen compute reference confirms the useful part of the design: a frame owns ordered resource
use, history transitions, and submission boundaries. Its direct D3D12 barriers and multiple
`SubmitCommandList` calls are backend details that Zircon must not copy into WGPU. Zircon retains
RenderGraph for dependency/order declaration and uses one neutral submission service to materialize
the resulting packet. The hard-cut sequence is consequently: (1) transition the product frame
owner and its resource/binding consumers to neutral handles and command lists, (2) transfer the
sole negotiated device context to `WgpuRenderDevice`, (3) move upload, readback, capture, and
present to ticket-qualified producers, and only then (4) delete the raw `RenderBackend` resource
and submit paths. No throughput, power, or image-quality optimization is authorized from this
inventory; those measurements follow the product cut and same-generation screenshot/RDC evidence.

## Source Evidence

### Current structural counts

The following are static source counts, not execution samples. Scope is non-test Rust under
`zircon_runtime/crates/zr_rhi_wgpu/src`, collected on 2026-08-24:

| Pattern | Count | Interpretation |
| --- | ---: | --- |
| `queue.submit(` | 7 | Submission authority is not yet singular across the full crate. |
| `device.poll(` | 1 | A direct poll remains outside the new production device boundary. |
| `wait_indefinitely` | 1 | A legacy blocking path remains and cannot be called an asynchronous pipeline. |
| `GpuReadbackQueue` | 13 | Legacy queue ownership remains in timer/statistics paths. |
| `to_string()` | 145 | Broad allocation census only; it does not identify a hot path by itself. |

The new `production/diagnostics/readback/` folder is split by ownership: `request` validates
neutral source metadata, `layout` owns padded texture-row math, `batch` owns submission-local
staging metadata, `completion_order` retains out-of-order map callbacks behind the oldest
diagnostic ticket, `delivery` owns the native-free public result, and `service` owns admission,
mapping, exact-once terminalization, and bounded completion retention. Its eight source files
are 22--415 lines each; no production file was added above the 800-line soft budget.

The deterministic RHI contract device now has 793 lines in its parent module. Its resource and
submission-accounting state lives in `device/state.rs`, its host-memory admission/allocation and
upload execution live in `device/resources.rs`, its texture-view lifetime and per-parent view
count live in `device/views.rs`, and its WGPU capability table lives in `device/contract_caps.rs`;
this keeps independently-owned lifecycle code out of the parent while keeping state accounting and
capability policy separately reviewable. The packet regression suite
covers both the deterministic execution path and the real WGPU production path, including mixed
queue rejection before ticket admission. These are source test definitions and static checks, not
executed test receipts.

### Current bottleneck hypotheses, not conclusions

1. `GpuPassTimer` retains `Vec<String>` pass names and delegates resolve mapping to
   `GpuReadbackQueue`. `GpuPipelineStatisticsTimer` does the same, then merges same-name scopes
   by linear search. This makes diagnostic cost depend on string allocation and repeated search
   rather than a compiled pass index.
2. `zr_rhi::DiagnosticQueryPlan` now establishes bounded dense pass IDs and O(N) aggregation for
   timestamp/statistics payloads. Its native WGPU consumer is packet-bound, but the legacy
   string-named timer/statistics paths still duplicate work and must be hard-cut only with product
   migration evidence.
3. The new M5 service batches mixed buffer and color-texture copies into one submission-qualified
   staging buffer and starts maps only through `WgpuRenderDevice` after its normal submission
   service poll. Native map callbacks may become ready out of order, so the completed staging
   buffer remains bounded by the existing in-flight quota until all earlier diagnostic tickets
   reach a terminal state; only then is it unpacked and delivered. This removes a new duplicate
   owner, but it does not prove the older owners are gone or that any runtime cost improved.

### F3/F16 frame-submission structural review, 2026-08-24

This is a source review and measurement plan for review findings F3 (frame-extract copies) and
F16 (compiled-scene render ownership). It is not a profiling result. No Cargo, product renderer,
Tracy/Chrome trace, allocation trace, WGPU timestamp sample, RenderDoc capture, PNG, RDC, or
power measurement ran for this review. Consequently none of the candidates below is called a
bottleneck and no algorithm change is authorized by this record alone.

#### Current ownership result

F16's requested split is already present in the compiled-scene path and must not be reimplemented
as another facade:

| Boundary | Current owner | Responsibility |
| --- | --- | --- |
| Frame resource setup and graph inputs | `scene_renderer_core_render_compiled_scene/render/render.rs` | Frame encoder, shared readback/timer admission, runtime preparation, graph resources, and final frame record. |
| Graph-stage execution | `.../render/execute_compiled_scene_graph_stages.rs` and `execute_graph_stage.rs` | Prepared-pass execution, graph resource use, pass profile data, and optional parallel encoder buckets. |
| Submission and present-adjacent finalization | `.../render/submit_compiled_scene_frame.rs` | Command-buffer finish/submit, post-submit mapping, transient release, and explicit completion ordering. |

The framework path is separately divided into `camera_loop.rs`, frame-context construction,
runtime preparation, `build_runtime_frame.rs`, and record/update owners. Therefore the F16 source
remediation is structurally complete; the remaining work is product validation of the existing
boundary, not another function split.

F3 is partially remediated already. A normal multi-camera loop owns one `Arc<RenderFrameExtract>`;
`FrameSubmissionContext` retains an `Arc` to the same source, so meshes, lights, particles, and
the majority of the extract are not cloned once per camera. `CameraLoopExtractSourceState` moves
the heavy virtual-geometry and hybrid-GI source payloads out once and passes borrowed views of them
to each camera submission. This makes an alleged whole-`RenderFrameExtract` per-camera clone an
incorrect current-source description.

#### Remaining measurable candidates

The remaining copy sites are intentionally limited but can still scale with scene or camera-stack
size. They need measurements before removal or redesign:

1. `CameraLoopPostProcessSourceState::capture` clones `volumes`, `stack`, and `graph`; every
   subsequent camera restores those values through `clone_from` or clone assignment. This cost is
   proportional to post-process volume and graph-descriptor size times additional camera
   submissions.
2. `build_frame_submission_context_from_source` materializes an effective virtual-geometry extract
   from a cloned authored source when that feature is enabled. `RenderVirtualGeometryExtract`
   contains clusters, hierarchy nodes/IDs, pages, dependencies, and instances. The HGI settings
   copy is small today, but it belongs to the same measurement scope because its contract may grow.
3. An on-demand planar reflection that is actually scheduled clones `extract.view.cameras` once to
   append the derived capture camera. It is not a per-camera steady-state copy, but scenes with
   frequently dirty probes may make it visible.
4. `build_runtime_frame` clones frame visibility and the previous motion-vector camera for its
   per-camera output DTO. Debug overlay construction clones overlays only when virtual-geometry
   debug output is active; it must be profiled separately from normal gameplay rendering.

The existing source tests prove camera ordering, UI routing, derived-state restoration, and the
direct terminal lookup. They do not prove allocation volume, copy bytes, p95 CPU time, or product
image equivalence for a densely populated multi-camera frame. The current `FrameProfiler` has
whole-submit CPU time and graph-pass profiles, while the existing `profile_scope!` regions already
separate submit, context build, runtime preparation, graph render, and feedback. Those spans are
the first measurement mechanism; a parallel profiler DTO or a speculative cache is not justified.

#### Reference-engine comparison and constrained design direction

- Unreal's `FRDGBuilder::Compile` establishes dependency/resource ownership before execution, and
  `FRDGBuilder::Execute` owns the graph prologue, resource collection, compiled pass work, and
  execution diagnostics. Zircon should preserve its existing graph-stage owner rather than move
  graph compilation into a camera loop.
- Unity Core's `RenderGraph` has an explicit record -> execute -> cleanup lifecycle. Its render
  graph test pipeline records and executes one graph for each enabled camera, then submits the
  command buffer after the graph lifecycle has completed. Zircon's multi-camera policy should
  likewise create a compact per-camera descriptor while keeping the source frame and graph
  lifecycle explicit.
- Bevy's `RenderGraph` schedule separates begin, render, submit, and finish; its `render_system`
  runs the graph, records screenshot/readback work into a follow-on encoder, then presents. This
  confirms that diagnostics and presentation should remain outside the graph compilation owner.
- The local Lumen compute reference keeps resource binding in pipeline objects but calls a native
  submit from `ComputePipeline::Execute`. It is useful for pass-resource decomposition only; that
  immediate-submit pattern is explicitly not a Zircon frame-submission model because it violates
  the single packet/ticket owner required by this plan.

If measurement ranks one of the copy candidates above as material, the only permitted structural
direction is a hard internal conversion from mutable source restoration to:

```text
immutable shared frame source (heavy scene and authored feature payloads)
    + compact per-camera submission descriptor
    -> per-camera derived context and output DTO
    -> existing compiled graph-stage owner
    -> existing single frame submit/ticket owner
```

The descriptor may own only camera selection, target/extent, output policy, terminal-UI ownership,
and derived post-process/temporal inputs. It must not own a cloned scene extract, raw WGPU object,
queue, encoder, or a second graph compiler. This is a hard internal migration: do not add an
adapter that presents both mutable and immutable submission paths after the cut.

#### Measurement and decision gates

1. On the managed Windows product backend, capture cold and warm runs separately at fixed 1080p
   with deterministic camera motion: one primary camera, a multi-camera stack, and a dirty-planar
   variant. Discard 60 warm-up frames and retain at least 300 raw measured frames per variant.
2. Record existing CPU spans for `submit_frame_extract`, `build_submission_context`,
   `prepare_runtime_submission`, `render_frame_with_pipeline`, and `collect_runtime_feedback`.
   Preserve raw samples and report median/p95 with camera count, post-process volume count,
   virtual-geometry cluster/page/instance counts, HGI state, and debug state. Use a CPU allocation
   trace only when the context/loop span ranks materially; do not infer allocation bytes from clone
   syntax.
3. In the same source fingerprint, export the `RenderFrameProfile`, capture a product PNG under
   `docs/tests/runtime/render/`, and collect a matching `D:\Tools\renderdoc` RDC. Timestamp-
   enabled and disabled runs must retain identical graph order, command-buffer order, fallback
   state, and output pixels before their timings can be compared.
4. Before a frame-source migration, add behavior tests for populated post-process state, VG/HGI
   sideband reuse across multiple cameras, planar-capture ordering, terminal-UI routing, and
   debug-on versus debug-off output ownership. The existing stream/restore test remains a required
   control until the old path is hard-cut.
5. Only a ranked candidate with matching-image evidence may enter an implementation slice. That
   slice must show before/after raw profiles, allocation data when applicable, and a fresh PNG/RDC
   pair. Power is reported only when a synchronized platform telemetry source is available; frame
   time and GPU timestamps are not power measurements.

### S1 contract convergence status

Static review found and corrected an existing neutral-contract divergence: deterministic
`RenderDevice::write_buffer(...)` had modeled `STAGING_WRITE`, while the production WGPU
queue-write implementation required `COPY_DST`. Both implementations and their upload fixtures
now require `COPY_DST`; the regression definition explicitly rejects a `STAGING_WRITE | COPY_SRC`
resource at the queue-upload entry point. WGPU prohibits a `MAP_WRITE` buffer from carrying
arbitrary copy usages, so accepting both flags as a compatibility shortcut would conceal an
invalid native descriptor rather than converge the interface. `STAGING_WRITE` remains reserved
for a later explicit map/unmap API. This corrects source-level S1 contract parity only: it does
not migrate raw product buffers and is not a measured bottleneck or performance result.

### Product migration census

Before product migration, a static scan of non-test-path Rust under `graphics/backend`,
`graphics/scene`, and `graphics/runtime/render_framework` found 612 `wgpu::Device` references,
207 `wgpu::Queue` references, 27 command-encoder creations, 90 texture creations, 149 buffer
creations, 48 render-pipeline creations, and 15 compute-pipeline creations. The same broad scan
found 33 `queue.submit` textual matches, four `device.poll` matches, and eleven
`wait_indefinitely` matches. These are upper bounds because several source files contain inline
test guards, but they establish that a one-file wrapper is not a legitimate hard cut.

The 21 currently identified non-test native submit owners fall into five migration strata:

| Stratum | Current owner examples | Required hard-cut destination |
| --- | --- | --- |
| S0 synchronous inspection | `render_backend/read_*`, `graphics_debugger_capture` | M5 diagnostic readback receipt and capture service; no inline `wait_indefinitely`. |
| S1 upload/writeback | GPU texture asset upload, output-target writeback, history construction | neutral resource upload/copy packets, then retirement by submission ticket. |
| S2 compiled scene frame | `submit_compiled_scene_frame.rs` | one graph-materialized `RhiSubmissionPacket`; it becomes the single normal-frame ticket. |
| S3 executor side submits | generic compute, parallel encoder set, scene clear, UI atlas | executor records only neutral command lists; the compiled frame joins and submits them. |
| S4 cold-path and recovery poll | IBL writeback and mesh pipeline cache | device-generation completion owner; cold cache status is observable but may not poll the device directly. |

S2 is not safe to cut first: it consumes command buffers produced by S1/S3 code that directly
owns WGPU resources, pipeline objects, and encoders. The correct dependency order is S0, then S1,
then S3, then S2, followed by S4. Only then can `RenderBackend` drop its raw device/queue fields
without a compatibility accessor. This is an architecture result from source inspection, not an
estimate of runtime cost.

The first concrete S1 product candidate is `gpu_texture_resource_from_asset.rs`: its uncompressed
RGBA8 and RGBA16F per-mip/per-layer writes are representable by the new neutral texture-upload
contract. The previous binding blocker is now implemented at source level: `TextureViewDesc` and
its generational handle carry explicit mip/layer ranges and D1/D2/D2Array/D3/Cube/CubeArray view
dimensions; typed sampled-texture and sampler layout entries encode to WGPU only through the
production registry. Bind-group use retains the view and its parent texture through the ticket,
and parent destruction fails while a live view remains. The registry maintains a per-parent
live-view count rather than scanning the global view table during destruction; this is an
ownership-complexity correction, not a measured performance claim. This follows UE's separate texture/SRV
model without exposing native WGPU objects or introducing a second queue owner. It is still not a
product hard cut: `GpuTextureResource` stores raw WGPU objects, sampler-cache ownership, runtime
mip generation, and compressed-payload paths that require a higher-level neutral material/image
resource conversion. The write-only `Rgba8Unorm`/`Rgba16Float` storage subset is now represented
by neutral layout descriptors, but raw product compute still owns its WGPU pipeline/layout/view
creation; read/read-write storage, wider format support, and compressed block payloads remain
separate contracts. No managed execution receipt exists for this new source slice.

### S1 texture hard-cut prerequisite review, 2026-08-24

The current `GpuTextureResource` cannot be migrated independently without violating the ownership
rules above. It owns raw `wgpu::Texture`, `TextureView`, cached `Sampler`, and native bind group;
`ResourceStreamer`, material/bindless binding, compiled-scene draw encoding, physical mip streaming,
and the runtime mipgen path consume those objects directly. Meanwhile `RenderBackend` owns raw
`wgpu::Device`, `Queue`, and an error supervisor, while `WgpuRenderDevice::new(...)` is deliberately
an owning device-generation boundary with its own queue submission service and supervisor. Adding an
adapter that exposes registry-native handles, or adding another `WgpuRenderDevice` around cloned
product handles, would create the dual ownership and submit/poll ambiguity this plan is intended to
remove. It is not an acceptable partial migration.

The required hard-cut order is therefore:

1. Convert compiled-scene material resource lookup and draw/compute recording to neutral handles
   and `CommandList` records; the production registry remains the sole native lookup point.
2. Move the render-backend device-generation ownership to one `WgpuRenderDevice` and remove the
   duplicate raw supervisor/normal queue-submit authority in the same cut, not by leaving a facade.
3. Convert uncompressed texture upload to `TextureDesc`, persistent views, neutral sampler/bind
   groups, and `RenderDevice::write_texture(...)`; retain only the upload payload until its
   submission ticket is terminal.
4. Convert runtime mipgen from its immediate native encoder/`queue.submit` path into the compiled
   frame's neutral compute command list using its linear storage and optional sRGB sampled views.
5. Extend neutral format/copy contracts for compressed payloads and physical mip-range rebuilds,
   then delete the raw `GpuTextureResource` path rather than keeping a compatibility representation.

The M9 format and M10 aspect contracts are prerequisites for steps 3--4, but they do not make any
of those product steps complete. No local timing, energy, or algorithm claim follows from this
static ownership review. Once one complete product frame uses the hard-cut route, collect RenderDoc
and timestamp evidence before optimizing upload batching, mipgen dispatch grouping, or cache policy.

### Reference-engine decisions

- Unreal `D3D12Submission.cpp` finalizes command payloads before placing them on one submission
  pipeline, batches queue work, and associates synchronization with that pipeline. The relevant
  Zircon adaptation is an immutable submission-qualified diagnostic packet, not copying Unreal's
  D3D12 thread implementation.
- Bevy `gpu_readback.rs` encodes buffer/texture copies into the render command encoder, then maps
  those buffers after submission. Its shape supports M5's one submission owner and rejects a
  per-request queue submission design.
- Unity Core `ProfilingScope.cs` brackets a command buffer and allocates recorder state lazily.
  Its useful constraint is that diagnostics must not create resource work when disabled; it is
  not a substitute for WGPU ticket or device-generation lifecycle handling.
- `dev/LumenInUE5.5.4WithComputeShader/App.cpp` declares the Lumen-style resources as named
  persistent, temporal, and discardable groups before pass execution. Its `D3D12Context.cpp`
  blocks every command-list submit with an infinite fence wait, so it is a useful pass/resource
  decomposition reference but explicitly not a submission/readback latency model for Zircon.

## Required Pipeline Shape

```text
compiled graph pass slots
    -> DiagnosticQueryPlan (dense pass/query ranges)
    -> immutable RhiSubmissionPacket
    -> WgpuRenderDevice submission owner
    -> one poll/completion step per device generation
    -> bounded ticket-ordered diagnostic delivery
```

`zr_rhi` owns neutral identifiers, quotas, terminal receipts, and future query-scope DTOs.
`zr_rhi_wgpu::production::diagnostics` owns WGPU query-set allocation, resolve/copy encoding,
and decoding. The graph compiler owns labels and maps dense `PassDiagnosticId` values back to
debug names only when exporting a completed frame. The device remains the only owner that may
submit or poll. No core-framework API may expose WGPU query objects or staging buffers.

## Implementation Order

1. Preserve the M5 readback lifecycle and M6 packet invariants, then obtain managed validation for
   their focused source and behavior tests. Do not merge the legacy `GpuReadbackQueue`, timer, or
   statistics owners into either by adapter or callback shim.
2. Source implementation completed: query scope and query resolve are now attached to the neutral
   command/submission packet after render/compute pass placement was explicit. Focused test
   definitions cover zero/single/exact-budget plans, truncated resolves, scope reuse, optional
   feature fallback, and source ownership. Managed validation still must add exercised
   cancellation, device loss, map ordering, and enabled-query execution receipts.
3. Use `DiagnosticQueryPlan`'s dense IDs during graph compilation. Query decoding must write
   arrays indexed by pass slot; labels and duplicate-name aggregation remain at export time.
4. Hard-cut the legacy timer/statistics/readback owners only when all product call sites use the
   packet path and the direct poll/indefinite-wait source counts reach zero outside the designated
   backend completion owner.
5. Only after product observability identifies a dominant cost may an algorithmic optimization
   begin. There is no authorization in the current evidence to tune Lumen-style screen-probe,
   radiance-cache, culling, or parallel-recording algorithms.

## Product Migration Boundary

The current product route does not construct `production::WgpuRenderDevice`. Instead,
`graphics/backend/render_backend/render_backend_new_offscreen.rs` creates a profile and fault
supervisor beside raw `wgpu::Device` and `wgpu::Queue` fields. `SceneRenderer` now passes its
startup-derived request policy into that construction so optional legacy diagnostics can be
negotiated before the same raw backend is created, while
`graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs`
receives those fields directly, calls `queue.submit`, starts legacy readback mapping, and polls
an IBL writeback path. The production neutral device therefore cannot be wrapped around that
route as an additional owner: two submit/poll/lifecycle authorities would be worse than the
current explicit raw path.

The required hard-cut order is:

1. The neutral, device-generation-qualified submission packet now exists at source level and can
   retain all command lists for one logical frame ticket. It also carries a frame-qualified
   `DiagnosticQueryPlan` whose scopes can only be consumed once by diagnostic render/compute
   passes; WGPU resolves native query data in the same logical packet.
2. Source infrastructure now includes ticket-qualified CPU color-texture uploads without adding a
   second native queue owner. Convert a deliberately narrow offscreen clear/copy/triangle product frame from raw resources
   and `wgpu::CommandBuffer` to neutral descriptors, handles, and the existing
   `WgpuRenderDevice` submission service. The product backend must transfer ownership rather
   than construct a second device wrapper.
3. Move graph materialization in dependency order: transient resources, graphics/compute passes,
   upload and readback packets, then UI/present. Delete each raw owner after its consumers move;
   do not retain a raw-device compatibility accessor.
4. Start optional timestamp/statistics query scopes only through the packet path. Query scopes are
   observability, not an MVP frame prerequisite: unavailable features or bounded-admission
   rejection leave normal rendering eligible for submission and emit a terminal diagnostic
   delivery instead.

This is a source-architecture finding, not a product submission migration claim. The startup
feature-negotiation call site changed in this work item, but the current raw owners remain outside
accepted M5 scope.

### Product texture-upload migration and measurement plan, 2026-08-25

Status: `implementation_started_source_validation_pending`. This is a design and measurement
decision for the next M3 product-owner slice, not a performance result or an accepted milestone.
The pre-cut audit found three independent `Queue::write_texture` paths and two independent
`Queue::submit` paths in source RGBA/lightmap upload, compressed upload, runtime mip generation,
and physical mip-tail replacement. The current source implementation moves those asset-texture
paths behind the batch boundary; treating them as separate helpers would have preserved the
queue-ownership and submission-fanout defect this plan is intended to remove.

The chosen boundary is a WGPU-production `WgpuTextureUploadBatch` consumed by the existing
generation-qualified `WgpuSubmissionService`, not a second product queue wrapper and not a graph
compiler feature. A batch has one ticket, one or more subresource updates, and a shared
`Arc<[u8]>` source payload with checked per-subresource ranges. Mip/layer updates therefore retain
the source bytes once through queue acceptance instead of copying the complete asset per mip. The
submission service remains the only owner that may call native `Queue::write_texture` or
`Queue::submit`; product resource code can only enqueue a batch through the backend coordinator.

Resource-streaming order is explicit. Ordinary asset uploads enqueue in FIFO order before the
frame's graphics command packet, and the submission service flushes them on the one native queue
timeline. Runtime-generated mips are post-upload command buffers appended to that same frame
packet. Physical mip-tail replacement first groups all common-mip copies into a bounded copy
packet, then enqueues only missing source mips; it must never copy a full CPU asset only to retain
already-resident GPU mips. The copy packet, upload ticket, and frame packet are all generation
qualified. Resource streaming publishes after coordinator admission because FIFO order guarantees
the writes precede the same frame's graphics commands; the submission service itself retains
accepted packets until native queue acceptance and terminal status remains available for resource
recycling and device-fault observation.

The source implementation must expose static regression guards for one ticket per texture batch,
shared payload ranges, coordinator-only queue access, and zero direct product `queue.submit` in
the texture owner. The managed measurement phase then records per-frame queued batch count,
subresource count, payload bytes, native submit count, submission completion latency, and texture
streaming budget deferrals. Only the fixed-scene protocol below may determine whether an upload
algorithm change improves CPU time, GPU time, memory, or power; no estimate in this section is a
performance value.

### Product buffer-upload migration audit and plan, 2026-08-25

Status: `implementation_complete_static_validation_passed_dynamic_validation_pending`. This is a source census and a
dependency-ordered implementation decision, not an accepted milestone or a performance result.
The file-level census across `graphics/` excludes standalone test files but may retain inline
source-test strings: it finds 54 `write_buffer`, 24 `write_texture`, and 42 `submit` tokens. The
candidate buffer writes divide into six under `GpuScene`, 47 under `SceneRenderer`, and one other
graphics owner. Texture and submit counts require separate producer classification; they must not
be bulk-rewritten as a single category.

`GpuScene` is the first buffer slice because its production ingress is centralized in
`build_mesh_draws/.../gpu_scene_sync.rs`. Its small path writes sparse primitive, instance, and
light ranges plus two count uniforms. Its large path first writes one three-slot staging-ring
buffer, then records copy commands into the already-owned frame encoder. Both paths require the
write to be ordered before those copy/draw commands, but neither requires an additional native
submit.

The chosen boundary is `WgpuBufferUploadBatch`: one Copy ticket contains buffer, offset, checked
source range, and immutable `Arc<[u8]>` payload ownership. `GpuScene` packs its sparse ranges
and count uniforms into one immutable blob at flush time rather than retain borrowed CPU shadows
or allocate one payload per range. For the staging path, the same batch writes the selected ring
buffer before the frame encoder's existing buffer-copy commands. `WgpuSubmissionService` remains
the only native `Queue::write_buffer` owner, and `RenderBackend` exposes only an enqueue method;
the regular frame submit remains the flush authority.

The initial migration is deliberately limited to the six core `GpuScene` writes in
`direct_upload.rs`, `staged_upload.rs`, and `staging_ring.rs`. Per-pass uniforms, offline IBL/bake
writes, fallback resources, UI uploads, readback/capture, present, and the independently-owned
morph/virtual-geometry uploads each have different creation and completion lifetimes, so they
require their own producer audit after this batch contract is exercised. Static guards must prove
that the three migrated core modules contain no raw `queue.write_buffer`, and that staging writes
are enqueued before the encoder-dependent copy packet. The implementation preserves dirty ranges,
force-full flags, pending frees, and scene-count publication until the coordinator admits the
batch; a rejected admission therefore leaves the CPU shadow state retryable. Managed measurement
then compares a fixed 1080p scene using batch count, range count, payload bytes, native submit
count, CPU encode/submit time, completion latency, peak staging bytes, and power. No baseline
exists yet, so no performance or energy value is asserted here.

### Submission measurement infrastructure status, 2026-08-25

Status: `implementation_complete_static_validation_passed_managed_runtime_validation_pending`.
This is an observability prerequisite for the fixed-scene measurement gate, not a product-frame
result, a `SceneRenderer` collector migration, or an accepted performance milestone.

Completed source items:

1. `WgpuSubmissionService` records device-generation-local monotonic facts for admitted buffer
   and texture batches, physical writes, payload bytes, native queue submissions, submitted and
   completed tickets, total and maximum completion latency, upload-admission rejections, and the
   lifetime pending-upload high-water mark. Counters are never reset by a reader, so independent
   diagnostics can derive their own intervals without consuming one another's samples. The public
   delta DTO requires an identical `DeviceId` plus `DeviceGeneration` and rejects any counter
   regression, so a device-owner replacement establishes a new baseline instead of yielding a
   false measurement interval.
2. Each snapshot additionally reports current retained upload bytes. It includes both accepted
   queued payloads and payloads currently being flushed, so it cannot report zero while the native
   submission owner is still retaining upload memory.
3. `SceneRenderer` forwards this snapshot without owning WGPU queue work, and
   `WgpuRenderFramework::try_submission_metrics_snapshot` exposes it to product measurement code.
   Its behavior is hard-cut into
   `graphics/runtime/render_framework/wgpu_render_framework/submission_metrics.rs`; the framework
   root retains type, state, and scheduler ownership rather than accumulating profiling behavior.
   The framework accessor uses `try_lock`; it returns no sample while a frame owns renderer state
   and deliberately performs no `finish_submission`, operation lock, native queue write, or native
   queue submit. Sampling therefore does not add a submission boundary or wait on an active frame.
4. Focused source tests define the counter, current-versus-peak upload-byte, queue-owner, and
   nonblocking-sampler contracts. File-local `rustfmt --check`, `git diff --check`, retained-byte
   accounting, queue-owner, and sampler-boundary checks pass. Managed Cargo execution has not run.

Still required before any producer algorithm decision: run the fixed Windows 1080p workload,
retain the required cold/warm raw samples, export a current PNG and matching RenderDoc RDC, collect
GPU timestamp and independent power telemetry when available, and compare equal-output samples.
The `SceneRenderer` collector remains intentionally unimplemented until those measurements identify
its producer/range distribution and a dominant bounded cost.

### Pending-upload accounting audit and optimization gate, 2026-08-25

Status: `architecture_review_complete_optimization_not_authorized_without_measurement`.
This is a bounded-complexity finding and a test-design record, not a claim that pending-upload
accounting is the current product bottleneck.

The current submission service keeps command packets and upload packets in one FIFO
Vec<QueuedWgpuSubmission>. Upload admission derives both the upload-packet count and retained
payload bytes by scanning that vector; pending_upload_bytes and the metrics snapshot repeat the
byte scan. Therefore one pre-flush burst of N upload packets costs O(N) per admission and
observation, or O(N^2) across N sequential admissions. The reference 1080p policy caps pending
uploads at 16 and staging at 64 MiB, so this is not automatically material on the default profile;
the policy is configurable, however, and the algorithm must not silently become quadratic for a
larger streaming budget.

The required future owner is a small state-local accounting record:

| Transition | Pending upload count | Pending payload bytes | Required invariant |
| --- | ---: | ---: | --- |
| admit one buffer/texture batch | increment | add exact batch payload | mutate only after budget admission and FIFO push |
| cancel an accepted queued upload | decrement | subtract removed batch payload | command-packet cancellation leaves it unchanged |
| take FIFO for flush | reset to zero | move all pending bytes to flushing bytes | snapshot still reports retained bytes during native writes |
| write one flushing batch | unchanged | subtract exact batch payload from flushing bytes | ticket joins the existing ordered native-submit boundary |
| terminalize after fault | reset to zero | reset to zero | no retained payload remains after terminal status |

The FIFO vector must remain the command ordering owner. Its O(N) flush walk is necessary to issue
the writes and preserve command/upload boundaries; cancellation's ticket search also remains a
separate semantic decision and must not be hidden behind an index that changes FIFO behavior.
Only repeated aggregate accounting is a candidate for O(1) state. All mutations must stay under the
existing queue-access then submission-state lock order. The change must not alter UploadBackpressure
or MemoryBudgetExceeded receipts, the reserved-ticket retry path, completion timing, or the
generation-qualified metric snapshot.

Before implementation, run the fixed Windows workload at pending depths 1, 8, and 16 plus an
explicit higher configured budget. Capture CPU exclusive time and lock hold time for upload
admission and metrics sampling with WPR/ETW, then use the current submission snapshots to record
batch count, retained bytes, native submissions, completion latency, and rejection count. Proceed
only if that phase is material in the equal-output p95 frame-submit profile. The implementation
must then add focused transition tests for admission rejection, queued cancellation, flush transfer,
fault terminalization, and snapshot parity before requesting managed validation, PNG/RDC, or any
before/after performance claim.

### SceneRenderer frame-upload collector audit and design, 2026-08-25

Status: `architecture_designed_implementation_not_started`. This section is a source-level
classification and an implementation gate, not a performance claim. A current production-file
census under `graphics/scene/scene_renderer/`, excluding standalone `tests/` paths, finds 46
`queue.write_buffer` tokens across 34 files. The equivalent `queue.submit` search finds 13 tokens
across 11 files and still includes inline source-test material, so submission ownership must be
classified by runtime call path rather than this token total.

The buffer sites are not one producer class. Frame-global scene state begins in
`core/.../write_scene_uniform.rs`; mesh indirect work uses dirty-range comparison in
`mesh/mesh_pass/indirect_buffer_upload.rs`; shadow, probe, history, and light-grid resources have
their own persistent lifetime; post-process and HZB values are pass-local; UI data has a separate
retained-resource lifetime. The indirect writer is especially important: one source call can emit
an unbounded number of disjoint dirty ranges, so replacing the function mechanically with one
ticket per range would preserve the principal scheduling defect while adding ticket pressure.

The next owner must be a `SceneFrameUploadCollector` with a strict frame lifetime: begin before
frame-global construction, accept producer-owned immutable buffer-write plans while passes are
prepared, admit one `WgpuBufferUploadBatch` immediately before the existing frame command packet,
then complete producer state only after successful coordinator admission. It is owned by the
direct/compiled scene-frame orchestration boundary, not by `RenderBackend` globally and not by an
individual post-process pass. The collector may aggregate ticket ownership and source-payload
lifetime, but it cannot claim to merge native writes targeting different WGPU buffers; a shared
staging-buffer/copy algorithm is a separate measured policy decision.

Every migrated producer needs a two-phase contract: `prepare_upload` derives ranges and immutable
payload without changing its CPU shadow or reset flags; `commit_upload_admission` advances those
state fields only after the collector gets its ticket; abandonment or device-fault termination
keeps the plan retryable. Resource recreation, bind-group rebuild, and history invalidation remain
explicit owner responsibilities. The first implementation target is the frame-global scene
uniform pair plus the indirect-workspace producer, because both are used before draw/compute
recording and expose existing byte/range diagnostics. Shadow, probes, post-process, HZB, UI,
readback, and persistent texture-copy paths remain separate audits.

Before choosing shared staging versus direct native writes, capture the fixed-scene baseline in
the measurement protocol below and compare equal output, identical adapter/driver, and the same
range distribution. The required data are per-frame producer count, planned range count, payload
bytes, collector ticket count, physical write count, native submit count, CPU planning/submit
time, GPU frame time, completion latency, peak staging bytes, and synchronized power. No current
run supplies those values.

## Measurement Protocol

1. Use a Windows product WGPU run with artifacts outside `C:`. Record adapter, driver, backend,
   feature tier, resolution, scale, scene, source fingerprint, and capture command.
2. Run deterministic 1080p cold and warm samples separately; discard 60 warm-up frames and retain
   at least 300 raw frames for each condition. Record CPU submit time, GPU timestamps, physical
   submit count, poll count, diagnostic bytes/requests, fallback flags, and memory counters.
3. Produce a current-source PNG under `docs/tests/runtime/render/` and a matching RDC captured
   using `D:\Tools\renderdoc`. The PNG, RDC, graph dump, and raw profile must share a frame
   generation and source fingerprint.
4. Compare instrumentation-disabled and instrumentation-enabled runs only when output pixels,
   pass order, command-buffer order, quality settings, and fallback state match. Otherwise the
   samples answer different questions and must not be compared.
5. Report median and p95 from raw data. Power requires an independent synchronized sensor; frame
   time, timestamps, or utilization are not power measurements.

## Completion Criteria

- The managed validation service accepts the focused source and behavior tests.
- A product capture produces the required PNG and RDC at the required locations.
- The raw profile identifies one dominant bounded cost with equal-output comparison evidence.
- Any subsequent optimization supplies before/after raw samples, a fresh RDC, and an explicit
  explanation of whether the measured bottleneck disappeared.

Until those criteria are met, this task remains `current_source_review_complete_runtime_baseline_pending`.

## Surface M6 Contract And Measurement Gate, 2026-08-24

Status: `source_implementation_complete_managed_validation_pending`. This section records the
implemented neutral surface contract and its measurement gate. It does not claim a product-frame
migration, managed Cargo validation, a capture, a PNG/RDC artifact, or a performance result.

### Current ownership finding

The current product `ViewportSurface::present_texture` acquires a WGPU surface texture, creates
its native view, records a separate fullscreen-blit encoder, submits it directly, and immediately
presents. `RenderGraphExecutionResources` can import an external native WGPU texture/view, but
its storage is likewise native. Passing the acquired view into that table would make the raw
surface a hidden bypass around the neutral resource registry and would retain the separate submit.
Neither is a valid hard cut.

Bevy stores an acquired `SurfaceTexture` and its view for one render frame, then presents in a
dedicated post-submit stage; it also records that surface acquire itself can expose GPU back
pressure. Unreal's RDG registers external textures, executes all graph work, and performs queued
extraction only at graph completion. Unity's registry similarly brackets each graph execution and
imports the current backbuffer as an external resource. The Zircon adaptation is therefore a
short-lived, device-owned acquired-target lease that becomes a graph external resource only by
neutral handle, never a native view export. The existing `SwapchainDesc` and `PresentMode` remain
the format/present-mode receipt vocabulary; no parallel swapchain descriptor is introduced.

### Required M6 contract

1. One `WgpuRenderDevice` generation owns every native `Surface`, acquired
   `SurfaceTexture`, and native target view. Surface creation consumes `RenderNativeSurfaceTarget`
   and returns an opaque, device-generation-qualified surface session identity. The negotiated
   `SwapchainDesc` is returned as a receipt after native capability selection; a zero extent is a
   typed non-renderable state, not silently clamped into a 1x1 product frame.
2. Acquisition returns a typed outcome: an acquired frame lease, retryable `Timeout`/`Occluded`,
   or reconfiguration-required `Outdated`/`Lost`/`Suboptimal`. A leased frame exposes only the
   neutral target `TextureHandle`, default `TextureViewHandle`, descriptor, and opaque frame
   identity. Its texture is surface-owned, has `PRESENT | RENDER_ATTACHMENT` usage, and may not be
   created or destroyed through the ordinary resource-creation API.
3. The WGPU surface registry registers the acquired target in the same production resource
   registry used by command encoding. A graph execution can bind the external target by neutral
   handle, and the final packet's render pass references that handle. No `wgpu::Texture`,
   `TextureView`, `Device`, or `Queue` crosses into graph/planner/framework code.
4. `present(frame, submission_ticket)` is accepted only for the same device generation after its
   packet has referenced that frame target and reached `Submitted` or `Completed`. It consumes the
   frame lease, retires its temporary registry bindings against that ticket, and invokes native
   present without recording or submitting another command buffer. A frame may instead be
   discarded exactly once on a pre-submit error, cancellation, device loss, or graph-cull path.
5. Surface/session/frame identities are fail-closed after resize reconfiguration, teardown, or
   device-generation replacement. Device loss terminalizes outstanding frame leases and reports a
   typed outcome; cache readiness must use the submission/present receipt rather than frame index
   or queue acceptance alone.

The initial implementation deliberately supports the existing Win32 native target, SDR
`Rgba8UnormSrgb`/`Bgra8UnormSrgb` preference, and a single serialized WGPU queue. HDR, VRR,
present-wait, and color-calibration policies remain later capability-gated work; they must extend
the negotiated receipt rather than fork the session API.

### Source implementation status

- Complete in source: `zr_rhi` now owns opaque session/frame identities, typed create/acquire/
  reconfigure/present/discard receipts, ticket-target qualification, and bounded terminal-frame
  history sized from the existing submission receipt limit. The deterministic contract defines
  regression coverage for ordinary resource-destruction rejection, an unrelated submitted ticket,
  accepted-packet cancellation during reconfigure, stale session/frame rejection, and zero-extent
  non-renderability.
- Complete in source: `WgpuRenderDevice` owns a Win32 `WgpuSurfaceService` for the same device
  generation as command encoding. It negotiates SDR sRGB format and present mode, registers each
  acquired native target/view in the ordinary production resource registry with
  `PRESENT | RENDER_ATTACHMENT`, and allows ordinary command packets to reference those neutral
  handles. The surface owner retains the `SurfaceTexture`; present and discard retire the registry
  handles and terminalize leases. Resize, teardown, and device-fault paths cancel only still-
  accepted packets that reference affected frame targets before invalidating the lease.
- Complete in source: the surface service neither creates a command encoder nor calls
  `queue.submit`; final rendering remains in `WgpuSubmissionService`, and present accepts only a
  `Submitted` or `Completed` ticket that actually referenced the acquired target.
- Complete in source: `RenderPassTextureViewDesc` can name a registered `TextureViewHandle` for
  an attachment. Shared deterministic/production validation requires that view to belong to the
  declared target, use the parent format with `All` aspect, have `D2` dimension, and resolve to
  exactly the selected mip/layer. A surface-frame final pass can therefore select its leased
  default view; the production encoder reuses that registry-owned native view instead of creating
  a second temporary view for the same attachment. Submission resource tracking retains both the
  selected view and its parent texture against the exact ticket before present can retire the
  public handles. This is an allocation-path correction, not a measured performance result.
- Source implementation slice, 2026-08-25: `WgpuMvpSurfaceTriangle` is a minimal direct-surface
  consumer of that contract. It owns only a depth target and reusable neutral triangle pipeline;
  an acquired `SurfaceFrameLease` supplies the color target and registered default view. The owner
  records one neutral graphics command list, receives the sole `RenderDevice::submit` ticket, and
  calls `present_surface_frame` with that ticket. Its public boundary accepts only
  `WgpuRenderDevice`; its crate-private contract route also verifies that the renderer, submitting
  device, and lease share a device ID/generation before command recording. It creates no offscreen
  color target, native surface view, command encoder, or queue submit. If recording, packet
  submission, or present fails through the lease-owning device, it discards the still-live lease.
  Deterministic regression definitions cover the normal ticket-qualified present and foreign-device
  rejection that must terminalize the lease. File-local formatting and source-boundary checks pass;
  managed Cargo and a real Win32 surface run remain pending.
- Source implementation and review slice, 2026-08-25: `WgpuSurfaceBootstrap` now creates and
  privately retains the native surface before adapter enumeration. It selects only policy-approved
  adapters that satisfy the same SDR format and present-mode predicate used by surface session
  negotiation, then consumes itself into `WgpuSurfaceAdapterBootstrap`. That one-shot owner keeps
  the selected adapter, instance, and surface together; it validates the neutral profile before it
  asks its stored adapter to create the native device and queue, so callers cannot inject a device
  or queue from another native instance. `WgpuSurfaceService` also declares acquired frames before
  sessions, guaranteeing a dropped `SurfaceTexture` discards before its owning `Surface` is
  released. Two independent source reviews found and closed those lifecycle/provenance failures;
  scoped formatting, whitespace, source-boundary, and documentation checks pass. This remains
  source-only: the pre-existing public device-created session route is documented as secondary
  while the product primary-surface hard cut is pending, and no Cargo, Win32, PNG, RDC, timing, or
  power evidence is claimed.
- Pending: the product `ViewportSurface` hard cut, managed Cargo execution, real Windows
  acquire/reconfigure/device-loss runs, screenshot and RenderDoc artifacts, and all timing,
  energy, or quality assertions remain outside this source-only slice.

### Dependency-ordered hard cut

1. Source implementation is complete for deterministic session/frame identity, retryable/
   reconfigure outcomes, ticket qualification, duplicate terminalization, stale-generation
   rejection, the WGPU surface service, and the direct-surface MVP conformance definition. Run
   those focused contracts through the managed validation service before treating this foundation
   as accepted.
2. Convert graph physical materialization and pass recording to neutral resource handles and
   `RhiSubmissionPacket` before binding a product viewport frame. This preserves the existing
   graph planner as WGPU-free and keeps the graph execution owner responsible for one packet.
3. In the same product slice, replace `ViewportSurface`, its fullscreen blit, and the raw
   `RenderBackend` surface methods with the neutral session/lease path; submit the final graph
   packet, then present with its ticket. Delete the raw owner and source guards must reject its
   reintroduction. The retained UI surface follows the same session rather than creating a second
   device or submitting independently.

An isolated native-surface wrapper, a native-view import into the old graph execution table, or a
new present-side queue submit is explicitly out of scope because each preserves the current dual
ownership failure.

### Measurement and decision gate

Before changing acquire timing, buffering depth, present policy, or blit/composition algorithm,
collect a managed Windows baseline at fixed 1080p after the hard cut: at least 60 warm-up frames
and 300 retained frames, with raw samples for CPU acquire, CPU encode/submit, GPU frame timestamps,
submission count, retry/reconfigure outcomes, queue completion latency, and peak surface-resource
residency. Compare only equal-output runs with identical adapter/driver, source fingerprint,
resolution, graph/command order, and policy receipt. Produce a matching PNG under
`docs/tests/runtime/render/` and `D:\Tools\renderdoc` RDC from one frame generation. Power is
reported only from a synchronized platform telemetry source. No threshold, Unreal comparison, or
algorithm optimization is authorized until this baseline identifies a dominant measured cost.
