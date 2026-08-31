# Static Mesh Command Cache View-Local Projection ABI Research

Date: 2026-08-26

Status: static payload/current-frame projection ABI implemented; managed compile, runtime, profile, RenderDoc and PNG acceptance remain pending.

Plan owner: `docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md` (MD-M2 and MD-M3).

## Finding

`CachedMeshDrawCommands` now owns `Arc<MeshDrawCommandPayload>` rather than a complete visible
command. The payload contains phase/pipeline selection, material and geometry handles, and only
the direct-indexed topology required by the static cache. `MeshDrawCommand` is the current-frame
projection: it owns source identity/order, sort key, GPU Scene instance source, current direct draw
span and current GPU Scene binding, while dereferencing to the immutable payload for replay.
Uncached commands retain the payload inline; cache admission moves it once into an `Arc`, so
dynamic/indirect commands do not acquire a per-command heap allocation from this split.

Both serial and parallel builder hits call `MeshBatchRef::project_cached_command`, which creates a
new visible command over the cached `Arc` without resolving a pipeline variant, running a pass
processor, or cloning material/geometry WGPU handles. Indirect commands are not cacheable and do
not duplicate their indirect buffer handle into the payload.

The pre-MeshDraw extraction owner does not have a complete current `MeshBatchRef`; it now carries
the synchronized GPU Scene span and the current `RenderPhaseSortComponents` on its extract item.
A span lookup is deliberately deferred until after static-cache eligibility, because extraction
visits every pending draw and dynamic/residual draws must not pay an additional GPU Scene map
lookup. The resulting span is shared by the cache-hit projection and a non-material miss rebuild.
A pre-MeshDraw hit calls `MeshDrawCommand::from_cached_payload` with explicit current entity,
source order, sort input and synchronized GPU Scene span. A missing synchronized span is not
allowed to consume a cached payload and returns the draw to normal MeshDraw handling. The same
captured span is passed to a non-material miss rebuild, so the hit and miss branches cannot
observe different GPU Scene allocations for one pending draw. Pre-MeshDraw projection deliberately
does not retain a cached current-frame GPU Scene bind group; replay uses the current frame context
fallback for these eligible non-skinned direct draws.

That cache ABI currently combines two lifetimes:

| Data | Current source of truth | Required lifetime |
| --- | --- | --- |
| Pipeline variant, material and geometry binding handles, direct draw topology | pass processor output | static entry, invalidated by static state or resource replacement |
| `sort_key` | `MeshBatchRef::command`, from `RenderPhaseSortComponents` | current view/frame projection |
| `source_draw_index` | `MeshDraw::mesh_pass_batch_ref` receives the enumerated current draw position | current frame deterministic ordering |
| GPU Scene instance span and its direct-draw first-instance argument | `MeshDraw::mesh_pass_batch_ref` | current GPU Scene registration |
| Visibility, relevance and culling payload | extraction and view selection | current view/frame selection |

The first row can be cached.  The remaining rows cannot be treated as cache-entry state.

## Evidence From Zircon

`mesh_draw/command_sort_input.rs` stores `depth`, `depth_bias`, render queue, camera order,
sorting-layer values, y-sort, UI z-index and a tie breaker.  Its `components()` method transfers
those values into `RenderPhaseSortComponents`.

`mesh_draw/mesh_pass_batch.rs` calls that method every time it creates a `MeshBatchRef`; it also
receives the current enumerated `source_draw_index` and transfers a current
`gpu_scene_instance_span` to the batch.  `MeshBatchRef::command` then calls
`packed_sort_key_u64(phase, sort_components, ...)` and writes the result and source index into
the current-frame `MeshDrawCommand` projection over a newly created payload.

The cached path preserves only immutable submission state:

1. `MeshDrawCommand::static_payload` returns the command's shared immutable payload.
2. `CachedMeshDrawCommands::store` accepts that `Arc<MeshDrawCommandPayload>` and rejects
   non-direct topology at the cache boundary.
3. `CachedMeshDrawCommands::lookup_status` returns an `Arc` clone, not a complete command clone.
4. The serial, parallel and pre-MeshDraw owners construct fresh visible metadata over that same
   payload identity.

The key contains stable instance identity, draw ordinal, phase, disabled-pass mask and shader
quality. Static
state comparison covers transform staticness plus geometry and material revisions.  Neither the
key nor the state includes the current sort input, source list position or GPU Scene span.  A
static cache hit can therefore retain stale ordering or stale instance addressing when those
per-frame values change without a static revision.

This is particularly important for `Opaque3d`, `AlphaMask3d`, prepass and shadow entries: they
are currently cacheable.  Transparent commands are excluded, but that exclusion does not make
opaque camera, queue and source-order data static.

## Unreal Boundary Check

Unreal 5.5 makes the boundary explicit in
`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h`:

- `FMeshDrawCommand` owns draw submission data such as pipeline state, bindings, vertex streams
  and draw parameters.
- `FVisibleMeshDrawCommand` points to that draw command while carrying the visible-command sort
  key, culling payload and per-view overrides.
- `FCachedMeshDrawCommandInfo` is cached scene metadata; the finalization path receives a sort
  key when it produces the visible command.

The relevant source comment restricts `FMeshDrawCommand` to draw-needed data and directs
InitViews payloads to `FVisibleMeshDrawCommand`.  Zircon should replicate this ownership
separation, not Unreal RHI types, allocators or global renderer state.

## Required Target ABI

The cache must own an immutable command payload, while each frame constructs a visible command
projection.

```text
CachedMeshDrawCommandPayload
  pipeline variant / pipeline key
  material, geometry and static resource handles
  static direct-draw topology
  static cache identity and static-state invalidation metadata

VisibleMeshDrawCommand
  reference or compact index to CachedMeshDrawCommandPayload
  phase sort key from current MeshBatchRef
  current source draw index
  current GPU Scene instance source / direct draw instance span
  current view culling and relevance result
```

`MeshDrawCommand` may become the static payload and a new wrapper may be introduced, or the
existing type may be split.  The chosen form must preserve one canonical replay input; it must
not create a parallel cached and uncached recorder.  The replayer resolves the payload and uses
the visible projection for ordering, instance binding and draw arguments.

For a cache hit, the required operation is a projection, not a processor rebuild:

```text
lookup static payload by (identity, phase, disabled passes, static state)
if hit:
    touch entry generation
    compose current visible projection from MeshBatchRef
    append visible projection
if miss or static invalidation:
    run the pass processor once
    split its output into static payload plus current visible projection
    store payload and append projection
```

The cache key must not grow to encode camera depth, source list position or transient GPU Scene
allocation. Encoding those values would turn every view or frame change into a miss and defeat
MD-M2. They are input to the projection instead. Shader quality and resolver configuration are
different: they select static pipeline variants. Shader quality is now part of the key and has a
focused low-to-high rebuild guard. A resolver-configuration epoch remains a researched
invalidation dimension for a later R02 slice; it is not a reason to put view-local data into the
key.

## Relationship To Existing R02 Findings

This is complementary to
`2026-08-26-static-command-cache-visibility-lifetime-research.md`:

- hidden but live static sources must touch matching entries without emitting visible commands;
- a visible cache hit must project current sort, ordering and GPU Scene data before emitting a
  command;
- a deleted source receives neither touch nor projection and retires at frame end.

The visibility-lifetime repair alone is insufficient, because keeping an entry alive without a
view-local projection would extend the lifetime of stale view data.  Conversely, a projection
does not change source-lifetime ownership.

## Test-First Regression Matrix

The cache ABI owner must add focused product-path coverage before changing implementation:

1. Complete in R02 for serial and parallel builder paths: build a cacheable static opaque command in
   generation 1, then build the same static state in
   generation 2 with a changed depth, queue/tie input and source draw index.  It must report a
   cache hit and zero rebuilds, while the emitted command has generation-2 sorting and source
   ordering values.
2. Complete in the builder and pre-MeshDraw extraction focused tests: repeat with a changed current
   GPU Scene instance span. The emitted draw arguments and instance source must reference the
   current span while the static payload remains a hit. The extraction tests also reject a cache
   hit if its current synchronized span is unavailable.
3. Complete at the command ABI and serial builder level: projections with different current sort,
   source and GPU Scene inputs share the same `Arc` payload identity. A full two-view product-path
   test remains pending.
4. Combine a hidden generation with the visibility-lifetime case: the entry remains resident,
   but no visible command is emitted until the next visible projection.
5. Change material or geometry revision.  The old payload must not be projected, must count an
   invalidation, and must rebuild exactly the affected phase.
6. Preserve existing serial/parallel parity: identical source batches must yield byte-for-byte
   equivalent ordered visible-command metadata and identical cache statistics.

Each test should inspect explicit metadata, not rely only on command count.  The current tests
that assert a second-frame hit are necessary but cannot prove that per-view fields are fresh.

## Measurement And Acceptance

This is a correctness-first structural repair.  It makes no CPU, GPU, power or asymptotic
performance claim.  Once implemented, run the existing Render-02 protocol with matched static,
material-diverse and GPU Scene scenes: 30 warm-up frames, 120 settled frames, CPU prepare timing,
GPU timing and WPR/xperf attribution.  Compare cache-hit projection cost against the prior
full-processor rebuild path at 1k, 10k and 100k static instances.

Visual acceptance must use real renderer output and a RenderDoc capture in the artifact-owning
lane under `docs/tests/runtime/render/`; no text-only result, synthetic PNG or fabricated timing
is acceptable.  Managed Cargo validation and artifact capture remain unavailable in this source
slice because the shared validation target and artifact directory are owned externally.

## Completion Status

Completed:

- Traced the current cache-hit path from batch construction through cache lookup and command
  append.
- Identified the concrete lifetime mismatch for sort key, source order and GPU Scene span.
- Checked the Unreal 5.5 ownership split and translated it into a WGPU/Rust-compatible target
  ABI.
- Defined the no-rebuild hit path, cache-key rule, source-lifetime interaction and regression
  matrix.
- Implemented current-batch projection for serial and parallel cache hits without changing cache
  keys or duplicating the processor path.
- Added focused serial and true-parallel cache-hit coverage for current sort, source index, GPU
  Scene instance source and direct draw instance arguments.
- Implemented the corresponding pre-MeshDraw cache-hit projection from explicit current sort and
  synchronized GPU Scene span inputs, with a no-span residual fallback and shared hit/miss span.
- Added focused extraction coverage for current sort and GPU Scene direct-draw data plus the
  missing-span rejection guard.
- Split the command ABI into immutable `MeshDrawCommandPayload` and current-frame
  `MeshDrawCommand` projection while preserving one canonical replay command surface.
- Changed cache entries, serial/parallel prepared work and pre-MeshDraw extraction to carry
  `Arc<MeshDrawCommandPayload>` instead of cloning complete commands.
- Kept uncached payloads inline and indirect draw arguments exclusively on current commands;
  cached payloads retain only compact direct-indexed topology. This avoids both a per-command
  payload allocation and an extra indirect-buffer `Arc` clone on dynamic paths.
- Added payload identity guards for cache hits and changed-state invalidation in addition to the
  existing current sort/source/GPU Scene projection assertions.
- Added shader quality to the static cache key across serial, parallel, visible extraction and
  hidden-entry touch paths, with a focused low-to-high variant rebuild guard.
- Audited the renderer's one-way environment-only-to-generic PBR profile transition. The scene
  uniform owner now clears static mesh command payloads exactly when that transition occurs, before
  draw construction, so commands cannot retain an environment-only variant after reflection
  providers require the generic binding contract. Added cache-clear and ordering guards.
- Replaced pre-MeshDraw per-draw phase, projected-command and deferred cache-store `Vec` staging
  with fixed three-slot arrays. Full-hit and visibility-pruned extraction no longer allocate those
  temporary collections; this is a structural result only, not a measured frame-time claim.
- Passed scoped `rustfmt --check` and `git diff --check` for the source slice. These are static
  gates only and do not count as managed compile or runtime acceptance.

Pending:

- Add a full two-view product-path identity test.
- Write successful projections directly into the generation-owned phase arena and evaluate a
  stable arena handle for cached payloads so the remaining command move and hit-path `Arc`
  reference-count clone can be removed without weakening cache-entry lifetime safety.
- Research and implement a general static-cache resolver-configuration epoch without reintroducing
  per-hit pass processing. The known one-way environment-profile transition now clears the cache,
  but future mutable resolver policies still need a typed epoch and invalidation diagnostic; the
  registry/cache owner files contain unrelated active changes and were deliberately not modified
  for that general contract.
- Run managed current-source compile/focused/product validation, then collect genuine profile,
  RenderDoc and PNG evidence.
