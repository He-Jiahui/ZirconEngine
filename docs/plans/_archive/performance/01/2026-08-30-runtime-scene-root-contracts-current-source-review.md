---
related_code:
  - zircon_runtime/src/core/framework/scene/level_manager_error.rs
  - zircon_runtime/src/core/framework/scene/level_summary.rs
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/module_identity.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/core/framework/scene/resource.rs
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/core/framework/scene/world_handle.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/scene/module/level_manager_project_io.rs
  - zircon_runtime/src/scene/module/scene_artifact_io.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/script/vm/reflection/catalog.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
status: static_complete_dynamic_pending
---

# Runtime scene root contracts current-source review (2026-08-30)

## Scope and status

The focused root contract set contains 8 Rust files: 235 physical lines, 204 nonempty lines and 6,665 bytes. The sorted raw-content SHA256 is `af5359c11067c2938d81f886aacc466698d013fefeb079e909d223f7aa74a127`. The files are the scene framework's public level-manager, artifact-ticket, property-value, stage, resource and handle leaves; `level_manager_lifecycle.rs`, `level_manager_project_io.rs`, `scene_artifact_io.rs`, `world_driver.rs` and their direct callers were read as implementation context but are not double-counted in this leaf fingerprint. Direct rustfmt checks pass for all 8 focused files. The root scene/module implementation and some framework files are foreign modified or untracked; no production or documentation ownership was overwritten.

This is a contract and lifecycle boundary, not a WGPU frame-loop owner. `SystemStage::ORDER` and `FIXED_LOOP` are fixed arrays, and `SceneResource` is a marker trait. Level loading/saving, reflection catalog publication and world ticking are selected operations whose costs need separate admission and generation records. No Cargo/product executable, WPR, RenderDoc or energy result is claimed because the current-source build remains blocked by the known UI/text, SDF test, stale scene/OIT and graphics reexport failures.

## Positive current behavior

- `WorldHandle` is `Copy`, and `DefaultLevelManager::try_prepare_level` uses checked atomic allocation. The maximum handle is accepted once and the next allocation returns a typed exhaustion error.
- `PreparedLevel` keeps a new level out of the registry until publication. `PreparedLevelPublication` removes an uncommitted entry on drop, and the focused tests cover both rollback and commit paths.
- `SceneArtifactIo` uses `BoundedKeyedIoLane` with an 8-entry queue and a 64 MiB per-entry quote. Same-key requests are superseded by the lane instead of both writes running; the ticket exposes terminal, wait and generation state.
- `save_world` and `save_level` capture a world before handing serialization and atomic file replacement to the bounded I/O lane. The callback does not hold the live World mutex while serializing.
- Runtime extension installation snapshots an `Arc<WorldRuntimeExtensionPlan>` before applying it. Focused tests cover reentrant publication and overlap across independent worlds, so the driver mutex is not held through extension callbacks.
- `SystemStage::ORDER` has one authority for the nine stage values. Schedule construction copies that static order once; it does not rebuild the stage table on every tick.

## Findings

### P0/P1 reflection synchronization holds every world and clones complete state

`DefaultLevelManager::sync_vm_types_atomically` holds the level-registry mutex while it clones and sorts every `LevelSystem`, locks every World, validates all registrations, then clones every complete World as a rollback snapshot before applying the new VM type set (`zircon_runtime/src/scene/module/level_manager_lifecycle.rs:159-186`). The reflection catalog calls this path when publishing a candidate registry (`zircon_runtime/src/script/vm/reflection/catalog.rs:282-349`). Work is therefore `O(L log L)` for level ordering plus full world validation and deep snapshot cost for all `L` worlds, while level creation/lookup and other registry operations wait behind the same mutex. This is selected reflection/schema mutation rather than a normal frame call, but a large editor project or frequent dynamic type update can create a main-thread stall and a memory peak proportional to the complete world set.

The rollback is semantically useful, but its ownership is too broad. Compile a checked `SceneReflectionGeneration` and per-world candidate validation without the registry lock; retain immutable world snapshots or transaction-owned deltas; publish the registry and all world type generations with one compare-and-swap receipt. A failed candidate must leave every world and the catalog generation unchanged. The dynamic gate must measure level count, entity/component rows, snapshot bytes and lock wait rather than infer cost from the number of registrations alone.

### P1 save path performs a full synchronous World clone before I/O admission

Both `save_world` and `save_level` call `level.snapshot()` before submitting work (`zircon_runtime/src/scene/module/level_manager_project_io.rs:20-41,68-91`). `LevelSystem::snapshot` clones the complete `World`, including its ECS maps, schedule, events and retained runtime containers, on the caller's thread. Only serialization and file replacement are moved to `SceneArtifactIo`. For a large editor save this makes the caller pay the full snapshot allocation and World-lock hold before the bounded lane can reject or supersede the request. The existing 64 MiB artifact quote limits serialized work but does not pre-admit the in-memory snapshot peak.

Use a dirty/generation-qualified immutable persistence snapshot: first propose serialized/source bytes and snapshot peak against the artifact lane, then transfer or share owned slabs into the I/O job. Save requests should return a typed queued/backpressured ticket before a large copy, and a superseded key must release its candidate payload. Product evidence needs snapshot bytes, serialization bytes, queue wait and caller lock time separately.

### P1 artifact generation identity saturates and the submit precheck is not key-aware

`SceneArtifactIo::submit` mints `next_generation` with `fetch_add(...).saturating_add(1)` (`zircon_runtime/src/scene/module/scene_artifact_io.rs:38-62`). After `u64::MAX`, distinct artifacts reuse the same generation, while `SceneArtifactTicket` exposes that value as the public identity. This is latent but violates the stable-generation contract used by editor save/capture consumers. Replace it with checked, non-repeating owner/device/session-qualified identity and return a typed exhaustion result before lane admission.

The same method rejects when `queue_entries >= MAX_PENDING_SCENE_ARTIFACTS` before calling the lane's key-aware `try_admit`. The lane is explicitly capable of superseding an older request for the same key, so the precheck can reject a bounded same-key replacement at high water instead of allowing the lane to replace in place. The current tests cover same-key supersession and different-key saturation separately, but not same-key replacement at saturation. Move capacity accounting into the lane proposal or add a key-aware admission receipt; do not rely on an outer count that can disagree with the lane's replacement semantics.

### P1 level publication and error payloads are explicit but not generation-qualified

`PreparedLevel::publish` inserts by raw `WorldHandle` and only uses `debug_assert!(previous.is_none())`; `PreparedLevelPublication::drop` removes by handle alone (`level_manager_lifecycle.rs:25-77`). Checked allocation makes collisions unlikely in the normal path, but the publication contract has no compare-and-swap generation or owner token if a future importer, restore path or identity migration supplies a stale prepared level. The architecture should publish a `LevelGeneration` containing handle, world generation and project/resource identity, and remove only the exact candidate. `LevelManagerError` owns URI/root/reason strings only on explicit failures, so its strings are not a stable-frame hotspot; they should remain bounded by the request/error budget rather than be optimized locally.

`ScenePropertyValue` intentionally owns strings for reflection and persistence (`property_value.rs:7-24`). It is suitable at authoring/serde boundaries, but stable property reads should consume compiled field IDs and borrowed/typed values. This is a cross-owner concern with the component descriptor and property-access plans, not a justified edit to the public enum.

## Reference-engine constraint

Unreal's `UWorld` keeps persistent and streaming levels in explicit `FLevelCollection`/`Levels` structures and tracks the active level collection in typed world state (`dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/World.h:661-739,1458-1469`). Its world tick is a world-owned lifecycle operation rather than an implicit global map scan. `FSoftObjectPath` likewise preserves structured object and subobject identity at the boundary instead of requiring repeated path-string reconstruction (`dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/SoftObjectPath.h`). These sources support typed level/resource generations and explicit lifecycle ownership; they do not prescribe Zircon's Rust serialization format, recoverable ticket ABI or lock implementation.

## Architecture handoff

- M0: add RED tests for 1/4/16/64 levels, world sizes, reflection candidate failure, snapshot bytes, same-key-at-capacity admission and artifact-generation exhaustion. Add counters for registry/world lock hold, deep clone bytes, queued/superseded/rejected tickets and caller snapshot time.
- M1: compile one immutable `SceneContractGeneration` containing level/resource/project identity, world/schema generation, stage schedule and bounded artifact policy. Use checked non-repeating level and artifact identities.
- M2: move VM reflection validation to lock-free immutable world views or transaction-owned deltas; publish catalog plus per-world schema generations atomically. No failed candidate may mutate a live World.
- M3: replace save-before-submit full cloning with a dirty, generation-qualified persistence snapshot and an admitted snapshot/serialization budget. Keep file I/O on the bounded lane and expose queued, superseded, cancelled and fault terminal states.
- M4: make stage execution consume one compiled schedule generation; leave the fixed `SystemStage` arrays as the static authority and charge worker-batch allocations to the existing ECS schedule owner.
- M5: keep `ScenePropertyValue` as the explicit authoring/serde DTO, while runtime reflection/property paths borrow compiled field/value layouts and only materialize strings for an admitted export or diagnostic.

## Acceptance gates

Dynamic acceptance requires current-source Cargo and scale evidence for level count/world size, reflection updates, save/load and same-key artifact storms. Hard gates are: failed reflection publication leaves all worlds and the registry unchanged; save admission accounts for snapshot plus serialized peak before large copying; same-key replacement remains admissible at lane capacity; every artifact/level identity is non-repeating and generation-qualified; no normal frame path performs level sorting, full World cloning or artifact I/O; stage execution retains one static order; diagnostics report actual lock, clone, queue and ticket work. Until those gates run, this root contract remains `static_complete_dynamic_pending` and does not enter `review.md`.
