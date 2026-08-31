---
title: RenderScene journal to GPUScene consumer review
status: core_neutral_render_extract_handoff_cpu_residency_reprojection_resolved_delta_world_bounds_local_bounds_staging_qualifier_and_source_resync_implemented_product_residency_wgpu_deferred
owner: render-03-gpu-scene
date: 2026-08-26
related_code:
  - zircon_runtime/src/graphics/scene/render_scene
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/core/framework/render/frame_extract/scene_changes
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PrimitiveSceneInfo.cpp
  - dev/LumenInUE5.5.4WithComputeShader
doc_type: architecture-review
---

# RenderScene journal to GPUScene consumer

## Decision

Do not add a second sidecar adapter around the current per-view draw synchronization. The product
cutover must make the sealed `RenderSceneChangeJournal` the only persistent GPUScene lifetime input.
`sync_gpu_scene_pending_draws` may remain temporarily for parity, but it must not register, retain, or
remove persistent entries after cutover.

The shared CPU generation gate is implemented as `RenderSceneJournalCursor`: preflight does not
advance state, exact replay is a no-op, and stale, skipped, inverted, non-adjacent, overlapping, or
superseded transactions return typed errors. Its token is opaque outside the cursor owner and commit
rechecks the range before state advancement.

The CPU residency half of the product consumer is now implemented in
`gpu_scene/journal_consumer.rs`. It consumes the sealed journal, validates the persistent
`RenderScenePrimitiveHandle` slot/generation and resident stable key directly and projects only
touched residency slots in slot order. Its only product-facing apply route is
`apply_with_staging`: internal preflight and commit are private, the caller's typed staging callback
must succeed before cursor/residency publication, and a staging error leaves both unchanged. A
successful apply returns the staging output for later upload/retirement reporting, while exact replay
skips the callback and returns `Replayed`. The same apply plan projects slot-ordered full writes for
additions, exact dirty-domain writes for updates, and old-generation retirements for removals.
Same-journal slot reuse preserves both the old retirement and new full write. Primitive payloads are
borrowed from the immutable journal, and full/dirty/retirement plus instance-transform/local-bounds
write counts are sealed without a diagnostics scan. `LOCAL_BOUNDS` is an internal staging qualifier,
not an extra diagnostic domain: transform-only changes still invalidate the CPU world envelope but
seal zero local-bounds GPU writes because the WGPU ABI stores transforms in the instance row. It owns
no second stable-key map or primitive-index allocator. This slice does not yet mutate
`GpuScene` buffers: asset/capacity staging, dependent arena allocation, WGPU writes,
submission-completion retirement, previous-state roll, and old-path hard cut remain deferred. The
affected `gpu_scene.rs`, mesh-build, extract, and `SceneRendererCore` owners contain extensive
concurrent worktree changes. The scene-side prerequisite now has a world-owned render-dirty entity
journal plus a world-bound sparse component projector with persistent removal cursors. The
resolved-input Render03 delta conversion is now implemented behind a narrow geometry resolver, but
the unified asset-residency/pending adapter and product WGPU staging remain pending. No WGPU product
path is changed by this record and no performance or runtime acceptance is claimed.

The producer/consumer type boundary is now physically connected. The immutable sparse DTO lives in
`core/framework/render/frame_extract/scene_changes/`, contains no scene component wrapper or WGPU
type, and is attached as one shared `Arc` on `GeometryExtract` for active and inactive cameras.
`RenderSceneComponentProjector::project_frame` consumes that exact field and rejects a mismatched
`RenderFrameExtract.world` before asset resolution; its initial/full replay test now enters through a
real frame instead of directly reading World internals. Stable repeated extracts preserve pointer
identity. This is the cross-layer handoff and typed
consumer entry only: scheduling it in `SceneRenderer`, all-LOD 09D residency resolution, GPUScene
staging, retirement, and old-path deletion remain pending.

The scene journal now also preserves the exact previous/current primitive `Arc` on updates and seals
a sorted net `UntypedResourceHandle` reference delta for base/all-LOD model, mesh, material,
primitive-binding, material-override, and skeleton dependencies. It scans only additions, removals,
and geometry/material/deformation dirty updates, cancels same-journal acquire/release pairs, and owns
no residency state. This removes the need for a future 09D consumer to keep a third per-primitive
dependency cache or rescan the live scene.

The component converter carries Runtime04's exact `WorldMatrix` into the persistent primitive and
never decomposes it to TRS. This preserves hierarchical shear from parent non-uniform scale plus
child rotation. Journal resident writes borrow that same immutable primitive, so the CPU staging
boundary receives the exact matrix; actual GPU row materialization remains part of the deferred
product staging cutover.

Primitive construction also projects the canonical local-bounds union through that affine matrix
once and retains the conservative world bounds in the immutable payload. The projection uses the
absolute linear matrix for extents, so shear remains conservative; non-affine input and numeric
overflow are rejected before scene mutation. GPU staging still consumes local bounds plus the exact
matrix, while future CPU view culling can borrow the retained world bounds without per-view
reprojection. The shared visibility product path is not changed by this slice.

The source recovery handshake is also explicit. If pending resolution leaves one artifact
unapplied and World advances, the Render03 cursor rejects the newer sparse artifact as a generation
gap. `World::request_full_render_component_projection` coalesces repeated feedback and publishes one
`Full(JournalRequested)` artifact on the next RenderExtract boundary. The consumer can then resolve
the latest complete state atomically; no unbounded artifact history or second state cache is added.
Product scheduling still needs to route this request back to the mutable World owner.

Device recovery now has a separate typed CPU reprojection route. It accepts only the current
`RenderSceneReadView`, rejects world, generation, slot-high-water, resident-count, or per-slot
identity drift before staging, and emits every live primitive as a slot-ordered full write without
advancing or resetting the accepted scene generation. The staging callback returns its output to the
caller; a staging error leaves the CPU consumer unchanged. This recovery plan is `O(N log N)` time
and `O(N)` temporary memory for `N` live primitives, is not used by normal incremental frames, and
performs zero stable-key lookups. Device-generation binding, real buffer recreation, upload, and
measured recovery acceptance remain deferred.

## Current path

1. `sync_gpu_scene_pending_draws` visits every pending draw for one viewport, builds a `HashSet` of
   live keys, calls `GpuScene::register`, writes primitive and instance shadows, and finally calls
   `retain_registered_keys`.
2. Repeated raster ranges for one source key reuse one `GpuSceneEntry`, but scene lifetime is still
   inferred from the camera-filtered draw set. A second viewport repeats registration and comparison.
3. `GpuScene` owns another stable-key `HashMap` and independent primitive allocator even though the
   new `RenderScene` already owns a generational persistent primitive slot.
4. `unregister` releases primitive and instance spans immediately. The allocator's pending-free
   behavior protects same-flush reuse, but lifetime is not tied to the 09A submission-completion
   authority, so a future journal cutover must not treat CPU removal as GPU retirement completion.
5. Previous transforms, skinned palettes/sources, and morph weights are rolled only after successful
   submission. That ordering is correct and must remain attached to the persistent primitive, not a
   view draw.
6. `RenderScenePrimitive` now carries the neutral skeleton resource plus the existing sealed
   `AnimationPoseHandle`, so the journal can identify deformation work without a view draw or copying
   the bone vector. GPU palette allocation, previous pose, upload residency, and
   submission-completion roll remain deferred to this product consumer.
7. `World` now publishes a world-bound, immutable render-dirty entity journal from direct mutation,
   actual lazy `Mut<T>` writes, removals, and derived active/transform propagation. Stable frames
   reuse the same `Arc`; changed entities are sorted and deduplicated once at publication. Component
   ticks therefore classify only these candidates instead of discovering candidates through
   `Changed<T>` scans. The implemented neutral projector preserves component absence explicitly,
   collapses remove/readd, forces full reprojection after removal-history loss, and clones no mesh
   payload for transform-only updates.
8. `GpuSceneJournalConsumer` now owns a world-bound CPU residency table indexed directly by the
   persistent primitive slot. Its ordered sparse overlay is `O(C log C)` time and `O(C)` temporary
   memory for `C` touched residency slots, independent of total scene size `N`; slot order is retained
   so the later WGPU stage can coalesce adjacent writes without another sort. The plan reports exact
   direct-slot validations, projected resident/high-water counts, and zero stable-key map lookups
   without scanning untouched slots. Its resident-write and retirement work is also slot ordered,
   remains `O(C log C)` time / `O(C)` temporary memory, borrows journal primitives, and records
   full/dirty/retirement counts while compiling the work set.
9. `RenderSceneComponentProjector` consumes the neutral world artifact once, rejects foreign,
   stale, or skipped incremental generations before resource work, converts sparse component values
   into camera-neutral persistent primitives, and applies the complete delta atomically. Exact
   replay performs no resolver call. Transform-only changes reuse prior geometry/bounds; Full
   recovery refreshes every surviving primitive and performs its live-key census only on that
   recovery path. Eight authored tests cover retry, multi-entity atomicity, exact hierarchical shear
   preservation, and pending-gap source-requested full recovery, but have no managed Cargo result
   yet.

## Unreal and Lumen evidence

- Unreal's packed `FScene` arrays and `FPrimitiveSceneInfo` are component-level scene primitives;
  their static mesh batches and LOD relevance remain children of that primitive.
- `FGPUScene::AddPrimitiveToUpdate` marks the scene-owned persistent primitive index. `Update` reads
  the scene dirty set; it does not reconstruct lifetime from visible mesh batches.
- Persistent primitive indices and instance ranges have distinct allocation roles. View-selected
  dynamic primitives are appended after the persistent range and do not redefine persistent scene
  membership.
- The Lumen compute sample keeps a persistent scene-instance-to-mesh-card mapping next to mesh-card
  and page-table buffers. Zircon should project this mapping from the same scene slot journal; it
  must not build a second identity table from Lumen-visible draws.

## Accepted consumer contract

### Identity

`RenderScenePrimitiveHandle.slot` is the CPU persistent primitive index. Its `slot_generation` is
validated on CPU and never discarded from cache keys. The preferred cutover maps the slot directly
to the GPU primitive row, matching Unreal's persistent-index model and removing the second
stable-key lookup/allocator. Sparse holes are legal; high-water capacity and fragmentation must be
measured before choosing compaction.

Numeric generation is not a scene identity. `RenderScene`, its journal, each consumer cursor, and
every preflight token carry the source `RenderWorldSnapshotHandle` lineage. A journal or token from
another world is rejected before replay/gap classification, even when both worlds have the same
generation number. Wholesale world replacement must therefore either preserve the authoritative
`WorldHandle` lineage and advance its world generation, or construct a new cursor and perform a
typed full reprojection.

Instance, palette, morph, VG, material-payload, and future Lumen-card ranges remain separate arena
allocations referenced by the primitive row. Their allocation records include primitive handle
generation and device generation. A stale handle cannot write a reused primitive row.

### Transaction order

For one journal, the consumer performs:

1. Call the shared `RenderSceneJournalCursor::preflight`; cross-world input is rejected before
   numeric comparison, same-world replay is an explicit no-op, and stale, skipped, overlapping, or
   inverted generations are typed resync errors. Do not advance the cursor during preflight.
2. Preflight all asset resolution, capacity growth, instance-span changes, and upload-byte budgets
   before mutating live allocation tables.
3. Mark removals inactive and enqueue their dependent ranges for submission-completion retirement.
   Removal never waits for the current camera to stop drawing the object.
4. Install additions at their persistent primitive slots and allocate dependent ranges once.
5. Apply updates by exact dirty domain. Transform does not probe material/geometry; material does
   not rewrite instance transforms; bounds-only updates do not resolve geometry; visibility-only
   updates, including LOD distance threshold edits, do not resolve mesh assets. A cutoff change
   within `Mask` remains material-only, while an `Opaque`/`Mask`/`Blend` transition also invalidates
   command state and view relevance. A skeletal-pose change enters only deformation and bounds
   staging; palette/current-previous state is not inferred from a pending draw. Transform staging
   consumes the primitive's exact `world_from_local` matrix and must not rebuild it from
   translation/rotation/scale components.
6. Seal coalesced upload ranges through the consumer's typed staging callback and commit the cursor
   only after that callback succeeds. The callback must leave its external owner unchanged on error;
   the consumer returns its successful staging output and skips it entirely for replay. A
   foreign-world token is rejected, and a stale token is rejected if another transaction advanced
   the cursor. WGPU submission failure keeps the journal pending and does not roll previous state.

The CPU transaction is atomic. Missing assets produce typed pending residency on the primitive and
conservative visibility; they do not silently remove scene identity. Device recreation invalidates
GPU allocations but not the CPU scene generation. The CPU full-write reprojection plan is
implemented; device-generation-qualified WGPU recreation and its measurements are not.

Do not materialize an intermediate row template that fills unresolved fields with plausible
defaults. Primitive/instance spans belong to capacity preflight; material, VG, morph, lightmap, and
palette slots belong to their residency owners; previous-transform/deformation availability belongs
to accepted submission history. The row is sealed only after those owners resolve or explicitly
publish typed pending state. The CPU residency plan intentionally returns the immutable journal and
ordered slot mutations so that later staging can consume the real inputs without another scene scan.

### Bounds and LOD

The primitive constructor now requires an explicit base plus one local-bounds input per LOD source,
rejects a count mismatch, validates every AABB, and stores their conservative union. The primitive
also stores one conservative world envelope projected from that union at construction. The GPU row
continues to receive local bounds plus the exact instance matrix, avoiding a coordinate-space ABI
change. The view chooses an LOD from
`RenderSceneMeshSource::select_for_distance` after visibility candidate selection. Every selected
mesh batch references the same persistent primitive row and instance transform. Camera motion
therefore changes view commands only and never writes camera-neutral GPU primitive data.

### Required counters

- applied/replayed/skipped scene generation;
- journal additions, updates, removals, and dirty-domain counts; the CPU journal already seals the
  seven domain entry counts during classification, so product diagnostics must project them without
  rescanning update payloads;
- stable-key probes after cutover, required to be zero in the GPUScene consumer;
- primitive high water, holes, fragmentation, grown bytes, and deferred-retired bytes; the CPU
  journal now seals live count, persistent-slot high-water, reusable holes, generation-exhausted
  slots, and total fragmented slots without a diagnostic rescan;
- dependent arena allocations/reuses/retirements by cause;
- primitive/instance/material/deformation/bounds upload rows, ranges, and bytes;
- asset pending/resolved/fail-open counts;
- stable-generation visits, comparisons, allocations, GPU creates, and uploads, all required zero.

The implemented CPU apply plan already exposes direct-slot validation count, projected resident
count, projected slot high water, full/dirty resident-write counts, exact instance-transform and
local-bounds staging counts, retirement count, and a zero stable-key lookup count. The transaction
result carries the successful staging output instead of requiring a captured side channel. The
recovery plan separately exposes its source world/generation, slot high water, resident/full-write
count, instance-transform/local-bounds full-write counts, direct-slot validation count, and zero
stable-key lookup count. Product diagnostics still need to project these values together with
journal dirty-domain counts and WGPU upload/retirement data.

## Implementation sequence

1. The Runtime04-boundary world-owned render-dirty journal, persistent removal cursors, fixed
   component-tick classifier, immutable sparse patch artifact, and Render03 resolved-input internal
   delta conversion are implemented. The artifact is core-neutral and now crosses the
   `GeometryExtract` boundary as a shared `Arc`; the Render03 projector has the corresponding typed
   frame entry with external world-lineage validation. A source-side coalesced full-reprojection request now recovers
   consumer generation gaps, but product scheduling must route that typed feedback to World. Next
   connect the converter to the unified residency ticket and typed pending/fail-open primitive
   state; component ticks remain candidate classifiers rather than full-row discovery scans.
2. The shared CPU generation cursor and nine focused preflight/commit tests are implemented under
   `render_scene/`. The direct-slot CPU residency/work consumer, single staging-before-commit product
   route, typed device-recovery reprojection plan, and eight focused tests are implemented under
   `gpu_scene/journal_consumer`; it does not call WGPU while consumer-specific asset/capacity
   preflight can still fail. The work plan deliberately stops before row materialization because
   dependent arena slots and previous-state availability are unresolved owner outputs.
3. Change GPU primitive indexing to persistent RenderScene slots, retaining generation in CPU
   allocation records and cache identities.
4. Move current primitive/instance shadow writes from pending draws to dirty-domain handlers.
5. Connect retirements to 09A completion tickets, then move previous transform/deformation roll to
   the accepted submission for that scene generation.
6. Run old/new parity per frame, hard-cut registration/retain from pending draws, and delete the old
   ownership path in the same accepted milestone.

## Acceptance

The consumer is not accepted until managed tests and a real product run prove: stable scene work is
zero; two viewports do not multiply scene-global work; a one-item transform change touches one
instance row without material/geometry probes; removal cannot alias an in-flight slot; camera-only
LOD changes do not alter scene generation; device recreation performs one typed full reprojection;
RenderDoc shows the expected range uploads; and the resulting framebuffer PNG is stored under
`docs/tests/runtime/render`.
