---
title: Plugin Physics Current Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/physics
status: static_complete_local_snapshot_fix_and_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Private/Chaos/Framework/PhysicsSolverBase.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PhysicsEngine/Experimental/PhysScene_Chaos.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PhysicsEngine/PhysicsQueryHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PhysicsEngine/Experimental/ChaosCooking.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PhysicsEngine/Experimental/ChaosDerivedData.cpp
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics/mod.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
---

# Plugin Physics Current Source Performance Review

## 1. Coverage

The current Rust surface is **86/86 files**, **13,359 physical / 12,377 non-empty lines**, **467,002 bytes**, **91 tests** and **1 ignored performance test**. Its workspace-relative `path + LF + raw bytes` SHA-256 after the local snapshot fix is `9845a831ddda70248a36099f5433773713651dbec496c5bd5956e247f7664aac`.

Every Dist, Editor and Runtime Rust file was indexed and parsed. Entry/registration, settings and clocks, builtin/Jolt backends, body/shape/constraint handles, collision/query/trigger algorithms, scene synchronization, skeletal runtime, editor authoring/overlay and all unit/integration/performance tests were reviewed. The eight files already modified at review start contained rustfmt import-order changes and were preserved as the current baseline.

| Area | Rust files | Physical lines | Current execution truth |
|---|---:|---:|---|
| Dist | 1 | 98 | Publishes a native registration manifest but does not materialize the manager, backend, systems or query interface. |
| Editor | 7 | 483 | Registers authoring descriptors and pure profile/overlay builders; the overlay builder has no product caller. |
| Runtime | 78 | 12,778 | Owns two clocks, a scene mirror, builtin collision/query code, optional Jolt FFI, Rust-side constraints and ragdoll projection. |

## 2. Implemented local optimization

`DefaultPhysicsManager` now exposes a crate-local shared `Arc<PhysicsWorldSyncState>` snapshot. Internal queries and `physics.sync_to_scene` use that snapshot directly; the public `PhysicsManager::synchronized_world` value-return contract remains compatible and performs a deep clone only for external callers that request owned state.

This removes one unconditional deep clone of `bodies + colliders + joints + materials` from every fixed post-step. The operation changes from O(B+C+J+M) element/nested-payload copies and vector allocations to O(1) `Arc` reference-count work. The existing pointer-identity test now covers the runtime-facing accessor. This is a bounded fix only: scene extraction, Jolt reconciliation, event scans and body writeback remain structurally expensive.

## 3. Structural performance findings

### P0: the default product does not contain an executable physics product

The package correctly labels all capabilities `partial/experimental`, but the product graph is still open. Runtime has no default backend feature; without `backend-jolt`, the manager defaults to `unconfigured + disabled`. `first_party_runtime_catalog` has no Physics provider branch. Native Dist reports the registration manifest while omitting the manager, systems, backend and query behavior supplied by the source-linked plugin.

An MVP profile can therefore advertise or package Physics without resolving one equivalent source/native product. Before performance qualification, one `ResolvedPhysicsProviderV1` must close backend, scene systems, queries, events, cook artifacts, settings and lifecycle for every supported packaging form. Unsupported native behavior must fail admission rather than return a successful manifest.

### P0: two fixed-step authorities expose contradictory settings

`PhysicsManager::plan_world_step` owns an accumulator using `fixed_hz` and `max_substeps`. The actual registered `FixedUpdate` system bypasses it and calls `fixed_update_step_plan`, which always returns one step with the scheduler-provided `delta_seconds`. A product test explicitly configures the Runtime scheduler to 64 Hz, Physics to 60 Hz, and accepts one 1/64-second physics step.

The scheduler should own fixed-time accumulation once. Either Physics consumes the scheduler's canonical fixed clock and removes its second product clock, or the scheduler obtains its fixed-step policy from one physics time-domain configuration. Keeping both makes settings misleading and breaks deterministic catch-up/replay reasoning.

### P0: builtin collision and trigger generation are duplicate O(N squared) scans

`compute_contact_events` visits every unordered collider pair. `collect_current_trigger_pairs` independently visits every unordered collider pair again, repeating filtering and overlap work. There is no broadphase, dirty pair cache, active island or subscription gate.

The two scans inspect `N(N-1)` candidate pairs per step: **9,900** at 100 colliders, **999,000** at 1,000 and **99,990,000** at 10,000. At 60 Hz those become approximately **0.594 million**, **59.94 million** and **5.9994 billion** pair checks per second before narrow-phase math and event allocation. This is a main-thread scale failure, not an iterator-level optimization target.

### P0: Jolt multithreading is followed by serial full-scene work

Jolt creates a native thread-pool job system and uses its broadphase for native simulation. The manager nevertheless holds one `jolt_worlds` mutex across full reconciliation and native step. Every tick rebuilds scene sync vectors, sanitization sets/maps, a cloned collider `HashMap`, a cloned desired-body `BTreeMap`, stale lists, command translations and active-state maps.

After native simulation, `read_active_states` invokes `refresh_events`, which clones every backend body/collider into another world snapshot and reruns the same two Rust O(N squared) pair scans. Jolt's own query trait methods for ray cast, shape cast and overlap are empty; product queries instead linearly scan the mirrored collider vector. Native parallel simulation is therefore bounded by a serial main-thread mirror and event tail.

### P0: the Jolt adapter does not use native constraints

`JoltPhysicsBackend::create_constraint` only stores `ConstraintDesc` in a Rust `HandlePool`. After every native step, `project_constraints` reads all native body states, applies custom Rust projection, and writes touched bodies back to Jolt. This duplicates solver ownership, loses native island/constraint scheduling and makes behavior depend on a single post-step projection pass rather than the selected backend's iterative solver.

The backend contract must create/destroy/update native constraint handles and receive break/violation outputs from that backend. Builtin fallback may retain a deliberately limited solver, but it must not be layered on top of Jolt as the production constraint implementation.

### P1: scene synchronization is full-snapshot rather than dirty-state driven

Every fixed step scans all world node records, resolves world transforms, clones collider shapes/materials/joints and builds a complete `PhysicsWorldSyncState`. Sanitization allocates entity sets and a material map. Jolt then clones those collections again for reconciliation. Fixed post-update walks every synchronized body and performs transform/component comparisons even when only a small active set changed.

The implemented `Arc` fix removes one snapshot clone, not the full-snapshot design. The target is stable physics object IDs plus dirty component/property commands into the backend, an active/changed body receipt out, and scene writeback limited to changed dynamic bodies. Multi-world work must use per-world ownership/jobs instead of one global world-map mutex.

### P1: mesh and height-field construction expands into per-triangle shapes

Jolt triangle meshes and height fields are converted to triangle arrays, then each triangle is created as an individual native shape and all shapes are placed in a static compound. A `W x D` height field creates `2(W-1)(D-1)` children: **130,050** at 256x256 and **2,093,058** at 1024x1024. This is unacceptable startup memory, allocation and cook cost.

Physics assets need an offline/versioned cook artifact keyed by source, scale policy, backend/version and platform. The runtime should load one backend-native mesh/height-field acceleration object, not rebuild millions of child shapes. Cook output, timing, bytes and cache hit/miss must use the shared non-C derived-data root.

### P1: ragdoll update compounds per-character linear scans

Each physics tick collects all bound bodies and clones bone-path strings. Topological ordering repeatedly searches and removes from a `Vec`, worst-case O(B squared). Target resolution scans pose rows per bone. Simulated pose feed resolves each joint body by scanning the body vector and builds temporary `(skeleton, String)` maps. Across many characters this compounds with the full-world physics mirror.

Ragdoll profiles should compile once into stable bone/body indices, parent order, offsets and native constraint handles. Per-frame animation-to-physics and physics-to-pose exchange should use dense indexed buffers, dirty/active masks and character-granularity parallel jobs. String-path lookup belongs to authoring/compile time.

### P1: telemetry cannot attribute the observed cost

Only `physics.step.duration_ms` is recorded around the whole FixedUpdate system. There are no phase timings or counters for extraction, dirty counts, reconciliation, lock wait, native step, constraint solve, event candidate pairs, query candidates, active-state readback, scene writeback, allocations or backend worker utilization. The ignored benchmark measures Arc/filter/query-mode microcases but not the product frame.

Add stable per-world/backend receipts and WPR-visible spans before claiming a bottleneck is removed. Editor overlay is not a current frame bottleneck because it has no product caller; when wired, it must consume an immutable generation and rebuild only on physics/debug invalidation.

## 4. Reference-engine constraints

Unreal is the primary architecture constraint:

- `PhysicsSolverBase.cpp` owns fixed-dt accumulation, max substeps and pending task limits in one solver advance path. Non-single-thread mode dispatches dependency-ordered solver tasks instead of holding the game thread through all simulation work.
- `PhysicsSolverBase.cpp` queues pending spatial operations by stable particle identity. `PhysScene_Chaos.cpp::OnSyncBodies` pulls only dirty proxies, skips unchanged transforms, updates the acceleration structure incrementally and emits scoped cycle statistics.
- `PhysicsQueryHandler.cpp` routes ray/sweep/overlap to the Chaos query interface and provides queued async variants. Queries do not rebuild or linearly scan a game-thread scene mirror.
- `PhysScene_Chaos.cpp` registers collision event handlers and component subscriptions, and parallelizes deferred skeletal-mesh kinematic updates. Event and ragdoll work has explicit ownership rather than a universal pair scan.
- `ChaosCooking.cpp` builds one triangle-mesh implicit object from indexed mesh data; `ChaosDerivedData.cpp` and `BodySetup.cpp` key/cache cooked geometry and report cook resource usage.

Secondary checks agree with those boundaries. Fyrox uses Rapier's broadphase query pipeline, `need_sync_model/try_sync_model` dirty flags and equality checks before transform writes. Bevy accumulates elapsed time once in `Time<Fixed>` and runs `FixedMain` zero or more times; systems do not establish a competing fixed clock.

## 5. Dependency-ordered optimization plan

### M0: close MVP product and time-domain truth

Select one production physics provider for each MVP target and make source/native behavior equivalent. Wire Physics into the resolved first-party profile only when its required backend and systems materialize. Make unsupported forms fail closed.

Choose one fixed-step authority. Publish the canonical dt, accumulated remainder, dropped/capped steps and interpolation generation from Runtime scheduling to Physics/replay. Delete or demote the unused duplicate clock/settings surface.

### M1: replace snapshots with a persistent physics scene bridge

Create stable entity-to-body/collider/constraint handles and component change generations. Feed create/update/remove/command batches only for dirty objects. Keep backend worlds per Runtime world with bounded command/result queues and explicit shutdown/replacement epochs.

Return an immutable step receipt containing changed/active bodies, contacts/triggers, backend generation and timing counters. Apply only changed transforms/components in FixedPostUpdate. Retain shared snapshots only for debug/editor inspection, not as the simulation transport.

### M2: make the selected backend own physics algorithms

For Jolt, implement native constraints, native collision listeners, layer/mask filtering and native ray/sweep/overlap. Remove Rust post-projection and O(N squared) event reconstruction. Use backend broadphase/narrowphase candidate/event data and subscription filters.

For builtin, define it as a bounded deterministic fallback with an incremental broadphase such as dynamic AABB tree or sweep-and-prune, one shared candidate pass, active islands and explicit supported shape/constraint limits. It must not silently impersonate a production-scale backend.

### M3: compile physics assets and ragdolls

Move mesh, convex, height-field and compound construction to a cancellable cook pipeline. Store versioned backend-native artifacts in the shared non-C DDC and load them without per-triangle shape creation. Publish source/cooked bytes, cook p50/p95, cache hit/miss and runtime load time.

Compile ragdoll profiles into dense bone indices, parent order, body/constraint descriptors and binding tables. Execute character batches through the Runtime scheduler, with deterministic single-thread qualification and bounded parallel production mode.

### M4: instrument and qualify current-source product scenes

Add phase spans/counters for extraction, dirty commands, lock wait, native simulation, constraint/event/query work, active readback and scene application. Record object counts, candidate pairs, contacts, triggers, awake/dirty bodies, allocations, queue depth and worker utilization.

After a managed Windows current-source executable exists, run fixed 100/1,000/10,000-body and query/event/ragdoll scenes with warmup and fixed hardware/power settings. Capture CPU p50/p95/p99, fixed-step misses, frame latency, threads/waits, allocations/RSS and energy/frame with WPR/ETW. RenderDoc is required only for matching visible debug/scene output and GPU work; it is not a CPU physics profiler.

## 6. Acceptance gates

1. One resolved Physics product closes source/native/backend/profile parity or fails admission with a typed reason.
2. Runtime scheduling is the single fixed-time authority; settings, replay and tests observe the same dt/substep/drop receipt.
3. Stable frames perform zero full-world physics rebuild and zero deep world snapshot clone. Work scales with dirty/active objects plus broadphase candidates.
4. Jolt production paths create native constraints, consume native collision/query facilities and perform zero Rust all-pairs event scans.
5. Builtin candidate generation is subquadratic for sparse scenes and shares one candidate set between contact and trigger processing.
6. A 1024x1024 height field loads one cooked backend artifact, not 2,093,058 child shapes; runtime performs no source mesh triangulation.
7. Ragdoll frame work uses compiled indices and bounded buffers with no bone-path string search/allocation in steady state.
8. Phase telemetry and WPR evidence show no unbounded main-thread/worker queue growth and report CPU p50/p95/p99 plus energy/frame for fixed qualification scenes.
9. Managed tests, current-source executable evidence and matched product correctness pass before protected-ledger promotion, milestone commit or WeCom completion notification.

## 7. Validation status

- Static per-Rust-file review: **86/86 complete** for the captured source fingerprint.
- `rustfmt --check --config skip_children=true`: the four behaviorally affected files pass; 84/86 module files pass overall, while two untouched Editor files have formatting-only diffs.
- Local snapshot change: **implemented** and covered by the existing Arc pointer-identity unit test at source level.
- `git diff --check` for the four affected source files: **pass**.
- Product closure: **failed statically** because Physics is absent from the first-party Runtime catalog, default Runtime has no active backend, and native Dist does not materialize source behavior.
- Algorithm scale: **failed statically** for duplicate all-pairs events, full-scene synchronization, Rust-projected Jolt constraints and per-triangle mesh construction.
- Cargo/test execution: **pending** because the managed Windows validation session is not executable; no raw Cargo lane was substituted.
- Current-source executable, WPR/ETW timing/power and visible product qualification: **pending**. No current-source executable exists, so WPR and RenderDoc were not run.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.

