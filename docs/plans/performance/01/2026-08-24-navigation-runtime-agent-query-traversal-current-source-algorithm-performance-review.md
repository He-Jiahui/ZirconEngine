---
title: Navigation Runtime Agent Query Traversal Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/navigation/runtime/src
  - zircon_plugins/navigation/runtime/src/agent
  - zircon_plugins/navigation/runtime/src/components
  - zircon_plugins/navigation/runtime/src/manager
  - zircon_plugins/navigation/runtime/src/manager/traversal
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Public/NavigationOctree.h
  - dev/UnrealEngine/Engine/Source/Runtime/NavigationSystem/Private/NavMesh/RecastNavMesh.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Navigation/CrowdManager.cpp
---

# Navigation Runtime Agent Query Traversal Current-Source Algorithm Performance Review

## 1. Coverage and execution truth

The production scope is **29/29 Rust files**, **3,825 physical / 3,552 non-empty lines**, **138,059 bytes** and **7 inline tests**. The captured worktree is based on repository revision `39fe594bdaef6555277386dcc38362a575ada1c6`; its ordered `workspace-relative path + NUL + raw bytes + NUL` SHA-256 is `63ccfe993b9c09d257b7eb58c28af81f304a4b90b89fdc25ae2f039385938b72`.

| Folder | Files | Static result |
|---|---:|---|
| `runtime/src` root and `agent` | 13 | Frame extraction, fallback selection, debug capture, component decoding and writeback reviewed. |
| `runtime/src/components` | 6 | Descriptor/behavior parity reviewed; several advertised fields have no runtime implementation. |
| `runtime/src/manager` excluding bake | 5 | Global state, query, statistics and tick ownership reviewed. |
| `runtime/src/manager/traversal` | 5 | Link selection, capacity, interpolation, cleanup and transform ownership reviewed. |

Existing concurrent formatting/import edits and the `context.tick().delta_seconds()` correction were treated as current source and preserved. No Cargo execution is claimed.

## 2. Structural performance findings

### P0: frame work is a full-World JSON extraction under exclusive ownership

`agent.rs:462` walks `world.node_records()` every frame and dynamically deserializes every navigation agent. The legacy path separately calls `collect_runtime_obstacles` at `manager/tick.rs:39`; links and bridges use additional whole-World passes in `off_mesh_connections.rs:39` and `:78`, and final statistics count them again. Each pass creates fresh vectors/maps/sets and repeats component JSON decoding.

The runtime plugin holds mutable World ownership across the entire navigation tick, overlay construction and event publication (`plugin.rs:153-167`). This is not a query-driven ECS phase and cannot be scheduled in parallel with systems that touch unrelated components. Invalid component JSON defaults silently, so bad data can also keep re-entering the hot path instead of failing once at admission.

### P0: query authority rebuilds native topology instead of reusing a world generation

`manager.rs:74-85` deep-clones the selected `NavMeshAsset`; `manager/query.rs` passes that owned clone to the backend. The native wrapper then creates `DetourQuery::from_asset` for every find/sample/raycast (`native/src/detour.rs:20-39`). The C++ constructor re-quantizes vertices, reconstructs polygon arrays and builds adjacency with nested polygon/edge scans (`detour_query.cpp:366-439`) before allocating and initializing a new `dtNavMesh` and `dtNavMeshQuery` (`:527-658`).

Therefore a nominal Detour query includes asset clone O(V+I+P+L), topology construction at least O(P^2 * E^2) in the current neighbor loop, native allocation, query, and destruction. The fallback is not a bounded substitute: `fallback_query/graph.rs:41-67` builds another O(P^2) graph and `:128-164` uses a linear-minimum Dijkstra, O(P^2+E), per request. Raycast takes 32 samples and may rebuild that graph when the sampled polygon changes.

### P0: fallback changes product semantics and movement ownership

Any runtime obstacle, persisted obstacle world, or loaded asset containing off-mesh links clears all persistent crowds and routes every agent through the legacy path. A native unsupported/create/query failure silently falls back to the Rust graph (`native/src/lib.rs:47-60`, `:82-114`) rather than exposing capability loss. With no loaded navmesh, the legacy tick uses the destination as its path target and moves directly; navigation absence can therefore become collision-blind movement.

Legacy avoidance is O(A * (A + O)); traversal interpolates positions and writes scene transforms directly. This bypasses the character movement, physics and replication authority except when an agent explicitly opts into desired-velocity writeback. Navigation must produce path/corridor/steering intent, not become a second transform simulation owner.

### P1: global mutex and raw handles prevent lifecycle-safe concurrency

All worlds and sessions share one `Arc<Mutex<NavigationRuntimeState>>` (`manager.rs:34`) keyed by raw mesh handles and entity IDs. Runtime obstacle synchronization holds this manager lock while updating TileCache and finding a path. Settings replacement clears task/state maps but provides no cancellation/join receipt. There is no unload/streaming API or world/session/replacement generation on query, crowd, obstacle or traversal state.

The lock serializes otherwise independent worlds, path queries, crowds, debug capture and bake publication. Poison recovery converts invariant failure into continued mutation. This is an ownership flaw, not a mutex-tuning problem.

### P1: debug mode multiplies query cost and payload construction

When debug capture is enabled, each agent performs an additional `find_path` (`agent.rs:196`) and stores a full path DTO. The runtime then materializes all loaded navigation triangles/links and clones the tick report for the mirrored event. Debug work is demand-gated, but it is not selected-agent, tile, bounds, frequency or byte-budget gated.

## 3. Unreal source constraints

Unreal is the primary structural reference:

- `NavigationOctree.h:111-141` stores navigation-relevant elements in a spatial octree with bounded leaf semantics; `:186-226` updates/removes nodes by stable element IDs. Zircon's repeated whole-World dynamic JSON scans do not provide equivalent indexed ownership.
- `RecastNavMesh.cpp:4378-4437` routes path requests through the persistent `RecastNavMeshImpl`; it does not rebuild topology from a serialized DTO for every query. `:1951-1967` provides batch-query scope used by crowd processing.
- `CrowdManager.cpp:232-316` caches active agents once, begins one batch query, executes separately instrumented corridor/path/proximity/steering/avoidance/collision/movement phases, and finishes the batch. Debug capture is separately gated by selected actors and console variables (`:43-83`).

Zircon should adopt the lifetime and phase ownership: one world-scoped navigation data generation, persistent query/crowd state, indexed changed inputs, bounded phase work and movement-intent commit. It should not copy Unreal's UObject surface.

## 4. Dependency-ordered optimization plan

### M0: make capability failure explicit

Define typed backend capability/status for Detour, TileCache and fallback. Do not silently switch algorithms after native construction or query errors. Fallback may be an explicit test/development provider, never an invisible product behavior. Block automatic movement when no admitted navigation generation exists.

### M1: establish world-scoped navigation ownership

Replace the process-global state with a world/session-owned navigation service. Qualify mesh, crowd, obstacle and traversal handles by owner plus generation; add unload, replacement, shutdown and cancellation receipts. Keep immutable asset data separate from persistent compiled native query data.

### M2: replace scans with typed changed-component extraction

Use cached ECS queries/change generations for agents, surfaces, obstacles, modifiers and links. Parse typed components at mutation/admission boundaries. Extract changed inputs briefly, run native work without mutable World ownership, then commit desired velocity/events through an explicit command buffer.

### M3: use one persistent query/crowd generation

Compile each admitted navmesh generation once into tiled Detour data and reusable query pools. Batch queries by mesh/filter, retain corridor state, and budget repaths by measured cost. Obstacles update the matching TileCache generation incrementally without disabling all crowds. Manual/automatic links must be filter policy on the same topology rather than trigger a cloned asset/backend rebuild.

### M4: converge movement ownership

Navigation publishes corridor, desired velocity, arrival and traversal intent. Character movement/physics owns transforms and replication. Off-mesh traversal becomes a movement-mode request with capacity/state events, not direct scene interpolation.

### M5: bound diagnostics

Add selected-agent/bounds/tile filters, update frequency, byte/primitive limits and retained generation reuse. Record scan, query-build, native query, crowd phases, lock wait/hold, repath deferral, fallback reason, debug generation and payload bytes.

### M6: qualify scale and product behavior

Measure worlds `1/4`, agents `0/1/100/1k/10k`, obstacles `0/100/1k`, polygons `10/1k/100k/1m`, path lengths, filters and debug modes. Report p50/p95/p99, allocations/bytes, World ownership duration, mutex wait, query compile count, path/crowd phase time, CPU, wakeups, RSS and power. Prove stable frames do zero component parsing and zero query compilation.

## 5. Acceptance gates

1. One navmesh generation is compiled once and reused by all queries until replacement/unload.
2. Stable frames perform zero whole-World scans and zero navigation component JSON deserialization.
3. Native failure is observable and never silently changes query algorithm or movement semantics.
4. Navigation does not directly own character transforms; commit uses the movement authority.
5. All state is world/session/generation qualified and cancellation/shutdown leaves no detached work.
6. Agent and polygon scale curves are no worse than expected Detour/crowd complexity; no O(P^2) per-query construction remains.
7. A current-source executable passes WPR/ETW timing/power capture before protected-ledger promotion.

## 6. Validation status

- Per-production-Rust-file static review: **29/29 complete** for the captured fingerprint.
- Native C++ query/crowd/TileCache source: reviewed as supporting algorithm evidence; not counted in the Rust ledger.
- Cargo/tests: **pending** because the managed Windows validation session is not executable; raw Cargo was not substituted.
- WPR/ETW: **pending** because no launchable current-source executable exists.
- RenderDoc: **not applicable to this CPU/navigation phase**; use only after a current-source viewport debug rendering path exists.
- Protected ledgers, milestone commit and WeCom completion: unchanged/pending until dynamic acceptance.
