---
title: Persistent RenderScene generation and change-journal architecture review
status: core_neutral_component_delta_render_extract_handoff_world_bounds_local_bounds_staging_qualifier_and_source_resync_implemented_static_validated_product_residency_wgpu_deferred
owner: render-03-gpu-scene
date: 2026-08-26
related_code:
  - zircon_runtime/src/graphics/scene/render_scene
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/core/framework/render/frame_extract/scene_changes
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrimitiveDataRenderer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrimitiveUpdates.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PrimitiveSceneInfo.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
doc_type: architecture-review
---

# Persistent RenderScene generation and change journal

## Review result

The current `CompiledSceneDraws` is a frame-local draw product, not a compiled or persistent scene.
`render_compiled_scene` still receives a complete `ViewportRenderFrame`, walks its geometry, rebuilds
mesh draws, registers entries in `GpuScene`, retains a full-frame live-key set, and increments
`mesh_command_generation` every frame. A stable scene therefore cannot make scene preparation work
zero, and multiple views repeat scene-global work.

The structural correction is to introduce one CPU-side `RenderScene` authority ahead of view
projection. Runtime04 must publish a graphics-independent, camera-neutral world/component change
journal; Render03 resolves asset state and applies an internal `RenderSceneDelta` once per world
snapshot, not once per viewport. `GpuScene`, mesh-command generation, visibility, shadow
invalidation, VG, and HGI then consume the same sealed change journal and generation. A view owns
only camera-relative visibility, LOD selection, and temporal history.

This record is an architecture and implementation record, not performance acceptance. No WPR,
WGPU, RenderDoc, power, or product-frame numbers are claimed while the managed runtime lane remains
blocked by the separate UI asset migration.

## Current-source findings

1. `SceneRendererCore` owns `GpuScene`, mesh command caches, indirect workspaces, and view-independent
   renderer resources, but it does not own a persistent CPU render scene.
2. `build_compiled_scene_draws` accepts the entire `ViewportRenderFrame`; its name describes output
   compilation only. The function immediately delegates to per-frame mesh-draw construction.
3. `GpuScene` has stable-key lookup and dirty GPU ranges, but registrations are inferred from pending
   draws. `retain_registered_keys` requires a full live-key census, so `GpuScene` is currently both a
   derived GPU mirror and a partial scene-lifetime detector.
4. `RenderMeshSnapshot` is view-derived, not merely a scene DTO. `scene/world/render.rs` selects a
   LOD from camera distance and replaces the complete `model`, `mesh`, `material`, and primitive
   binding list in addition to setting `mesh_lod`. Excluding only the marker would still contaminate
   the scene generation. The persistent primitive must own the base source and every LOD source;
   view-family projection selects one source later. The current extract also lacks resolved
   conservative all-LOD local bounds and a scene-level delta.
5. HZB local-bounds correctness is now explicit, but CPU visibility still uses an independent proxy.
   Threading another resource lookup through per-view visibility would preserve the wrong owner and
   is rejected.
6. `World` already records `ComponentTicks` and bounded `RemovedComponentEvents`, but
   `RenderExtractProducer` remains a stateless `&self -> RenderFrameExtract` API and clones the world
   for each request. A render delta must be sealed once for a world snapshot/change-tick window and
   shared by all viewport extracts; emitting it independently from each camera would duplicate work
   and make removal-reader continuity ambiguous.
7. `Changed<T>` queries are not a substitute for that sealed journal. They still scan the candidate
   component rows before testing ticks, so a query-based producer would revisit every
   `MeshRenderer` row in a stable generation and repeat the scan for each relevant component type.
   `World` must append entity IDs to one deduplicated render-dirty journal at the actual mutation or
   derived-state propagation boundary. Component ticks then classify only those candidates; the
   persistent removal reader supplies removals.

## Unreal ownership evidence

The target follows Unreal's responsibility split rather than copying surface names:

- `FScene` keeps packed primitive arrays (`Primitives`, `PrimitiveTransforms`,
  `PrimitiveSceneProxies`, `PrimitiveBounds`) and maps persistent primitive IDs to packed indices.
- `FScenePrimitiveDataRenderer` owns registered data arrays and their shared dirty tracking.
- `FScenePrimitiveUpdates` classifies transform, instance, culling-bounds, culling-logic, and GPU
  state changes. Pre-update and post-update change sets separate removed and added primitive lifetime.
- One `FPrimitiveSceneInfo`/proxy owns multiple static mesh batches and their LOD relevance. A mesh
  batch points back to its primitive; selecting a batch/LOD does not create a different persistent
  scene primitive identity.
- `FGPUScene::AddPrimitiveToUpdate` marks the scene-owned dirty set; `FGPUScene::Update` consumes it.
  GPUScene does not rediscover scene membership from each view's draw list.
- Instance slots are allocated/freed persistently; Lumen is notified from the same primitive
  allocation/update event instead of maintaining a separately inferred scene identity.
- The standalone `dev/LumenInUE5.5.4WithComputeShader` sample binds a persistent
  `SceneInstanceIndexToMeshCardsIndexBuffer` beside mesh-card and page-table buffers throughout the
  surface-cache/tracing chain. It is useful ABI evidence for scene-index-to-Lumen-data projection,
  but its fixed global buffers are not a dynamic lifetime model to copy.

## Accepted Zircon contract

### Single authority

`graphics/scene/render_scene/` owns:

- one `RenderWorldSnapshotHandle` lineage identity retained by the scene, read view, journal,
  consumer cursor, and preflight token;
- compact persistent `RenderScenePrimitiveHandle { slot, slot_generation }` values;
- a stable-key-to-handle map;
- dense primitive storage plus handle indirection, with O(1) swap-remove compaction;
- one component-level `RenderScenePrimitive` whose stable identity is not multiplied by mesh section
  or selected LOD;
- the exact `world_from_local` matrix sealed by Runtime04 derived-state propagation; Render03 never
  decomposes it to TRS, so parent non-uniform scale plus child rotation cannot lose shear before the
  GPUScene staging boundary;
- one conservative world AABB/sphere projected from the canonical local bounds at primitive
  construction using `world_center = M * local_center` and
  `world_extent = abs(M3x3) * local_extent`; view consumers borrow it instead of repeating the
  affine transform for every camera;
- a camera-neutral mesh source containing the base source and all LOD source alternatives, rather
  than any model/mesh/material already selected for a camera;
- validated, canonical LOD thresholds with O(log L) per-view source selection;
- a neutral current skeletal pose input attached to the component-level primitive by reusing the
  Runtime04-sealed `AnimationPoseHandle`; GPU palette slots, previous pose, residency, and retirement
  remain `GpuScene` responsibilities;
- explicit transform, geometry, material, bounds, and deformation revisions that are independent of
  static-mesh eligibility;
- immutable primitive payloads shared by `Arc`;
- a monotonic scene generation advanced once per effective delta;
- an immutable journal split into removals, updates, and additions.
- sealed apply counters for input mutations, stable-key probes, payload comparisons, all seven dirty
  domains, handle-slot reuse/append, and dense relocations, produced without a diagnostics rescan.

`GpuScene` remains the GPU resource owner. It must consume `RenderSceneChangeJournal`; it must not
become the CPU scene authority. View-family code borrows a sealed `RenderSceneReadView` and creates
per-view visibility bitsets against that generation.

The public producer contract cannot use `graphics::scene::RenderSceneDelta`: `scene::World` is built
without the `graphics` feature. Runtime04 owns a neutral component-change artifact using scene/core
resource handles and change ticks. Render03 owns the conversion from that artifact into validated,
asset-resolved internal primitives. A consumer cursor must reject another world even when its numeric
generation matches, reject a skipped generation, and ignore an already-applied journal from the same
world so two viewports do not apply the same change twice.

When a resolver leaves an artifact unapplied and the source later advances past it, the consumer
must not apply the latest sparse artifact across that gap. It returns a typed discontinuity; the
source-side `World::request_full_render_component_projection` coalesces repeated requests and makes
the next RenderExtract publication `Full(JournalRequested)`. This recovers the latest complete state
without an unbounded artifact history or a second scene cache.

### Delta semantics

The product path accepts an incremental `RenderSceneDelta`, never a camera-filtered full snapshot.
The delta is validated before mutation:

- duplicate upserts are rejected;
- the same stable key cannot be removed and upserted in one delta;
- a stable key cannot silently change owning entity;
- non-finite world matrix, tint, morph, skeletal-pose transform, material-property override, or
  bounds inputs are rejected at primitive construction; alpha-mask cutoff is additionally
  constrained to `[0,1]`;
- input order is normalized by stable key so journal order and slot assignment are deterministic.

A no-op upsert does not advance generation. Removal invalidates the old handle before the slot can be
reused. Each removal records its swap-remove relocation so dense-index consumers can update without a
full scene scan. Journal payloads retain `Arc` ownership and remain valid after later scene updates.

### Dirty domains

The first contract distinguishes seven diagnostic domains: `TRANSFORM`, `GEOMETRY`, `MATERIAL`,
`DEFORMATION`, `RENDER_STATE`, `VISIBILITY`, and `BOUNDS`. `LOCAL_BOUNDS` is a staging qualifier,
not an eighth diagnostic domain: it marks a change to the local-space bounds stored in the GPU
primitive row, while `BOUNDS` continues to invalidate the conservative CPU world envelope. The
classifier uses the following minimal invalidation matrix:

| Changed source | Published domains |
|---|---|
| transform or transform revision | `TRANSFORM | BOUNDS` |
| model/mesh source or geometry revision | `GEOMETRY | VISIBILITY` |
| LOD distance threshold only | `VISIBILITY` |
| local bounds or bounds revision | `BOUNDS` plus `LOCAL_BOUNDS` staging qualifier |
| material binding, tint, material revision, property override, or same-phase mask cutoff | `MATERIAL` |
| morph weights, skeletal pose, or deformation revision | `DEFORMATION | BOUNDS` |
| queue, ordering, depth, static eligibility, receive-shadow, or motion-vector state | `RENDER_STATE` |
| mobility | `RENDER_STATE | VISIBILITY` |
| cast-shadow state | `RENDER_STATE | VISIBILITY` |
| `Opaque`/`Mask`/`Blend` phase transition | `MATERIAL | RENDER_STATE | VISIBILITY` |
| enabled, layer, or LOD-group state | `VISIBILITY` |

Mesh/model changes, LOD policy, local bounds, and material binding changes are compared separately.
A threshold-only LOD edit therefore changes view selection without entering geometry asset
resolution, a bounds-only edit does not enter geometry asset resolution, a geometry-only edit does
not upload an unchanged bounds row, and changing a cutoff within `Mask` does not rebuild phase
commands. Consumers may subscribe to the narrow flags they need; they must not hash the whole
primitive again.

This split is required by the current WGPU ABI: transforms live in `GpuInstanceData`, while local
bounds live in `GpuPrimitiveData`. A transform-only update therefore publishes
`TRANSFORM | BOUNDS` for exact CPU visibility but seals one instance-transform write and zero
local-bounds writes. Additions and device reprojection require both writes. This is typed staging
metadata only; the product WGPU path is not cut over by this slice and no runtime performance claim
is made.

## Complexity and performance gates

The implemented data structure must satisfy these algorithmic bounds before product wiring:

| Operation | Required scale |
|---|---|
| stable-key lookup | expected O(1) |
| primitive update | expected O(1) plus payload comparison |
| add | expected O(1), lowest reusable handle slot O(log free slots) |
| remove and dense compaction | expected O(1) plus O(log free slots) |
| delta normalization | O(k log k) for input-order-independent deterministic journals |
| no-op delta after normalization | O(k), zero scene-generation advance, zero journal entries |
| view read | borrowed immutable generation, zero scene clone |
| journal retention | O(changed entries), payload handle clones only |
| GPUScene residency preflight | O(C log C), O(C) temporary state for changed residency slots, zero stable-key map probes |
| per-view LOD source selection | O(log L) after source canonicalization |
| world-bounds projection | O(1) once per added/changed primitive, zero projection work per view |

The sort is paid once at the scene mutation boundary; consumers do not sort their own copies. A
future producer may publish a validated canonical delta to remove this cost, but it must preserve the
same deterministic contract and measurements must justify the extra API. This slice deliberately
does not add a `reconcile_full_snapshot` product API. Such an adapter would
make an O(scene) census look canonical and would preserve the current per-view architecture. The
later world/extract cutover must emit actual add/update/remove deltas.

## Implementation sequence

1. Add the pure CPU `RenderScene` owner, typed errors, immutable primitive payload, generation read
   view, deterministic delta validation, dense storage, and focused tests.
2. Add a graphics-independent Runtime04 component-change producer backed by a world-owned,
   deduplicated render-dirty entity journal populated by mutation and derived-state propagation.
   Use `ComponentTicks` to classify only journal candidates and a persistent
   `RemovedComponentReader<MeshRenderer>` for removals. Seal it once per world snapshot/change-tick
   window and share it across viewport extracts. Do not derive this delta inside
   `render_compiled_scene`, and do not replace the journal with full candidate-row `Changed<T>`
   queries.
3. Resolve base/all-LOD assets once in Render03, compute conservative all-LOD local bounds, and
   apply the internal `RenderSceneDelta`. Missing assets publish typed pending/fail-open state rather
   than silently deleting the primitive.
4. Move `GpuScene` registration, previous-transform lifetime, local bounds, and dirty uploads to the
   journal consumer. Delete full-frame `retain_registered_keys` ownership after parity validation.
5. Build view-family visibility from one sealed generation. CPU and HZB visibility consume the same
   local bounds and revisions.
6. Move static mesh commands, indirect plans, VG/HGI scene inputs, and shadow invalidation onto the
   same generation identity. Remove frame-number cache identities where scene content is unchanged.

## Implemented static slice

The first CPU owner now exists under `zircon_runtime/src/graphics/scene/render_scene/`:

- `core/framework/render/frame_extract/scene_changes/` now owns the immutable, graphics-independent
  sparse component artifact contract. Scene-only `MeshRenderer`, `WorldMatrix`,
  `ActiveInHierarchy`, and `RenderLayerMask` wrappers are projected once into resource handles,
  exact `Mat4`, `bool`, `u32`, and core `Mobility`; base/all-LOD source lists and morph payloads are
  retained behind immutable `Arc` slices. The former scene-owned artifact and mask definitions were
  deleted, so `graphics/render_scene` no longer imports a `scene::world` product DTO;
- `GeometryExtract::scene_changes` carries one optional `Arc<RenderComponentChangeArtifact>`.
  `World::build_prepared_render_frame_extract_for_request` attaches the world-published artifact for
  both active and inactive cameras after `RenderExtractPrepare`; stable and multi-viewport extracts
  therefore clone only the same `Arc`, not component payloads or a per-camera delta. The graphics
  component projector exposes `project_frame`, validates `RenderFrameExtract.world` against its
  persistent scene before invoking resource resolution, and consumes the artifact through this real
  frame-extract boundary;
- `mod.rs` is a 40-line curated facade;
- `journal_cursor.rs` is a 294-line world-bound consumer-generation preflight/commit state machine;
  exact same-world replay is a no-op, cross-world/stale/gapped/inverted/non-adjacent journals are
  typed errors, the preflight token is opaque outside its owner, and commit independently rechecks
  its range before a foreign, obsolete, or malformed token can advance a cursor;
- `mesh_source.rs` is a 205-line base/all-LOD source owner, validator, canonicalizer, split
  geometry/material/LOD-policy comparator, and binary-search view selector;
- `deformation.rs` is a 60-line neutral skeleton-resource/current-pose owner that retains the sealed
  `AnimationPoseHandle` without copying bones and validates every bone transform; it intentionally
  owns no GPU palette, previous-pose, or residency state;
- `core/framework/render/mesh/bounds.rs` is a 149-line shared bounds owner whose affine projection
  handles rotation, non-uniform scale, and shear with the exact center/extent algorithm. It rebuilds
  derived center/radius from min/max instead of trusting externally deserialized metadata. Three
  focused tests cover TRS, shear, and stale-metadata canonicalization;
- `primitive.rs` is a 545-line component-level descriptor, explicit revision bundle, finite/range
  input validator, base/all-LOD bounds-count validator and conservative union, exact canonical
  per-LOD bounds owner, bounds canonicalizer, and dirty-domain classifier. Its transform authority
  is the exact `world_from_local` matrix rather than a lossy TRS decomposition, and it caches the
  matching conservative world bounds once. Non-affine matrices and finite-input projection overflow
  are typed construction errors. LOD source sorting now reorders the matching bounds instead of
  discarding their association, and a per-LOD bounds change remains `BOUNDS` dirty with the
  `LOCAL_BOUNDS` staging qualifier even when the conservative union is unchanged;
- `change_journal.rs` is a 445-line immutable, world-tagged `Arc` journal and sealed apply-counter contract,
  including transform/geometry/material/deformation/render-state/visibility/bounds entry counts
  accumulated during classification rather than by rescanning the journal, plus the resulting
  generation's O(1) storage statistics. Updated rows retain both the exact previous and current
  primitive `Arc`, so downstream consumers do not need a second per-primitive shadow cache;
- `resource_dependencies.rs` is a 301-line changed-frontier projector. It derives one deterministic
  net typed-resource delta from add/remove and dependency-dirty before/after rows, covering base and
  all LOD model/mesh/material sources, primitive bindings, material overrides, and skeletons. Two
  reusable scratch vectors plus one contiguous observation buffer replace per-primitive tree/hash
  allocation; duplicate references and opposing same-journal changes cancel before publication;
- `scene.rs` is a 631-line world-bound stable-key map, compact generation handle table, dense payload
  array, lowest-free-slot allocator, swap-remove relocation publisher, and atomic delta validator;
  construction requires an explicit world identity, exposes no anonymous `Default` path, and
  publishes live primitive, persistent-slot high-water, reusable-hole, generation-exhausted slot,
  and total fragmentation counts without scanning slots;
- `tests.rs` is a 696-line focused owner with 25 general behavior tests, including a 10k
  primitive/single changed-entry contract, dynamic-primitive material revision coverage,
  geometry/material/bounds dirty-domain separation, handle high-water/hole/reuse statistics,
  mobility and shadow relevance invalidation, alpha cutoff range/phase narrowing, and non-finite
  world-matrix/material-override rejection, exact shear world bounds, and projection-overflow
  rejection, plus exact before/current `Arc` retention on transform updates;
- `tests/fixtures.rs` is a 127-line folder-backed helper owner, keeping fixture construction out of
  the root behavior owner before it reaches the 800-line warning;
- `tests/mesh_source.rs` is a 265-line folder-backed owner with nine camera-neutral source/LOD
  tests, including binary-search selection, invalid threshold rejection, source-domain separation,
  the threshold-only `VISIBILITY` contract, base/all-LOD bounds union, and LOD bounds-count
  rejection, plus canonical source/bounds alignment and equal-union per-LOD bounds invalidation;
- `tests/deformation.rs` is a 129-line folder-backed owner with three skeletal-pose tests covering
  sealed-handle identity, exact `DEFORMATION | BOUNDS` publication, and non-finite
  translation/rotation/scale rejection;
- `tests/resource_dependencies.rs` is a 310-line folder-backed owner with five tests covering inverse
  add/remove references, exact material before/current `Arc` retention, material-only replacement
  after unchanged dependency cancellation, cross-primitive replacement cancellation, and complete
  all-LOD/binding/override/skeleton coverage;
- `journal_cursor/tests.rs` is a 221-line folder-backed owner with nine world/generation transaction
  tests, including forward generation skips, wide overlapping replay, and forged non-adjacent
  commit-token rejection;
- `component_projector/` is the world-artifact-to-RenderScene conversion boundary. Its 397-line
  projection owner converts full and sparse component snapshots, keeps camera-neutral base/all-LOD
  mesh sources and exact world matrices, preserves unchanged geometry and bounds on transform-only
  patches, forces fresh geometry resolution on full reprojection, performs the O(N log N) live-key
  census only on full recovery, and builds the complete delta before mutating the scene or advancing
  its artifact generation. It accepts resolved geometry through a narrow resolver trait and
  publishes typed pending/missing/invalid resolution errors; it does not invent fallback bounds or
  own residency;
- `component_projector/tests.rs` is a 452-line folder-backed owner with nine tests for initial/full
  application and exact replay, transform-only geometry reuse, same-artifact retry after pending
  resolution, full-resync removal and forced geometry refresh, multi-entity transaction atomicity,
  incremental generation-gap rejection before asset work, and preservation of hierarchical shear
  that a TRS recomposition would lose. The recovery test covers pending resource work followed by a
  skipped sparse artifact and source-requested full recovery; the frame-boundary test rejects a
  mismatched external world before resolver work. The RenderScene subtree therefore has 60 focused
  tests in total;
- `gpu_scene/journal_consumer.rs` is a 553-line world-bound direct-slot residency and transaction
  owner. It uses a slot-ordered sparse projection instead of a second stable-key map or primitive ID
  allocator, validates only journal-touched slots, maintains resident/high-water counters without a
  scan, and exposes one staging-before-commit product route. Internal preflight/commit cannot be
  called by sibling production owners; staging failure leaves cursor/residency unchanged, successful
  apply returns its staging output, and exact replay skips the staging callback;
- `gpu_scene/journal_consumer/work.rs` is the slot-ordered full/dirty write and retirement
  projection owner. It borrows journal primitives and seals instance-transform and local-bounds
  staging counts together with full/dirty counts, so the future WGPU stage does not have to conflate
  world-envelope invalidation with a local-bounds row write;
- `gpu_scene/journal_consumer/reprojection.rs` is a 238-line typed device-recovery owner. It validates
  the current read-view world, generation, slot high water, resident count, and every direct slot,
  then emits slot-ordered full writes, including exact instance-transform and local-bounds write
  counts, without resetting CPU generation or consulting a stable-key map. Its `O(N log N)`/`O(N)`
  full-scene work is isolated to recovery and not used by steady frames;
- `gpu_scene/journal_consumer/tests.rs` is a 453-line folder-backed owner with six focused tests for
  initial projection, same-journal remove/add slot reuse, exact replay, wrong resident identity,
  stale-plan rejection, exact dirty/retirement work with unchanged world-matrix payload, and
  staging-before-commit atomicity;
- `gpu_scene/journal_consumer/tests/reprojection.rs` is a 137-line folder-backed owner with two
  focused tests covering hole-preserving slot order, full-write classification, unchanged CPU
  generation on success/failure, and world/generation drift rejection before staging. Together with
  the RenderScene subtree, this slice has 68 authored tests;
- `scene/world/render_dirty_journal/` is the scene-side, graphics-independent publication owner. It
  assigns a runtime-only world identity, records a monotonic journal generation plus source world
  generation/change tick, retains an immutable changed-entity list, and reuses the exact same `Arc`
  on stable frames;
- `scene/ecs/change_detection/component_mutation.rs` records only actual lazy `Mut<T>` writes. The
  first mutable dereference appends one typed mutation record; read-only `Mut<T>` access does not
  advance world generation or publish render work. World-side replay restores inspection, binding,
  hierarchy, active, transform, node-cache, and render-dirty side effects before derived systems;
- the folder-backed `scene/tests/render_dirty_journal/` owner adds five authored tests for world
  identity/stable `Arc` reuse, sorted deduplication with removals, descendant transform propagation,
  read-only versus actually-written lazy queries, effective generation across clone, and lazy
  transform-derived-state freshness;
- `scene/world/render_component_changes/` adds the world-bound neutral projector. It consumes each
  publication once, probes only the fixed five component ticks for candidate entities, owns bounded
  removal cursors for lifetime and optional-state loss, and publishes sparse
  `Unchanged`/`Present`/`Removed` values in one immutable artifact. Seven focused tests cover full and
  stable replay, narrow classification, optional removal, mesh removal/readd, history-loss resync,
  zero mesh-payload clones for a transform-only update, and coalesced source-side full-reprojection
  requests. An eighth source regression proves active/inactive frame extraction retains the exact
  world-published `Arc` across a stable repeat. The combined CPU architecture now has 81 authored
  tests.

Scoped `rustfmt --check`, untracked whitespace checks, production forbidden-pattern checks, and the
camera-neutral ownership scan pass. The new component converter production files contain no
`unwrap`, `expect`, `panic!`,
`allow(dead_code)`, WGPU type, full-snapshot reconcile API, viewport frame, or GPUScene dependency.
The scoped second review corrected the journal-cursor and GPUScene-consumer tests to the required
`render_` filter prefix. A final combined scan confirms all 81 authored test names are unique and
use that prefix. All newly added production owners are below the 800-line review warning. The
existing `scene/world/render.rs` orchestration owner is 816 lines; this handoff adds only the
same-responsibility `Arc` attachment and remains inside the modularization rule's under-900 small-edit
range. The consumer contains zero secondary `HashMap`, primitive
allocator, or pending-draw registration symbols; and the only four `HashMap` textual references are
the CPU RenderScene stable-key authority. No remaining actionable static P0/P1/P2 was found in this
exact CPU slice.
The repository-wide structure gate did not complete within a 60-second local window and produced no
result; this is not recorded as pass or failure. Managed Cargo remains unresolved, so the 81 tests in
this architecture are authored but not executed in this record. The earlier producer check was
rejected before Cargo by `unmanaged_artifacts_detected`; the later focused projection request was
accepted but returned `command_post_timeout` during `cargo.acquire` without a terminal result.
Neither is counted as a compile pass.

The neutral source artifact now reaches `RenderFrameExtract` and the Render03 projector has a typed
frame consumption entry with external world-lineage validation. Product scheduling deliberately does
not invoke it yet: the
current synchronous `ResourceStreamer` cannot satisfy the required unified 09D residency ticket,
all-LOD pending/fail-open semantics, or no-third-cache rule.

The retained world bounds are now available on the sealed primitive, but the existing CPU
visibility product path still constructs its legacy translation/scale proxy. That owner and the new
view-family tree contain concurrent worktree changes, so this slice deliberately stops before the
consumer hard cut. It does not claim that CPU visibility already consumes the persistent bounds.

The render-dirty producer has no stable-frame scene scan: checking pending work and cloning the
published handle are O(1), with zero changed-entity allocation when the generation is stable. One
actual lazy query mutation performs one O(1) append behind an uncontended world-owned lock; direct
mutations append to the world-local pending vector. Publication is O(C log C) time and O(C) memory
for C recorded/propagated candidates because it sorts and deduplicates once. Derived transform and
active propagation add one O(1) mark to their existing subtree traversal, so they do not introduce a
second traversal. These are source-level complexity bounds, not measured timing or power results;
lock contention and allocation cost must be profiled before any algorithmic replacement.

## Profiling plan before optimization wiring

The structural owner can be implemented without claiming a speedup. Before replacing the product
path, capture a current-source baseline with WPR/xperf and renderer counters for 1, 1k, 10k, and 100k
static primitives, one and four views:

- frame extract geometry visits and bytes;
- pending draw builds and stable-key probes;
- GPUScene registrations, live-key census visits, dirty entries, ranges, and uploaded bytes;
- mesh command builds/cache hits;
- visibility bounds resolutions and BVH/HZB inputs;
- CPU p50/p95 preparation time, GPU pass time, GPU object creation, and queue-write bytes;
- process CPU/GPU utilization and package/GPU power where the host exposes trustworthy counters.

### Static finding: fragmented-slot planning

Current-source review found one unmeasured algorithmic risk inside `RenderScene::apply_delta`.
Every effective delta calls `plan_addition_handles`, which clones the complete
`free_handle_slots` heap before it knows whether any additions exist. With `H` reusable holes, a
single transform-only update can therefore perform O(H) clone work even though the changed set is
one primitive. This violates the required sparse-update scale, but it is not yet called a measured
bottleneck because the managed product binary is unavailable.

Before changing this algorithm, capture WPR/ETW CPU samples and sealed counters for 100k live
primitives at 0%, 25%, and 50% handle fragmentation, applying independently: one transform update,
one removal, one addition, and a 1% mixed delta. Record `apply_delta` duration, heap clone/allocation
bytes, reusable-slot visits, additions, and changed entries. The proposed correction, only after
that baseline, is to split fallible capacity/generation preflight from commit-time allocation:
preflight counts reusable removed/free slots without cloning the heap, removals publish their slots,
and sorted additions pop the real min-heap or append prevalidated slots. Acceptance requires
transform-only and removal-only planning to visit/clone zero reusable slots, while deterministic
lowest-slot assignment, atomic error behavior, and journal order remain unchanged. No timing or
speedup claim is made in this record.

The current `plan_addition_handles` clones the complete free-slot heap before choosing even one
slot. This is a structural candidate, not an accepted bottleneck. Add a fragmented-handle profile
with 1, 1k, and 100k free slots and addition batches of 1/16/1k; record preflight CPU p50/p95/p99,
free-slot entries visited/copied, and allocation bytes. Only replace the heap/preflight algorithm if
the trace confirms work scales with total holes rather than requested additions. The target after a
measured change is `O(additions log free_slots)` and zero full free-list clones.

The CPU owner now seals live primitives, persistent-slot high-water, reusable holes,
generation-exhausted slots, and total fragmented slots into each apply report and exposes the same
snapshot through its read view. These counters are measurement inputs only; no allocator speedup is
claimed until WPR/xperf supplies time and allocation evidence.

After cutover, a stable generation must report zero scene delta entries, zero GPUScene registration
work, zero scene-global command rebuilds, and zero scene-global work multiplication from additional
views. Changed work should scale with dirty entries plus visible view projection. RenderDoc evidence
must show the resulting WGPU resource/update path, and a real framebuffer PNG must be written under
`docs/tests/runtime/render` before this milestone can be accepted.

## Open constraints

- The managed runtime validation lane has no current terminal result: the latest focused request
  timed out during `cargo.acquire`. Raw Cargo is not used and the coordinator is not polled.
- `ResourceState` and `ResourceLease` already exist, while the current renderer `ResourceStreamer`
  still performs synchronous full-asset preparation and owns a second prepared cache. RenderScene
  must not add a third sidecar residency authority. The Runtime04/Render03 producer must consume the
  unified 09D residency ticket/generation contract when it becomes available. The pure CPU
  component converter now accepts already-resolved exact base/all-LOD bounds and typed resolution
  failure, but no `ResourceStreamer` adapter, residency ticket, conservative pending visibility, or
  fail-open product state is claimed. The core-neutral artifact and `GeometryExtract` handoff are
  complete; only the residency-backed scheduling/application edge remains open.
- `SceneRendererCore`, compiled-scene execution, and the main `GpuScene` owner currently contain
  extensive foreign worktree changes. This slice adds an isolated journal-consumer child plus curated
  facade wiring and avoids overwriting those foreign behavior owners.
- The scene `World`/change-detection owners contain extensive concurrent worktree changes. The
  render-dirty producer was forward-merged into their current structure without replacing the
  existing incremental hierarchy/frontier work. Component classification, bounded removal cursors,
  sparse neutral patch production, stable replay, and history-loss full reprojection are present.
  The resolved-input internal `RenderSceneDelta` conversion is present; unified asset residency and
  product pending/fail-open state remain pending.
- No real WGPU run, Naga validation, RenderDoc capture, PNG, WPR trace, or power comparison exists for
  this record yet. Status therefore remains implementation in progress.
