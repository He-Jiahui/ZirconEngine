---
record_kind: research
status: planned_measurement
created_at: 2026-08-15
origin_plan: docs/plans/mvp/00-current-source-baseline-recovery.md
research_start_head: 2edd2ed7bb2ee90d215526d5d01edacc2cf58ce2
validation_state: static_review_only
scope: wgpu-render-profiler-and-hybrid-gi-follow-up
---

# M0 performance research: WGPU frame profiling and Hybrid GI scheduling

## Decision

No rendering-performance code change is authorized by this record. M0 current-source compile
recovery remains the prerequisite for an executable profile. The observations below identify
measurement candidates only; they do not establish a bottleneck, a power target, or a replacement
algorithm.

## Static evidence

### WGPU frame profile path

- `FrameProfiler::write_frame_profile` is called unconditionally by
  `submit_frame_extract/update_stats/update.rs` after each submission. It creates one
  `RenderPassProfileEntry` per recorded pass, clones `pass_name` and `executor_id`, constructs all
  subsystem entries, and publishes an `Arc<RenderFrameProfile>` even when no GPU readback has
  resolved.
- The profile keeps at most four pending snapshots. This is an intentional asynchronous-readback
  contract, not a queue to remove or a reason to wait for the GPU.
- `merge_gpu_timer_result` and `merge_gpu_pipeline_statistics_result` match each returned pass by
  repeatedly scanning profile entries with duplicate-name preservation. Their current upper bound is
  O(P*T), followed by O(K*P) subsystem aggregation, where P is profiled-pass count, T is timing or
  pipeline-statistic result count, and K is the fixed budget-key count.
- `GpuPassTimer` creates a `Vec<String>` at frame start and copies each reserved pass name into the
  deferred readback callback. It resolves timestamps through `GpuReadbackQueue`; no synchronous
  `map_async` wait is present. The default timestamp capacity is 64 passes.
- Existing `graphics/tests/render_perf_baseline.rs` proves deterministic counts, cold/warm graph
  cache behavior, pipeline-mode pixel parity, parallel-recording topology, and resource upper
  bounds. It does not measure profiler exclusive CPU time, allocations, or GPU query overhead.

### Hybrid GI follow-up, not M0 work

- `prepare_frame/collect_pending_updates.rs` filters each candidate through ancestor traversal,
  then calls descendant traversal and lineage-support scoring from a sort key.
- `probe_topology.rs` produces descendant lists with fresh traversal state. `scene_trace_support.rs`
  and `build_resolve_runtime/lineage.rs` independently repeat descendant and ancestor walks.
- `register_extract` clears the hierarchy maps and rebuilds its child index for every enabled
  extract. There is no explicit topology revision today. A persistent topology cache would
  therefore either invalidate every frame (no benefit) or require a new, authoritative identity
  boundary before it can be correct.
- Materializing every probe's descendants is not an acceptable default: a parent chain has
  O(N^2) total descendant entries. The existing cycle-bounded semantics also intentionally allow
  a cycle member to appear in a traversal, so an acyclic Euler-tour shortcut cannot silently
  replace it.
- These observations are not enough to replace the deterministic BTree-backed traversal. Any later
  cache must preserve tie ordering, duplicate suppression, cycle bounds, and the exact selected
  update set.

### Hybrid GI algorithm decision after detailed traversal review

Do not start with a cross-frame all-descendants cache. The safe first candidate, after profiling,
is a frame-local `PriorityFacts` projection constructed only for the retained pending-update
antichain. `collect_pending_updates` already rejects a pending probe whose ancestor is pending.
For each remaining candidate, build its cycle-bounded ancestor chain and ordered descendant list
once, then derive static depth/count, current resident-descendant count, and the dynamic trace /
request support from that one local view. Sort the resulting `(priority_tuple, request)` records
instead of recomputing the key inside comparison-driven sorting.

This preserves the current tuple order exactly: trace-support key descending, resident descendant
count descending, total descendant count descending, depth ascending, request generation ascending,
then probe ID ascending. It also keeps all time-varying inputs per frame: resident slots, scheduled
trace regions, requested IDs, and decayed recent-support maps must not be stored in a topology
projection. The expected work becomes one bounded traversal per retained candidate plus O(P log P)
comparison of precomputed scalar keys, rather than repeated traversal and `Vec` construction while
sorting P candidates. Actual improvement must be demonstrated with an adversarial chain, wide tree,
and cycle workload before adoption.

Only after that measured change should a persistent `ProbeTopologyProjection` be considered. Its
owner must publish an explicit monotonically increasing topology revision at the one canonical
parent-map rebuild boundary, retain the BTree ordering, and fall back to the present bounded walk
for cyclic components. The projection must remain linear-space; it may cache child adjacency and
validated component metadata, but not all ancestor/descendant pairs.

## Structure and review constraints

The current review register makes three findings directly applicable to this work:

- F3 identifies per-frame render extract and large `Vec` copies as a performance risk. The profile
  path's string and vector construction is therefore a measurement target, not authority to cache
  mutable frame data or weaken immutable profile snapshots.
- F4 and structure rule E10 require a typed `RenderFrameworkError` for a missing viewport or
  provider. No profiler, capture, or pipelined-submit change may turn that failure into a
  `unwrap`/`expect`, an empty success result, or a compatibility fallback.
- F16 requires `render_compiled_scene()` responsibilities to remain split between resource binding,
  graph-stage execution, and submit/present. `update_stats/update.rs` is already a leaf owner that
  gathers final reports and delegates frame-profile construction; a future measurement hook belongs
  there or in `FrameProfiler`, not in a root `mod.rs` or a re-flattened renderer method.

The RHI hard-cut remains mandatory: `zr_rhi` owns neutral contracts only, while `zr_rhi_wgpu`
owns WGPU timestamp and readback mechanics. The observed `GpuReadbackQueue` boundary satisfies the
intended one-way dependency and must remain nonblocking. Any post-M0 profiler change must retain
the existing named leaf modules and file-size/owner limits instead of introducing an umbrella
diagnostics implementation.

## Visual-evidence disposition

`docs/tests/runtime/render/plan17_pfm1_render_graph_cold_warm_wgpu_20260729.png` is rejected as
current acceptance evidence. Visual inspection shows a predominantly pink/cyan noise field rather
than an inspectable scene result, and it predates the current source. Its later 2026-08-01 export
log records a coordinator `session.register` post-response timeout before the managed test could
produce a terminal receipt; that log does not prove a current Cargo run or a generated image.

The later proof must use an ignored product exporter that first checks frame content and then writes
its PNG under `docs/tests/runtime/render/`. For example,
`render_product_camera_targets/visual_export.rs` verifies independent red, green, and blue camera
regions in both the sampled target and PrimarySurface before emitting its image. The exact MVP
exporter may use a different representative scene, but it must retain that ordering: terminal
managed receipt, pixel/content assertions, newly written PNG, and source-manifest identity.

`D:\Tools\renderdoc` contains `renderdoccmd.exe`, `qrenderdoc.exe`, and `renderdoc.dll`. The
current WGPU adapter only sets a capture-file template after `renderdoc.dll` is already injected,
which is the correct test-only boundary. The final visual bundle must therefore include a fresh PNG
and, when capture injection succeeds, the matching `.rdc` under `docs/tests/runtime/render/`; an
older capture or textual log alone is insufficient.

## Reference-engine comparison

The local Lumen compute reference uses persistent, named radiance-cache resources and explicit
last/current-frame indirection textures in `AllocateUsedProbes.cpp` and
`UpdateCacheForUsedProbes.cpp`; allocation and update are dispatched as separate compute passes.
The appropriate lesson for Zircon is resource lifetime and frame association, not a line-for-line
D3D12 port. Unreal's `GPUProfiler.h` likewise associates profiling with a frame boundary and a
deliberate capture switch. Zircon already has the corresponding nonblocking readback boundary;
any change must retain that property.

## Measurement plan after M0.3 is green

All captures and reports belong under `docs/tests/runtime/render/` or the non-C workspace target.
Use the same Windows machine, GPU driver, resolution, quality settings, and source fingerprint for
each before/after pair.

1. Establish three deterministic WGPU workloads: empty warm frame, representative MVP scene, and
   a controlled render-graph case at 1, 16, and 64 profiled passes. Record 300 warm frames after a
   60-frame warm-up in synchronous and pipelined submission modes.
2. Use Windows Performance Recorder/Analyzer for CPU sampled and allocation evidence. Attribute
   inclusive and exclusive time to `FrameProfiler::write_frame_profile`, GPU-result merging,
   graph execution, and render submission. Preserve the ETL outside C:.
3. Capture the representative frame with `D:\Tools\renderdoc\renderdoccmd.exe` / qrenderdoc.
   Record per-pass GPU duration, query resolve/copy placement, transient allocation, barriers, and
   frame latency. Save the capture and a rendered visual result under `docs/tests/runtime/render/`.
4. Report median, p95, and maximum CPU submit time; profile-path exclusive CPU time; allocation
   count and bytes; GPU frame/pass time; present interval; readback status distribution; and power
   telemetry only when the platform provides calibrated GPU board-power data. Do not compare power
   against another engine unless its workload and hardware are matched.
5. Check existing correctness invariants before any optimization: byte-identical captured pixels,
   stable pass order including duplicate names, unchanged per-pass name association, unchanged
   frame-generation/readback latency behavior, and unchanged budget/degrade decisions.

## Decision gates

- If the profiler's exclusive p95 CPU time is not material relative to the measured frame CPU time,
  keep the current design and publish the data; no speculative optimization follows.
- If it is material, first assess a capture-gated immutable-detail path while retaining always-on
  scalar statistics. This must not make diagnostics stale or change `RenderFrameProfile` consumers.
- If result merging is material, evaluate stable compiled-pass identities so matching is linear in
  returned results while preserving the current duplicate-name ordering in exported diagnostics.
- Hybrid GI may be considered only after M0 and only with a separate owner/scope. Its first
  candidate is a frame-local topology/score cache built once per immutable topology generation,
  validated against the present traversal on adversarial tree and cycle cases.

## M0 validation-admission performance finding

The first current-source validator dry-run did not launch Cargo. Its durable
`session.register` request was accepted at `2026-08-15T01:33:51.224186Z` and completed at
`2026-08-15T01:34:31.813546Z` (40.589 seconds), while `tools/zircon-session.ps1` applies a
15-second command deadline and a one-second reconciliation interval. The resulting
`command_post_timeout` is therefore a control-plane latency failure, not a Rust or WGPU
diagnostic and not evidence of a Cargo lane leak.

Static inspection identifies the structural cause. `validate-matrix.ps1` registers its
`validate-matrix:*` session without a `plan_path`, but
`CoordinatorApplication._execute_session_registration_request` unconditionally calls
`FailureGraphService.prepare_import_snapshot` before admission. On this checkout that scan
reads 1,931 plan Markdown files (41,043,063 bytes, including 379 failure artifacts), then
`import_prepared_snapshot` re-hashes the manifest and replaces the whole failure graph inside
the session-registration transaction. A planless validation session has no failure-routing
semantics to compute, so this cost is unnecessary on every validation invocation.

The coordinator owner should make the smallest semantic-preserving change: determine whether
the requested registration has an effective `plan_path` (the argument or the immutable existing
session value) before scheduling `prepare_import_snapshot`. Pass no `before_admission` callback
for a planless session; retain the existing immutable snapshot, graph replacement, and
`resolving_failure` behavior for every planned session. This changes the planless admission path
from O(number of plan artifacts) to O(1) coordinator work without weakening failure routing.

Required focused regressions for that owner are: a first planless registration never invokes
`prepare_import_snapshot`; re-registering an existing planless session also bypasses it; a
planned registration still imports a fresh graph and returns its open handoffs; and durable
request replay remains terminal/idempotent in both paths. After those tests pass, rerun the
validator dry-run with the normal 15-second wrapper deadline before any managed Cargo request.
This report does not claim that implementation or revalidation has occurred.

## Current outcome

Static review completed. No CPU, GPU, power, or image-quality measurement exists for the current
source fingerprint because M0.3 has not reached a managed executable compile. No performance claim
or optimization result is recorded.

## Render01-04 current-source architecture audit

This audit was made against the current source tree, rather than assuming that the numbered plan
text still names the live module paths. It identifies the first three render-path hypotheses to
measure after M0; it does not authorize a change before those measurements.

### Render01: render graph and compiled pipeline cache

`render_graph/builder/compile.rs` already has the required graph foundations: validation,
topological ordering, a packed `ManualPassReachability` representation, root-driven pass culling,
and compiled resource lifetimes. The render-graph change should therefore not be a replacement of
the scheduler or its lifetime model.

The first candidate lies at the compiled-pipeline boundary. `CompiledGraphCache` has a bounded
HashMap (default capacity 16), but `get_or_compile_with_status` invokes the miss callback while it
holds `&mut self`. The current submission path obtains this cache through framework state, so a
cold compile can extend the duration of the framework-state critical section. Separately,
`scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs`
filters stage records by name and then linearly searches graph passes by that name for every
selected stage. It also owns and clones diagnostic strings on the execution path.

Do not optimize either point in isolation. The correct follow-up is one coherent compiled-artifact
contract: resolve final variant inputs before a single cache lookup, reserve a miss under the
state lock, compile outside that lock, publish with a generation-aware compare-and-publish rule,
and store dense pass identities or stage ranges in the compiled artifact. Execution can then use
the direct graph-pass reference and preserve the current ordering, culling, duplicate-name
diagnostics, and parallel-recording eligibility. A cache change without that identity contract
would only move allocation and lookup work to another layer.

### Render02-03: mesh preparation and GPUScene upload

`scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs` currently iterates every
`PendingMeshDraw`, creates a live-key `HashSet` and entry `HashMap`, registers the instance,
stages skinning/morph data, writes primitive and instance records, then calls
`retain_registered_keys`. This is correct for a fully dynamic scene, but zero bytes written to the
GPU does not demonstrate zero CPU work for a stable scene. Static cached draw commands therefore
cannot be the main optimization boundary while the producer still materializes and synchronizes a
full per-frame pending-draw representation.

The follow-up design target is a versioned static-scene projection, owned at the mesh/GPUScene
boundary, plus a compact per-frame dynamic delta stream. The static projection may contain stable
instance IDs, immutable resource/material/LOD information and GPUScene allocation ownership; the
frame stream may contain only visibility, transforms, skin/morph changes, and invalidation events.
It must not use a full-scene live-key sweep as a substitute for explicit removals. The existing
direct versus staging upload policy is retained until profiling establishes that transfer behavior,
rather than producer-side traversal, dominates.

### Render04: multi-view visibility

`graphics/visibility/view_context/build_views.rs` builds one shared frustum-candidate array, which
avoids rebuilding bounds for extra views. Each custom camera, directional cascade, point-light
face, and spot-light view nevertheless calls `mesh_frustum_visibility` over that complete array;
layer or shadow-caster relevance filtering follows in a second full traversal. Point shadows
multiply the view count by six. This is a structural scaling hypothesis proportional to the number
of candidate primitives times the number of extra views, not evidence that HZB itself is at fault.

The candidate architecture is a static visibility projection keyed by the existing static-index
generation. It should query a broad-phase cell/range set per view, apply layer or shadow relevance
before exact frustum work, and emit stable compact primitive ranges or bitsets that both mesh and
GPUScene consumers can borrow. HZB remains an asynchronous later-stage decision and is explicitly
out of scope for this first CPU broad-phase change. The projection must retain current custom
camera, cascade, point-face, and spot-light semantics.

### Ordered measurement matrix

| Order | Hypothesis | Required counters | Do not change until |
| --- | --- | --- | --- |
| 1 | Cold graph compile extends framework-state contention | cache hit/miss, miss duration, lock wait/hold time, pass lookup count, allocations | 300-frame CPU sampling shows material p95 contribution |
| 2 | Stable meshes still incur full CPU preparation/GPUScene traversal | pending draws, static/dynamic count, map/set allocation bytes, sync CPU time, dirty upload bytes/ranges | a static-scene workload separates producer CPU time from upload time |
| 3 | Extra views scale culling with the full candidate set | candidates tested per view, relevance rejects before/after frustum, CPU time by view type, visible count | light/camera sweep confirms the expected scaling and pixel/draw parity |

For every candidate, the before/after fixture must use identical scene content, resolution,
adapter/driver, feature tier, warm-up count, and source fingerprint. Record median, p95, and max
for CPU phase time and GPU frame time; retain captured-pixel hashes, pass order, visible sets, and
draw counts as correctness gates. A board-power comparison is valid only when a calibrated
telemetry source and matched workload are available. Until then, neither power parity nor
algorithmic optimality is claimed.

## Render05 current-source lighting and shadow audit

### Existing ownership and working baseline

The MVP light and shadow foundation is already present under
`graphics/scene/scene_renderer/{lighting,shadow}`. `core/framework/render/light` supplies neutral
light and shadow DTOs; the scene renderer owns GPU packing, per-view grid construction, atlas
allocation, cache identity, and WGPU upload. This is the correct existing boundary. No new root
crate, editor dependency, plugin, or framework-wide renderer abstraction is justified by the
observed work.

The light grid is a CPU-produced zbin/tile-mask product. `build_light_grid` allocates zbin and
tile-mask vectors sized by the active view and light count, projects each light to an affected tile
rectangle and zbin range, then writes a bit into those products. It is bounded by the configured
word budgets. `light_grid_stats` subsequently walks every zbin x tile x mask-word intersection to
derive occupancy and average/peak counts. `build_light_grid_for_frame` independently packs the
extract lighting for this grid, so the profile must determine whether this conversion duplicates
work already paid by the GPUScene light-buffer path before any shared representation is designed.

The shadow path has the right correctness direction already. `ShadowCache` keys reusable static
depth with the light parameters, static-caster revision, and atlas-slot generation, and fails
closed whenever any input changes. The CPU cost that remains hidden behind a cache hit is
`static_shadow_caster_revision_from_meshes_with_resource_revisions`: it walks eligible meshes,
builds a temporary list, sorts by stable instance key, and hashes the result each eligible frame.
Likewise, `FreeRectPacker` re-sorts free rectangles for every allocation and `compact_free_rects`
performs pairwise containment elimination after reservation. These are valid correctness-preserving
algorithms, but their cost must be measured together with free-rectangle fragmentation and
allocator churn; neither is enough evidence to replace the allocator.

### Reference-engine result and target shape

Unreal's `LightGridInjection.cpp` treats cell size, depth slices, work distribution, optional
two-level culling, and async-compute routing as separately tunable decisions. It does not imply
that Zircon should immediately add an async compute pass. Its relevant lesson is to expose a
stable workload report before changing producer placement. Bevy's clustered assignment uses
previous-frame capacity feedback, but also records that an apparent cache can be slower than
recomputing an AABB. That reinforces the profiling gate for the current CPU producer.

Unity URP's `AdditionalLightsShadowAtlasLayout` keeps request buffers and free-area scratch
storage reusable, sorts request priority before layout, explicitly rejects slices below a visual
minimum, and publishes a reverse lookup. Zircon already has deterministic request ordering,
priority, tier downgrade, slot generation, and rejection reasons. The appropriate future change,
if allocation measurements justify it, is renderer-instance-owned reusable workspace with the same
deterministic allocation semantics. Do not copy Unity static global scratch state: multiple
Zircon renderer instances and parallel paths require owner-local lifetime instead.

The first structural light-grid candidate is an immutable, renderer-internal `FrameLightSet` made
once from the render extract and consumed by both the GPUScene upload preparation and each
view-local `LightGridProduct`. It belongs below `scene_renderer`, not in the neutral framework,
because cookie packing and view grid policy are concrete renderer behavior. The second candidate
is an explicit static-caster generation published at the authoritative scene/resource mutation
boundary, consumed by `shadow` to avoid rebuilding a whole-scene revision when no eligible caster
changed. It must retain the current fail-closed result for missing resource revisions. Neither
candidate may be implemented as an editor-owned cache or an unbounded global map.

### Render05 measurement gates

1. Separate `build_light_grid` time, `light_grid_stats` time, lighting-packer time, allocation
   count/bytes, grid upload bytes, and final Forward+/Deferred GPU lighting time for 16, 64, 128,
   and dense-local-light workloads at fixed resolution. Record actual tile size, zbin count,
   words-per-tile, average/peak lights per cluster, and CPU/GPU queue overlap.
2. Exercise stable static casters, one moving caster, one material/mesh revision change, changing
   light transforms, and atlas oversubscription. Record static-cache hits and invalidation reason,
   static-revision CPU time, slots reused/reallocated/rejected, free-rectangle count before and
   after packing, allocator CPU time, depth pass count, and shadow pass GPU time.
3. Retain the current visual contracts: CSM translation stability, point/spot atlas coexistence,
   shadow receiver darkening, PCF quality distinction, Forward+/Deferred parity, and exact
   accepted/rejected slot sets. Fresh WGPU PNG and RenderDoc evidence is still required under
   `docs/tests/runtime/render/` after M0 recovery.

Only if grid production, not shader lighting, is material in sampled CPU time may the owner compare
three alternatives: reuse renderer-local scratch products, reuse a shared immutable light set, or
move the already-declared light-grid IO to a compute producer. Only if static-revision work or
atlas packing is material may the owner introduce mutation generations or reusable packer
workspace. Each alternative must preserve `LightGridParams` and `GpuShadowSlot` ABI, graph
resource declarations, deterministic allocation order, typed errors, and the existing leaf-module
structure. No CPU, GPU, power, or quality improvement is claimed by this audit.

## Render06 current-source temporal pipeline audit

### Existing temporal contract

The current temporal path already has the required MVP contract. `TemporalHistoryStore` owns a
fixed read/write pair of RGBA16F textures and carries explicit validity plus read index; history
handle release removes the matching `SceneFrameHistoryTextures` entry. History preparation rejects
a new handle, size mismatch, HZB shape mismatch, TAA key mismatch, or volumetric-quality mismatch,
then invalidates TAA and related histories instead of sampling stale content. This is a concrete
resource-lifetime boundary, not a generic texture cache to flatten.

`execute_taa_resolve` writes final scene color and the next history in one multi-render-target
draw. The shader combines closest-depth velocity selection, 3x3 YCoCg variance clipping, depth and
motion rejection, reactive-mask suppression, and a bounded history-confidence channel. The scene
renderer keeps velocity, TAA, history copy, and runtime camera-history updates in separate leaf
modules, consistent with the structure convention. The renderer-owned camera history stores an
unjittered camera only after a successful frame path, which prevents the next frame from treating
the sampling jitter itself as scene motion.

### Measured-risk candidates, not proposed changes

The TAA bind-group cache is bounded to eight entries and keys all five sampled textures. It clears
when the frame target or read/write history pair changes. It only enables the cache when the
reactive-mask identity is the black fallback; a material or particle reactive-mask path creates an
uncached bind group even if the underlying WGPU texture identity later proves stable. This may be
an allocation/CPU candidate, but it is not a correctness defect and must be measured against the
total resolve cost first.

The other significant cost is inherent pixel work: the fragment resolve uses two 3x3 scans (depth
selection and neighborhood statistics) before history sampling. Replacing that with compute,
changing the quality constants, or adding a history cache would be speculative. The correct first
study separates GPU resolve time from CPU bind-group creation and parameters upload; it also checks
whether the current history invalidation matrix covers camera cut, camera/projection switch,
viewport rectangle, output size, internal render size, feature/quality toggle, and resource
recreation. The test asks for coverage, not an assumption that any case is presently wrong.

### Lumen and Unreal comparison

The local Lumen-style reference uses explicit read and write histories for radiance, metadata, and
depth in temporal reprojection, with the output pair bound as a separate stage. Zircon should take
the resource-family and frame-association lesson, not its static-global D3D12 object lifetime.
Unreal's `TemporalAA.cpp` likewise treats history validity, camera cuts, viewport transforms,
velocity availability, and output history as one pass contract. Zircon already matches the crucial
MVP parts: explicit history validity, prior camera state, velocity input, and a dedicated output.

The safe future shape remains renderer-internal: a `TemporalHistoryDescriptor` may describe the
complete compatibility identity of an individual history family, while each concrete family keeps
its own valid/flip policy. Such a descriptor belongs under `scene_renderer/history`, not in
`core/framework/render`, unless another independent renderer backend demonstrably needs to create
or consume it. This keeps TAA, HZB, exposure, volumetric, SSR, and GI from becoming a single
over-coupled state machine.

### Render06 measurement gates

1. Capture 300 warm frames for a static scene, a moving camera, a camera cut, resolution/internal
   render-size changes, viewport-rect changes, and reactive transparent/particle content. Record
   TAA resolve CPU/GPU median, p95, max, bind groups created/reused, history recreate/invalidate
   counts and reasons, history memory, velocity coverage, reactive-mask coverage, and disocclusion
   rejection rate.
2. Use RenderDoc to verify the named velocity, TAA resolve, and history operations; retain a
   current PNG and capture with the same source fingerprint. Pixel checks must cover static
   stability, camera-cut reset, dynamic occlusion convergence, reactive-material suppression, and
   particle motion, not only a nonempty framebuffer.
3. A reactive bind-group cache extension is admissible only when sampled CPU cost is material and
   the cache key includes every sampled identity, including the reactive mask and both history
   textures. A history-descriptor change is admissible only when the invalidation matrix is fully
   represented and all camera-cut/resolution tests remain equivalent. A compute resolve comparison
   is admissible only after measured GPU headroom, resource transition, and presentation-latency
   data show it improves the complete frame.

No temporal performance result, power result, or visual-quality improvement is claimed by this
audit.

## Render07 current-source post-process and HDR audit

### Confirmed execution boundaries

The post-process stack is already the authority for optional effects. Its construction enables
Bloom only when intensity is positive, enables the color-LUT bake only when color grading,
tonemap, or a user LUT is active, and creates the histogram pass only for histogram exposure.
`descriptor_filtering.rs` then removes the Bloom and LUT executors when the corresponding stack
effect is disabled. The defensive clear in `execute_bloom` is therefore not evidence of a
disabled-effect clear pass in a compiled frame and is explicitly rejected as an optimization
candidate until a capture proves otherwise.

Terminal resources follow the intended ownership rule: `TerminalPostProcessResourceCache` is
owned by `ScenePostProcessResources`, reuses region buffers and SMAA backing textures, and bounds
its physical-region cache to 16 entries. A proposed change must not replace that renderer-local
owner with a framework cache or a process-global WGPU object map.

### Measured-risk candidates, not proposed changes

`PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale`
still builds a per-camera collection of `Vec<String>` resource names, cloned dependency vectors,
and effect settings before the graph is compiled. It is the architecture-level CPU candidate:
the value to preserve is a resolved, ordered post-process plan, rather than a collection of
independent executor caches. The existing Render07 handoff for a generation-keyed compiled
artifact is consistent with this observation.

For histogram exposure, the histogram and resolve executors both build `ExposureParams`, upload
the same resource-owned parameter buffer, and create one WGPU bind group each. The second upload
is a concrete static observation, but may be insignificant relative to histogram pixel work or
command submission. The correct experiment compares a stable automatic-exposure camera with a
manual-exposure camera and records CPU command-recording time, WGPU bind-group creations, queue
writes, and GPU times for `ExposureHistogramPass` and `ExposureResolvePass` separately.

When enabled, color-LUT bake creates a transient texture view, creates a bind group, and dispatches
the fixed 8x8x8 workgroup grid for the 32-cubed LUT on every executed graph. Its current executor
has no visible color-transform generation check. That does not authorize caching the current bind
group: it includes the current exposure buffer and user-LUT views, which can change independently.
The first comparison must establish whether bake CPU/GPU time is material and whether the dynamic
exposure input is semantically used by the bake shader. Only then may the owner split a stable
color-transform artifact from dynamic exposure consumption.

The terminal output-transfer pass correctly reuses its physical-region uniform buffer, but still
creates a bind group per recording. This is a lower-priority CPU candidate. It may only receive a
bounded renderer-local binding cache if WPR identifies the creation cost as material and the key
contains both sampled/output identities plus the physical render region and device generation.

### Required first measurement

1. For a fixed-resolution scene, capture 300 warm frames for no optional post effects, manual
   exposure, automatic exposure, static color grading/LUT, changing LUT asset, and a moving
   camera. Record post-stack construction count and allocation bytes, compiled graph cache
   hit/miss, command-recording CPU median/p95, `queue.write_buffer` calls and bytes, and bind
   groups created/reused by executor label.
2. Use RenderDoc markers to measure Bloom, exposure histogram, exposure resolve, LUT bake, uber,
   terminal AA, upscale, and output transfer individually. Capture intermediate formats and named
   graph resources, then retain PNG and RDC artifacts under `docs/tests/runtime/render/` with the
   current source fingerprint after M0 compile recovery.
3. Use Windows Performance Recorder/Analyzer for CPU samples and GPU utilization; report the same
   adapter, power mode, resolution, frame count, median/p95/max, and whether present throttling
   occurred. No cross-engine power comparison is valid without that controlled baseline.
4. Preserve visual tests for disabled-effect pass removal, manual/automatic exposure convergence,
   LUT asset replacement, SDR transfer, HDR transfer when supported, resize, dynamic-resolution,
   and terminal-AA composition. A nonempty final target is insufficient evidence.

### Admission criteria for a later owner

If the measured dominant cost is post-stack/graph construction, introduce one immutable,
renderer-internal `PostProcessFramePlan`, keyed by camera post-settings generation, history/AA
compatibility, render size, output transfer mode, feature mask, and relevant asset generations.
It owns resolved ordering and compact effect/resource identities; strings remain diagnostics-only.
If the measured dominant cost is LUT bake, introduce a separate persistent color-transform artifact
with an explicit immutable key, preserving dynamic exposure as an independent consumer unless the
shader contract demonstrates it is safe to move. If bind-group creation is material, cache only
complete identity keys in the existing renderer resource owner with a bounded eviction policy.

All variants must preserve the existing post-process graph ordering, exposure history semantics,
`PostProcessGraphResourceNames` ABI, descriptor filtering, terminal physical-coordinate behavior,
and leaf-module boundaries. No Render07 performance, power, HDR-quality, or visual improvement is
claimed by this static audit.

### Structure and review alignment

This audit addresses the existing code-review finding F3 rather than creating a parallel cache
policy: per-frame extract/camera-loop cloning is a P0 performance concern, so a later solution
must remove repeated post-plan construction at its renderer owner. The code-structure convention
also requires behavior in named leaf modules, production owners below the file-size budgets, no
glob facade, no compatibility shim, and poison-safe shared-state handling. The current terminal
cache is a useful conforming example because it is bounded and owned by
`ScenePostProcessResources`.

Accordingly, `PostProcessFramePlan`, a possible color-transform artifact, and any bind-group cache
remain implementation details below `graphics/scene/scene_renderer/post_process`; they must not
be promoted into `core/framework/render`, the editor, or a process-global cache. The present
post-process worktree has foreign modifications, so this record deliberately supplies the
architecture and measurement contract without claiming an unreviewed source merge.

## Render08 current-source material and shader-variant audit

### Confirmed variant ownership

The current `MeshPipelineVariantRegistry` holds one `MeshPipelineVariantKey` made from mesh pass
kind, `PipelineKey`, and `ShaderVariantKey`, then publishes a dense `MeshPipelineVariantId`.
The shader key keeps geometry source, pass type, shading model, feature bits, quality tier, and
platform token distinct. This follows the useful part of Unreal's material-shader map model:
material/pipeline identity, vertex-factory-equivalent geometry source, and pass shader are one
specialization decision. Post and compute shaders remain outside this material variant family.

The ready path is correctly ordered. `ensure_pipeline_for_variant_with_async_placeholder` drains
completed background work and returns from `mesh_variant_pipelines` before it assembles WGSL,
performs disk lookup, creates a shader module, or creates a render pipeline. That O(1) warm path
is an invariant for a later optimization, not a candidate to replace.

The source disk cache is also structurally sound: its BLAKE3 key includes the canonical variant
key and ordered include-content hashes; metadata records schema, template, naga, and WGPU
versions; writes use temporary-file rename; corrupt or mismatched entries become misses; project
relative roots retain physical project identity. It caches compressed WGSL source, not a WGPU
driver pipeline object, so a disk hit alone must never be reported as a ready GPU pipeline.

### Measured-risk candidates, not proposed changes

The async Base-pipeline queue currently starts after geometry-source WGSL assembly and
`mesh_pipeline_shader_source_with_cache` have been called. On a module-cache miss, that path can
perform cache lookup, source comparison, compression/write on disk miss, and validation setup on
the frame thread before the driver shader-module/render-pipeline work is queued. Async compile
therefore protects the later driver work and supplies `SkipDraw` for a first-frame miss, but does
not by itself prove that first-use CPU stutter is removed.

The registry owns `HashMap` entries, a dense key vector, registered shader variants, and a
per-frame miss report without a visible variant-retirement policy. Hot reload, material revision
growth, or repeated feature-quality changes may increase that renderer-lifetime state even when
the draw stream later becomes stable. An LRU cannot be added mechanically because
`MeshPipelineVariantId` reaches draw commands and diagnostics; eviction would need generation-safe
handles, exact invalidation, and a ready/fallback policy.

The existing disk layer reads and decompresses WGSL after a process restart, while the in-memory
module/pipeline maps are empty. Driver-pipeline cache use must be measured separately from this
source cache. No statement about cold-start time, warm-start time, CPU savings, or driver cache
effectiveness follows from this static trace.

### Required first measurement

1. Measure cold process, disk-WGSL warm process, WGPU-driver-cache warm process, and fully
   in-process warm frames for a fixed material set. Repeat with 1, 16, 256, and 1,024 distinct
   material/geometry/pass variants and with 100 material or shader revision changes. Record
   source assembly, include hashing, disk read/decompress/write, naga validation, shader-module
   creation, render-pipeline creation, queue wait, first-frame placeholder draws, and ready-frame
   hit counts separately.
2. Report resident registry entries, registered shader variants, live WGPU modules/pipelines,
   cache bytes, eviction/rebuild reason, CPU allocation bytes, and frame median/p95/max. Capture
   WPR CPU samples around first variant use and capture RenderDoc pipeline/shader state for the
   same source fingerprint. Prewarm success is valid only when a following launch has zero
   runtime compilation for the declared manifest, not merely when WGSL files exist on disk.
3. Test static material reuse, skinned/static geometry separation, all mesh pass types, quality
   change, shader hot reload, device recreation, missing/corrupt cache entry, and async worker
   saturation. Retain exact rendering parity and fresh PNG/RDC evidence under
   `docs/tests/runtime/render/` after M0 recovery.

### Admission criteria for a later owner

If first-use CPU samples show source preparation is material, move the complete source artifact
(canonical key, include DAG/content hashes, assembled WGSL, and validation result) into a bounded,
content-addressed producer that prewarm and runtime both consume. If driver pipeline creation is
material, keep the existing `Queued -> Ready/Error` and `SkipDraw` behavior while using a
device/adapter/driver/layout/source-generation-compatible cache or background lane. If registry
growth is material, add a renderer-local generation-aware residency policy only after proving that
all consumers tolerate eviction and re-resolution.

Any change must keep `ShaderVariantKey` as the canonical specialization identity, retain the
ready-map early return, preserve pass/geometry separation and typed diagnostics, avoid global
cache state, and keep WGPU object lifetime in graphics/scene-renderer owners. The current Render08
worktree is foreign-modified, so this audit supplies no source patch and claims no performance or
visual result.

## Render09-10 queue and batching admission review

### Confirmed correctness repair

`RenderQueueValue::from_authored_queue` accepts an authored `i32`. The planned contract treats
`1_000..=5_000` as absolute queues and all other non-zero values as a material-relative offset,
clamped to `[-100, 100]`. The prior implementation narrowed that offset to `i16` before it
reached the existing `i32` clamp. Consequently `i32::MAX` became `-1` and `i32::MIN` became `0`,
which violated both the clamp contract and deterministic render ordering.

The Render09-owned correction keeps the established queue helper and passes the original `i32`
through `with_material_offset_i32`. Focused regression cases cover both integer extremes; normal
absolute queue, zero/default, and bounded offset cases remain covered by the existing tests. This
is an MVP correctness fix, not a measured performance claim. Managed Cargo validation and a
fresh visual product remain pending the shared validation lane.

### RendererCommon and batching decision

The current Render10 contract places renderer-independent intent in the framework-only
`RendererCommon` value: enabled state, layer set, queue override, shadow and motion-vector modes,
material overrides, static eligibility, and LOD-group identity. This is the correct boundary:
the contract carries no WGPU objects, while graphics prepare consumes it to produce render-local
batch decisions and later reports their outcomes. Its private, sorted material-override storage
also gives a stable batch-key input instead of exposing a mutable collection to consumers.

Do not change the planned instancing/static/dynamic batching precedence, dynamic vertex threshold,
or cache/eviction policy before PF-M1 profile records are available. A policy change at this layer
would otherwise conflate three independent costs: CPU extract/prepare time, draw and state-change
count, and GPU work. It could reduce draw calls while regressing upload bandwidth, vertex work, or
pipeline residency.

### Required first batch measurements

Use fixed scenes at 1, 100, and 1,000 renderers for each of these cohorts: same mesh/material
static, same mesh/material dynamic, distinct material, distinct mesh, skinned, and queue/layer
separated. For each cohort, capture the current serial path and the candidate policy with the same
camera and resolution. Record CPU extract/prepare/queue medians and p95, draw count, instance
count, state changes, upload bytes, transient/staging peak bytes, variant misses, and frame GPU
time after warm-up. A RenderDoc capture must confirm that layer or queue separation never merges
semantically distinct draws.

Only adopt a batching change when it preserves the Render09 ordering contract and fresh rendered
PNG output, reduces a measured limiting metric, and does not move pressure to an unbounded cache
or per-frame CPU vertex copy. Store resulting PNG and RDC evidence under
`docs/tests/runtime/render/`; no such artifact exists yet because the managed WGPU compile lane is
not available.

## Pipeline reconstruction admission gate

The present priority is not to add isolated rendering effects. It is to converge one UE-shaped
rendering path that every baseline and advanced feature shares:

1. `zr_rhi` owns backend-neutral resource, command, surface, and capability contracts.
2. `render_graph` owns the logical pass/resource declaration, dependency validation, culling,
   lifetime, aliasing, and compiled topology. It is Zircon's `FRDGBuilder` equivalent and contains
   no WGPU allocation or feature-specific execution policy.
3. `graphics::pipeline` and renderer-family/feature owners consume extracted view and scene
   state, then add named graph passes and explicit resources. Mesh/GPU scene, visibility, lighting,
   temporal, UI, and Lumen-like GI are peers at this layer; none may submit a side-channel frame.
4. `graphics::scene::scene_renderer::graph_execution` materializes the compiled graph and resolves
   its declared resources for an executor. The WGPU backend records only those compiled passes and
   presents the declared external target.
5. profiling, RenderDoc markers, transient-pool accounting, load/store lint, and frame products
   all use the compiled pass name/topology as their common identity.

This follows the local UE reference: Slate selects a rounded-box shader and sends local size,
outline, and per-corner radius as batch parameters, while SlateRHIRenderer draws through an
`FRDGBuilder`; Lumen systems similarly accept `FRDGBuilder` and add resources/passes instead of
owning an independent execution loop. The corresponding Zircon UI solution is an analytic shape
instance plus graph-owned UI pass, not higher CPU tessellation or an MSAA-only workaround.

The first structural gate is therefore: a feature may not own a private WGPU command submission,
transient allocation, attachment lifetime decision, presentation target, or independent timing
identity. A candidate must first express its inputs, outputs, queue lane, attachment load/store
semantics, and culling root through `CompiledRenderGraph`; its executor can then be measured and
optimized without changing the frame's topology. The pending store-lint repair is a small but
necessary instance of this rule: an attachment `Load` is a graph dependency, whereas only an
explicit attachment `Clear` proves that a later write fully replaces prior content.

MVP exit evidence remains source-bound: one physical-extent backbuffer, one compiled graph per
viewport frame, a renderer-family selected from extracted view state, a graph-owned present pass,
and a fresh PNG plus RenderDoc capture under `docs/tests/runtime/render/`. GPU-driven visibility,
hybrid GI, dynamic resolution, and asynchronous compilation are admitted only after this path is
both correct and observable. This is an architecture conclusion from static source/reference
review, not a claim that current WGPU compilation, timing, power, or image quality has passed.

### Critical legacy-frame-path finding

The public `SceneRenderer::render_frame_to_offscreen_target` currently calls
`SceneRendererCore::render_scene` directly. The HDR capture entry also allocates its own transient
targets and invokes the same direct renderer. That legacy path owns a command encoder and records
realtime IBL, GPU-scene upload, scene content, output transfer, overlays, UI, timestamp/readback,
and `queue.submit` itself. It bypasses the `render_frame_with_pipeline -> render_compiled_scene`
chain, where graph resource materialization, pass execution records, graph coverage, transient
alias accounting, and the single compiled-scene submission are already centralized.

This is a topology defect, not a candidate micro-optimization. The required hard cut is:

1. Make every public scene-frame and HDR capture entry obtain or receive the same compiled
   pipeline/context used by `WgpuRenderFramework::submit_frame_extract`.
2. Route capture readback through the renderer-owned bounded readback queue after graph execution;
   the capture target must be an imported graph resource, not a private frame encoder target.
3. Remove `SceneRendererCore::render_scene` and its direct-pass timing identities once callers are
   migrated. Do not preserve a compatibility path that can submit scene/UI work outside the graph.
4. Add a source and product regression proving the legacy entry cannot call `render_scene`, every
   frame reports graph execution coverage, and each frame has exactly one scene command submission.

The files are modified but presently unowned in the shared worktree, so this finding is deliberately
not patched here. It must be claimed as a Render01 graph-entry migration after the current
Render01 failure returns; a local wrapper or graph-looking facade around the old direct encoder
would retain the two-pipeline architecture and is explicitly rejected.

### 2026-08-24 RenderGraph resource and execution re-audit

The current source has made the first execution-packet correction: final graph passes now carry
stable IDs into an immutable packet and stage iteration can directly address a graph-pass index.
That eliminates the prior steady-frame pass-name lookup. It does **not** make the stage list a
compiler schedule. The product renderer still partitions work through hard-coded early, scene,
post, history, and late functions, while clear, history copy, readback, and writeback remain
outside the compiled graph. A loop over contiguous equal-stage entries would therefore preserve
the split execution authority and must not be described as execution batching.

The resource boundary has the same structural constraint. `RenderResourceSchema` now carries
explicit texture and buffer contracts, including typed external physical descriptors. Its explicit
format, extent, dimension, mip, sample, usage, size, and fallback contract is already sufficient
for fixed custom resources such as SSAO. However, the ordinary product path still falls back to
`texture_desc_for` and `buffer_desc_for`: texture format, size, MSAA, mip count, and usage are
selected from a resource label, and an unknown buffer defaults to a pixel-count-derived size.
Generic feature builders and shader-binding lowering can still produce `schema: None`.
Consequently, replacing every call with one static literal schema would be incorrect: HDR choice,
view versus render extent, graph MSAA, half/quarter resolution, full mip chains, froxel depth,
and OIT capacity are compile-input policies rather than fixed constants.

The local Unreal reference separates this correctly. `FRDGBuilder::CreateTexture` receives an
`FRDGTextureDesc` separately from the debug name, registered external resources retain their
physical identity, and the builder owns prologue and epilogue passes for transitions and
extraction. The small local Lumen compute sample is useful only as a warning: it hard-codes D3D12
bindings, dispatch dimensions, and manual resource barriers inside each pass. Zircon must take its
resource lifetime and frame-association lesson, but not copy that per-pass state-management model.

Before any resource or batching performance change, the required implementation sequence is:

1. Extend the graph declaration IR with typed texture and buffer allocation policies. A buffer
   contract must express byte-size policy and usage; texture policies must preserve the current
   render/view/fixed extent distinction and add the named dynamic policies currently hidden in
   authoring code. The policy resolves against the validated frame compile input, never against a
   string heuristic during materialization.
2. Introduce a built-in `RenderResourceSchemaCatalog` that resolves canonical built-in resource
   identities before graph authoring. Built-in descriptors and plugin descriptors then carry the
   resolved typed contract. A plugin-owned transient or typed external resource without a schema
   must fail compilation. Existing external resources may remain physical imports only when their
   producer owns and validates the supplied descriptor.
3. Move every current name match, `contains`, and pixel-count fallback into the catalog migration
   until it is deleted. The final allocation functions may consume only a resolved schema and
   frame compile input. Unsupported format/usage combinations must fail device-qualified compile;
   a WGPU materializer must never silently remove declared usage.
4. Add compiler-owned prologue and epilogue artifacts for clears, history, readback, and writeback.
   Only after per-access resource state, queue lane, and transition information exists may the
   compiler lower the final topological order into immutable contiguous execution batches. Batch
   boundaries are execution domain, resource transition, and queue synchronization boundaries,
   not `RenderPassStage` equality.

The first measurement after those correctness gates is a controlled before/after graph compilation
and steady-frame run. It must report CPU p50/p95/max for compile, packet construction, recording,
and submit; allocation count and bytes; GPU pass/frame timing; queue waits; and the final PNG/RDC
identity. No present measurement proves a resource-schema or batching bottleneck, so this section
authorizes neither a performance claim nor an optimization patch. M0.3 coordinator-managed
compile recovery remains the admission prerequisite for that measurement.

#### P1-030 compiled execution batch design (2026-08-24)

The current runtime has four separate ownership regions that must become explicit packet artifacts
before hard-coded stage loops can be removed:

| Region | Current owner | Required compiled artifact | Migration constraint |
|---|---|---|---|
| Frame prologue | pool begin, external binding/materialization validation, `scene_clear.record_frame_clear` | prologue resource initialization and transition list | Clear attachment and physical external-resource leases must be declared before the first consumer batch. |
| Graph body | `execute_compiled_scene_graph_stages` invokes early, lighting, scene, post, and late stage paths | ordered `RenderGraphExecutionBatch` list keyed by queue, execution domain, transition boundary, and parallel-recording eligibility | A batch may contain direct compiled pass indices only; equal `RenderPassStage` is not a batch key. Scene-specific executor context must be supplied by an explicit domain capability, not an out-of-band loop. |
| Frame epilogue | `copy_history_textures`, timestamp resolve, HZB/viewport and generic readback copy encoding | history/readback epilogue passes with resource leases and completion dependencies | History consumers must retain their producer versions through the epilogue; readback ring admission failure must abort the frame transaction without releasing live allocations early. |
| Submission epilogue | IBL cache writeback currently appends a command buffer in `submit_compiled_scene_frame` | compiler-visible writeback packet followed by one submission receipt | The writeback command buffer must become an ordered epilogue batch before `queue.submit`, not an untracked append after graph execution. |

Implementation order is deliberately constrained: first introduce compiler IR for prologue/body/epilogue
operations and explicit execution-domain requirements; then migrate clear and history; then readback
and IBL writeback; only then delete the hard-coded early/scene/post/late order and have the renderer
iterate packet batches. Device-qualified queue waits, transition lowering, and completion tickets
remain prerequisites for multi-queue or allocation-reuse optimization. This plan prevents a cosmetic
stage-loop rewrite from retaining two scheduling authorities.

### Current status (2026-08-15)

| Item | Status | Evidence / next gate |
|---|---|---|
| UE/Lumen/Slate and Zircon responsibility audit | Complete (static) | This record's pipeline reconstruction gate; no source or performance claim. |
| Render01 transient allocation dump identity | Implemented, static review complete | Each resource row now carries its allocation bucket hash alongside the bucket-local slot; managed Cargo pending. |
| Render09 authored queue extreme-offset contract | Implemented, static checks complete | `render_queue.rs` preserves `i32` through clamp; focused regressions added; managed Cargo pending. |
| Render17 attachment store-lint dependency correction | Diagnosed, not edited | A later attachment `Load` must retain the prior store; only `Clear` proves overwrite. Render17 UI12 owns the active primary session. |
| Analytic UI rounded-box AA | Current source implemented; static audit complete | UI12-owned geometry, pipeline, and WGSL now use a fixed six-vertex analytic SDF with `fwidth` coverage, outer-minus-inner border coverage, premultiplied alpha, and original-frame-preserving clipping. Geometry/shader regressions exist; native submission linkage plus managed PNG/RDC evidence remain pending. |
| SceneRenderer direct frame path | Critical migration identified | Public offscreen and HDR capture paths still bypass the compiled graph through `SceneRendererCore::render_scene`; hard-cut migration to the compiled-scene entry is required before MVP acceptance. |
| RenderGraph resource/batching architecture | Typed texture/buffer schema foundation implemented (source-only, 2026-08-24) | Compute buffer schemas reach graph authoring and exact transient descriptors. Typed external buffer descriptors now propagate through compiled declarations/lifetimes, runtime-prepare leases, stable backing bindings, and materialization validation; physical buffers may be larger or expose additional usage, but must satisfy the compiled minimum. Legacy descriptor-less report-only imports remain compatible and fail closed when a typed contract is required. Catalog resolution, device-qualified validation, graph prologue/epilogue, true compiler batches, and managed validation remain pending. |
| WGPU compile, native product run, PNG/RDC, timing/power data | Pending | Requires coordinator-issued managed validation lane and a current-source build. |

No milestone is accepted, committed, or reported to WeCom from this record. That requires the
managed validation evidence above and coordinator integration approval.
