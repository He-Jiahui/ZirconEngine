---
related_code:
  - zircon_runtime/src/scene/ecs
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/render_extract
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SceneComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityQuery.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassCommandBuffer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassObserverManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
tests:
  - seven 2026-08-14 current-source ECS/derived/extract reviews reconciled as one frame pipeline
  - 27 current cross-owner production anchors, 8992 lines and 16 inline tests read against current worktree
  - current anchor manifest sha256 cb5ef6484c1e3188250a7d329710965b4fa1955fea3311ee7831ac93fbc7b95d
  - rustfmt aggregate failed only on foreign-dirty editor_state_render and camera_loop formatting drift
  - managed Windows product and focused Cargo gates remain blocked; zero current product traces
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# World/ECS/Frame Extract current architecture review (2026-08-15)

## Scope and current-source reconciliation

This review does not replace the per-file manifests from 2026-08-14. It composes the completed
schedule, query, deferred command, archetype/storage, observer/event, derived-state and render-extract
reviews into one frame-level architecture and rechecks 27 cross-owner product anchors against the
current worktree. The anchor set contains 8,992 lines and 16 inline tests; its path-and-content
fingerprint is `cb5ef6484c1e3188250a7d329710965b4fa1955fea3311ee7831ac93fbc7b95d`.

The source is heavily foreign-dirty under active Runtime08 and Editor work. This review therefore
does not edit Rust. A local change to one container or cache would compete with active ownership and,
more importantly, would preserve the wrong frame contract described below.

## Corrections to stale findings

The current implementation has useful repairs that the hard cut must retain:

- `Schedule` owns an `Arc<SceneScheduleStagePlan>` and rebuilds it only after registration changes.
  The old claim that every stable frame recompiles all stage ordering is no longer correct.
- Dense table components have one resident body owner, query plans bind table column slots, and new
  archetypes are appended by generation. The old double-storage and full-query-cache rebuild claims
  are stale.
- Deferred commands pack small payloads, merge worker buffers deterministically and preflight an
  entire barrier before the first target is published. The old one-`Box`-per-command model is stale.
- Hierarchy parent/child order is persistent and traversal is iterative. Stable frames skip clean
  derived domains, so recursion and repeated hierarchy-index construction are no longer the root.
- Physics, animation and script frame state use separate synchronization and sealed snapshots. The
  former single runtime-state mutex is no longer present.

These are local prerequisites, not proof that the system is now incremental end to end.

## P0 architecture findings

### 1. The compiled schedule is not the product execution authority

`SceneScheduleStagePlan` freezes stage order, but `SceneScheduleRunner` still builds a fresh
`worker_batch`, checks conflicts against current batch members, creates `DeferredSystemKey` values,
takes systems out of `World`, and restores them after execution. The registry lookup is a linear scan
followed by `Vec::remove`; restore performs ordered insertion. The separate
`ScheduleParallelExecutor` is not the product scene owner. Its existence therefore does not prove
that F2 data-bearing ECS work uses the TaskGraph.

The worker-safe contract remains worldless. `Query`, resources, events, messages and removed
components do not expose validated chunk/resource views to workers. Real data systems still execute
inside `LevelSystem::with_world_mut`, while only local state and deferred-command producers can use
the worker lane. The result is a cached description wrapped around a string-addressed, move-out,
main-thread execution model.

### 2. There is no single World commit artifact

The deferred command barrier preflights globally but publishes target by target. Lifecycle callbacks
are expanded per entity/component, removed-component history has no single frame update owner, and
derived invalidation is five booleans rather than changed entity/archetype/resource ranges. A local
transform therefore sets whole domains dirty; active and matrix propagation traverse all roots and
write every row; `NodeCache` then clears and clones a wide `SceneNode` projection for all named
entities.

These paths do not consume one shared commit result. Command publication, lifecycle, observer/event,
derived state, editor inspection and render extraction each rediscover affected work. Adding another
cache beside them would create a sixth invalidation authority.

### 3. Storage and query layout cannot yet support chunk execution

The default system query preserves global entity order with a `HashMap`, a global `BTreeMap`, one
`BTreeMap` per archetype and a `BinaryHeap` merge. Stable generations still scan every cached
archetype to refresh membership counts. Per-row fetch materializes component locations and repeatedly
matches type identities. Mutable and combination paths first collect candidates and create short-lived
location buffers.

Full archetype identity still includes sparse membership while every full signature owns its own
dense table. Sparse-only changes can therefore move the dense row. A dense row move converts each
column value into an opaque box and a tree entry before writing it back. This is incompatible with a
range-based command buffer, chunk query view and worker partition sharing the same physical identity.

`combination_count` also returns `usize::MAX` on intermediate overflow while iterator termination is
driven by that count. Once the final index tuple is reached, advance can fail without ending the
iterator, repeating the last tuple for an effectively unbounded duration. PERF-MVP-606 remains a
correctness gate, but it is not the architectural owner of this milestone.

### 4. Frame extraction is a clone adapter, not a generation boundary

There are three unequal product paths:

- `LevelSystem` owns animation state but holds the World mutex across prepared extraction.
- Dynamic runtime builds its cache key under one World lock, misses into
  `World::to_render_frame_extract`, clones the World, and deep-clones the complete extract on both
  cache population and cache hit. Its key omits animation generation and uses world handle zero.
- Editor calls `World::build_viewport_render_packet`, which clones the World, then adapts through a
  snapshot path that omits animation and other sideband data.

Scene-global mesh/light/particle/volume output is already projected with the first selected camera.
The graphics camera loop changes view fields later, so additional cameras do not receive a genuinely
camera-specific LOD, transparency order, volume resolution or visibility input. This is both a
performance and correctness defect.

## Unreal source basis

The target below copies behavior boundaries, not C++ types:

- `TickTaskManager.cpp:280-376,1098-1130` uses stable tick-function identity, explicit desired thread,
  prerequisite tracking and frame-end reset that retains batch allocation.
- `MassEntityQuery.cpp:138-228,258-396,573-690` uses an archetype data version, compiles requirement
  mappings for new archetypes, iterates archetype chunks/ranges and gives the same chunk job to the
  parallel executor.
- `MassArchetypeData.cpp:1672-1785,2028-2110` prepares destination spans and moves/removes ranges;
  sparse add/remove operates on existing entity ranges without forcing a dense-body row move.
- `MassCommandBuffer.cpp:109-225` owns reusable commands by operation group, runs the whole command
  batch, then resets capacity. `MassObserverManager.cpp:431-506` consumes entity collections and
  prevents the same processor from executing more than once per batch.
- `SceneComponent.cpp:760-826,968-1026,2909-2953` compares old/new transforms, propagates only when
  required and walks attached children. `RendererScene.cpp:1570-1709` queues changed primitive
  transforms, skips redundant updates and batches render-thread publication.

These sources support versioned plans, chunk/range work, changed-value gates and persistent render
scene publication. They do not justify copying Unreal object ownership, constants or worker counts.

## Required hard-cut architecture

`zircon_runtime::scene` must own one dependency-ordered generation chain:

| Artifact | Owner and contents | Forbidden parallel truth |
|---|---|---|
| `FrameScheduleGeneration` | Runtime03; dense system slots, access, affinity, dependencies, barriers and command lanes | per-frame String keys, registry remove/insert and product-only alternate executor |
| `WorldStorageGeneration` | Runtime08; entity slots, full membership, table schema, chunks, component bindings and query plans | sparse membership owning a duplicate dense table or editor runtime storage |
| `WorldCommitGeneration` | Runtime03/08; commit id plus added/changed/removed entity, component, resource and hierarchy ranges | setter-local callback publication, five global dirty booleans and consumer-specific rescans |
| `SceneRenderGeneration` | Runtime07; camera-neutral persistent primitive/light/particle/volume/camera data updated from one commit | World clone, NodeCache as render dependency and first-camera scene DTO |
| `FrameExtractGeneration` | Runtime07/10; scene handle plus per-view projections, animation/resource handles and sideband | editor/runtime/snapshot product adapters and deep-cloned cache values |

Implementation order is mandatory:

1. Runtime08 freezes `WorldStorageGeneration`: separate full membership from table schema, introduce
   chunk/range identity, compiled move/init/drop plans and fast versus explicitly ordered query APIs.
2. Runtime03 compiles `FrameScheduleGeneration` against those chunk/resource views; Runtime11 executes
   dense slots through the shared TaskGraph. Stable frames do not remove systems or rebuild keys.
3. Runtime03/08 make a deferred stage publish exactly one `WorldCommitGeneration`; lifecycle,
   removed-component windows, observers and derived state consume its ranges. Publication is atomic.
4. Runtime07 consumes the same commit to update `SceneRenderGeneration`; derived transforms use
   minimized dirty roots and changed-value writes. Render no longer demands `NodeCache` freshness.
5. Runtime10 and Editor05 switch all product consumers to `FrameExtractGeneration`; Render04/07/12/17
   build per-view work from the camera-neutral scene. Old World/snapshot adapters are deleted in the
   same milestone, with no compatibility shim.

PERF-MVP-604..620 remain the detailed storage/query/command/event/extract gates.
PERF-MVP-632 is the cross-owner acceptance task that prevents those repairs from shipping as separate
generations or dual paths.

## Complexity and dynamic acceptance

Run entities 1/1k/100k, systems 1/16/256/1k, components 1/8/31, archetypes/chunks 1/8/256/4k,
dirty 0/1/10/100%, views 1/2/8 and editor/runtime mode. Separate plan-build, mutation, commit,
derived, scene publication and per-view windows. Record slot/hash/string work, chunk/range visits,
row moves, boxes/tree operations, allocations/clone bytes, lifecycle/observer calls, World lock
wait/hold/acquires, main/worker wall, queue age, p50/p95/p99, RSS, context switches, ReadyThread,
GPU upload/draw/dispatch/timestamps and energy.

Hard gates are: stable schedule build/key/registry movement = 0; stable World full clone and full DTO
clone = 0; one stage barrier publishes zero or one commit; no-observer event allocation = 0; one local
change visits/writes near its affected ranges; render scene work does not multiply by view count;
non-first-camera output is correct; editor and runtime observe the same World/scene generations.

The managed product build is still blocked by foreign current-source compile errors and there is no
current runnable binary. WPR/xperf and RenderDoc therefore cannot produce valid before/after data yet.
This module remains in `pending.md`, does not enter `review.md`, and is not a commit or WeCom milestone.
