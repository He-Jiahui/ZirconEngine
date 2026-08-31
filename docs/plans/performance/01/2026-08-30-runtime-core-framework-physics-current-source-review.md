---
related_code:
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src/manager
  - zircon_plugins/physics/runtime/src/backend/builtin
  - zircon_plugins/physics/runtime/src/runtime_system.rs
  - zircon_runtime/src/scene/world/query.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/Chaos/ChaosScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/BodyInstanceCore.h
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/Chaos/ChaosPhysicalMaterial.h
  - dev/UnrealEngine/Engine/Source/Runtime/PhysicsCore/Public/Chaos/SceneQueryCommonParams.h
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
status: static_complete_dynamic_pending
---

# Runtime core physics framework current-source review (2026-08-30)

## Scope and status

`zircon_runtime/src/core/framework/physics/**` was read file by file: 32 Rust files, 1,099 physical lines, 990 nonempty lines, 34,180 bytes and 9 inline tests. The sorted raw-content aggregate SHA256 is `64a5cbc9d8579ae47676d88a4d106c1ee4ef65d7856e552d1f7fec78ec1ad309`. `skeletal_pose.rs` is foreign modified and was preserved. Direct rustfmt was checked for all files: 25/32 pass and 7 fail on existing import/assert formatting.

The directory defines the physics manager contract, sync DTOs, query DTOs, settings and skeletal pose data. It has no WGPU work or frame loop. The dynamic owner is `zircon_plugins/physics/runtime/**`, whose implementation files are foreign modified in the current worktree; this record therefore remains static-complete/dynamic-pending and stays in the broad `zircon_runtime/src/core/**` and physics plugin pending rows.

## Static findings

`PhysicsManager` exposes settings, backend status, world-step planning, world synchronization, synchronized-world snapshots, ray/shape queries and contact/trigger drains. Most values are fixed or move-oriented DTOs, but the public `synchronized_world` path clones the complete `Arc<PhysicsWorldSyncState>` payload into a new DTO. Query methods return owned hit vectors, so scale and result-byte limits are part of the product contract rather than this leaf DTO layer.

The active plugin path rebuilds the full world projection on every physics tick. `build_world_sync_state` first calls `World::node_records`, which materializes and sorts a node vector, scans that vector again for capacities, then scans it again to resolve transforms and build bodies, colliders, joints and materials. Collider material locators allocate owned identifiers and compound shapes can allocate nested payloads. `sanitize_world_sync_state` then creates uniqueness sets and a material lookup map and repeats validation/deduplication over the already-produced sync state.

Builtin and Jolt service ticks both call the full projection even when no world generation changed. Jolt synchronization builds clone-heavy collider/body/material maps, clones desired rows into ordered maps, computes stale rows, and clones unchanged entity state back into retained records. Constraint cleanup uses a linear stale-membership check inside the constraint scan. Active-state reads create another handle map and clone rows into the public sync snapshot. The Jolt world mutex remains held across backend synchronization, so backend work can extend the lock's owner-wait scope.

These are source-derived algorithm and ownership findings, not measured frame percentages. The implementation files are currently foreign-owned, so no local optimization was attempted. A changed-generation path must also preserve exact invalid-candidate behavior and contact/query semantics before replacing the current full rebuild.

## Reference-engine constraint

Unreal keeps body mass/start-awake and dirty-mass policy in `FBodyInstanceCore`, and groups friction, restitution and sleeping thresholds in a physical-material contract. `ChaosScene` exposes explicit actor updates, acceleration-structure updates and solver waits, while `SceneQueryCommonParams` carries typed query filter data. This supports dirty incremental synchronization, explicit task ownership and typed query inputs; it does not establish an identical Rust ABI or backend implementation.

## Architecture handoff

- M0: add scale counters for node projection, capacity/actual scans, validation visits, body/collider/joint/material rows, cloned bytes, query hits, stale checks and backend lock hold time; cover 0/1/1k bodies and unchanged/changed worlds.
- M1: compile one immutable `PhysicsWorldGeneration` from world/component changes. Store borrowed or Arc-backed typed rows and a dirty set; unchanged worlds perform no full node projection or DTO clone.
- M2: validate and budget a `PhysicsSyncProposal` before backend mutation, including nested shape bytes, material identifiers, query/result caps and current/candidate/pending rows. Invalid candidates publish no partial world state.
- M3: apply one accepted generation transactionally to Builtin/Jolt. Reuse exact entity-indexed backend records, replace linear stale membership with generation/index membership, and do not hold a global world mutex across backend waits.
- M4: qualify synchronized snapshots, contacts, triggers and query results by world/provider/submission generation. Public exports use Arc-backed or bounded views; full owned copies are explicit sideband work.

## Acceptance gates

Dynamic acceptance requires current-source Cargo and scale evidence for world projection, unchanged/changed sync, backend preparation, query filtering, contact/trigger drains and Jolt lock duration. Hard gates are one world-generation authority, zero unchanged-tick full rebuilds or payload clones, all-or-nothing invalid sync, bounded nested/query bytes, generation-qualified feedback, and diagnostics matching actual visits, copies and backend waits. No source micro-fix is justified while the plugin implementation and its existing physics contract work remain foreign-owned.
