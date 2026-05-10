---
related_code:
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/archetype_id.rs
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/change_detection/mod.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick_window.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/change_detection/wrappers.rs
  - zircon_runtime/src/scene/ecs/commands/mod.rs
  - zircon_runtime/src/scene/ecs/commands/command.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/commands/commands.rs
  - zircon_runtime/src/scene/ecs/component.rs
  - zircon_runtime/src/scene/ecs/component_id.rs
  - zircon_runtime/src/scene/ecs/component_registry.rs
  - zircon_runtime/src/scene/ecs/despawned_entity.rs
  - zircon_runtime/src/scene/ecs/entity_location.rs
  - zircon_runtime/src/scene/ecs/entity_registry.rs
  - zircon_runtime/src/scene/ecs/entity_registry_error.rs
  - zircon_runtime/src/scene/ecs/events.rs
  - zircon_runtime/src/scene/ecs/internal_entity.rs
  - zircon_runtime/src/scene/ecs/internal_scene_system.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_access.rs
  - zircon_runtime/src/scene/ecs/query/query_access_error.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/query/query_filter.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_state.rs
  - zircon_runtime/src/scene/ecs/removal.rs
  - zircon_runtime/src/scene/ecs/resource.rs
  - zircon_runtime/src/scene/ecs/resource_id.rs
  - zircon_runtime/src/scene/ecs/resource_registry.rs
  - zircon_runtime/src/scene/ecs/resource_store.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_error.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/stable_entity_location.rs
  - zircon_runtime/src/scene/ecs/storage/mod.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage.rs
  - zircon_runtime/src/scene/ecs/storage/component_remove_result.rs
  - zircon_runtime/src/scene/ecs/storage/storage_error.rs
  - zircon_runtime/src/scene/ecs/storage_type.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/system/events.rs
  - zircon_runtime/src/scene/ecs/system/local.rs
  - zircon_runtime/src/scene/ecs/system/param_set.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/ecs/system/removed_components.rs
  - zircon_runtime/src/scene/ecs/system/res.rs
  - zircon_runtime/src/scene/ecs/system/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_param_access.rs
  - zircon_runtime/src/scene/ecs/system/system_param_error.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/component_access.rs
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/identity.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/world.rs
implementation_files:
  - zircon_runtime/src/scene/ecs/archetype_id.rs
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/change_detection/mod.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick.rs
  - zircon_runtime/src/scene/ecs/change_detection/change_tick_window.rs
  - zircon_runtime/src/scene/ecs/change_detection/component_ticks.rs
  - zircon_runtime/src/scene/ecs/change_detection/wrappers.rs
  - zircon_runtime/src/scene/ecs/commands/mod.rs
  - zircon_runtime/src/scene/ecs/commands/command.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/ecs/commands/commands.rs
  - zircon_runtime/src/scene/ecs/component.rs
  - zircon_runtime/src/scene/ecs/component_id.rs
  - zircon_runtime/src/scene/ecs/component_registry.rs
  - zircon_runtime/src/scene/ecs/despawned_entity.rs
  - zircon_runtime/src/scene/ecs/entity_location.rs
  - zircon_runtime/src/scene/ecs/entity_registry.rs
  - zircon_runtime/src/scene/ecs/entity_registry_error.rs
  - zircon_runtime/src/scene/ecs/events.rs
  - zircon_runtime/src/scene/ecs/internal_entity.rs
  - zircon_runtime/src/scene/ecs/internal_scene_system.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_access.rs
  - zircon_runtime/src/scene/ecs/query/query_access_error.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/query/query_filter.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_state.rs
  - zircon_runtime/src/scene/ecs/removal.rs
  - zircon_runtime/src/scene/ecs/resource.rs
  - zircon_runtime/src/scene/ecs/resource_id.rs
  - zircon_runtime/src/scene/ecs/resource_registry.rs
  - zircon_runtime/src/scene/ecs/resource_store.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_error.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/stable_entity_location.rs
  - zircon_runtime/src/scene/ecs/storage/mod.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage.rs
  - zircon_runtime/src/scene/ecs/storage/component_remove_result.rs
  - zircon_runtime/src/scene/ecs/storage/storage_error.rs
  - zircon_runtime/src/scene/ecs/storage_type.rs
  - zircon_runtime/src/scene/ecs/system/mod.rs
  - zircon_runtime/src/scene/ecs/system/events.rs
  - zircon_runtime/src/scene/ecs/system/local.rs
  - zircon_runtime/src/scene/ecs/system/param_set.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/ecs/system/removed_components.rs
  - zircon_runtime/src/scene/ecs/system/res.rs
  - zircon_runtime/src/scene/ecs/system/system_param.rs
  - zircon_runtime/src/scene/ecs/system/system_param_access.rs
  - zircon_runtime/src/scene/ecs/system/system_param_error.rs
  - zircon_runtime/src/scene/ecs/system/system_state.rs
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/change_detection.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/component_access.rs
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/identity.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/world.rs
plan_sources:
  - user: 2026-05-08 ECS to render chain milestone execution
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .codex/plans/ECS SystemParam Commands Change Detection 下一阶段计划.md
  - user: 2026-05-08 Bevy-grade ECS / Reflect / Scene / Transform roadmap implementation
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/ZirconEngine Bevy-Style 自研 ECS 与场景编辑模式计划.md
tests:
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - tests/acceptance/ecs-to-render-chain.md
  - .github/workflows/ci.yml
doc_type: module-detail
---

# Scene ECS Kernel

`zircon_runtime::scene::ecs` is the local ECS kernel beneath the public `World` authority. The public scene identity remains `EntityId = u64` for editor, asset, and serialized scene compatibility. Internally, the kernel now carries these lower-layer responsibilities:

- stable-to-internal generational entity identity and storage primitives for the Bevy-grade ECS roadmap;
- typed component/resource/bundle contracts and registries for the M2 public `World` API without importing `bevy_ecs`;
- typed resource and event stores for systems without ad hoc global maps;
- typed query/access descriptors for M3 runtime borrow-safety checks;
- typed deferred command queues, Bevy-style system parameter state, event params, and per-system run windows for M4-style systems;
- per-component/resource change ticks plus `Added<T>`, `Changed<T>`, `Ref<T>`, `Mut<T>`, and removed-component readers for M5 change detection;
- stage/system descriptors that let `WorldDriver` run native scene systems and plugin hooks in one deterministic schedule.

The implementation follows Bevy's split between external entity values, entity locations, component IDs, storage type selection, and table/sparse backing stores. It deliberately diverges by keeping Zircon's serialized `EntityId` as the stable project-facing key instead of making the generational handle the saved identity. Fyrox's generational pool model also informs the stale-handle behavior: an index alone is never enough to prove liveness after despawn.

## Identity Model

- `InternalEntity` is a compact generational handle used only inside the runtime ECS kernel.
- `EntityRegistry` maps stable scene `EntityId` values to `InternalEntity` and `EntityLocation` records.
- `EntityLocation` currently records the archetype and table row needed by the storage/query milestones. Fixed scene maps remain the serialization/editor authority for existing scene components, while the typed ECS storage now mirrors component presence and typed values by `ComponentId`.
- `World::internal_entity`, `World::internal_entity_location`, and `World::contains_internal_entity` expose read-only inspection for tests and later runtime systems without replacing the public stable ID API.

## Storage Model

- `ComponentId`, `ResourceId`, and `ArchetypeId` are typed indices scoped to the ECS kernel. They are intentionally separate from asset `ResourceId` under `zircon_runtime::core::resource`.
- `StorageType::Table` is the default fast-iteration path. Removing a table component uses swap-remove and reports the moved entity so callers can update table rows.
- `StorageType::SparseSet` is the random-access insertion/removal path and does not move unrelated entities on removal.
- `ComponentRegistry` assigns `ComponentId` values for Rust `Component` types and dynamic plugin component ids. Rust components use their `Component::STORAGE_TYPE`; dynamic plugin component presence is represented as sparse-set storage.
- `ResourceRegistry` assigns `ResourceId` values for Rust `Resource` types. This ECS `ResourceId` is separate from asset/resource-manager ids.
- `ComponentStorage` is a type-erased lower-layer storage primitive for M1 tests and the M2 typed component API. It enforces one storage type per component ID and reports type mismatch rather than silently changing storage semantics. Each stored component also carries `ComponentTicks { added, changed }` so query filters can evaluate Bevy-style `Added<T>` and `Changed<T>` windows. Despawning a scene entity removes all typed component storage entries for its internal entity.

## World Integration

`World` owns `EntityRegistry`, `ComponentRegistry`, `ComponentStorage`, `ResourceRegistry`, `ResourceStore`, `CommandQueue`, and the current `ChangeTick` as skipped runtime state. Spawning, inserting saved node records, despawning, and project loading update or rebuild those runtime-only maps while serialized scene files remain unchanged. This keeps current scene roundtrips stable and gives later milestones live internal identity/component/resource maps for archetype movement, queries, deferred commands, and change detection.

Direct `World` serde deserialization follows the same runtime-only rehydration rule without invoking full project-load normalization. The custom deserialize path restores the persistent scene maps, then rebuilds `EntityRegistry` and typed component presence so `contains_component_id(...)` observes serialized fixed/dynamic scene components immediately, without adding default cameras/lights or other project policy defaults.

The M2 typed API is intentionally narrow:

- `World::spawn(bundle)` creates a stable scene entity and inserts a `Bundle`.
- `World::insert`, `World::get`, `World::get_mut`, and `World::remove` work for Rust `Component` types.
- `World::component_id`, `World::registered_component_id`, `World::registered_dynamic_component_id`, `World::component_count_for_id`, and `World::contains_component_id` expose component identity/presence for tests and later query work.
- Existing fixed scene setters such as rigid body, transform, active state, render-layer mask, and dynamic plugin component attachment/property writes now flow through the typed component presence path instead of maintaining a separate identity model.
- Fixed scene component maps still back serialization, property paths, editor projections, and render extract data. Typed storage mirrors these values so M3+ queries can converge without breaking the existing public scene model.

## Resources And Events

`ResourceStore` and `EventStore` are type-indexed support layers for scene systems. They use `TypeId` internally but expose typed `insert`, `get`, `get_mut`, `remove`, `send`, `update`, and `drain` operations so systems avoid ad hoc global maps. `ResourceStore` records `ComponentTicks` for resources too, allowing `Res<T>` and `ResMut<T>` system params to expose `is_added()`, `is_changed()`, and `last_changed()`. `Events<T>` keeps a `next` queue and a `current` queue; `send` publishes into the next frame, while `update` swaps it into the readable queue. The explicit `Default` implementation intentionally does not require `T: Default`.

`World::resource_id`, `World::registered_resource_id`, `World::insert_resource`, `World::resource`, `World::get_resource`, `World::resource_mut`, `World::get_resource_mut`, and `World::remove_resource` expose the M2 resource API over `ResourceRegistry` and `ResourceStore`. `World::resource` and `World::resource_mut` intentionally panic on missing resources; use the `get_*` variants for optional access.

`World::send_event`, `World::update_events`, and `World::events` expose the runtime event queue. `EventWriterParam<T>` writes through `EventStore::send` into `next`, and `EventReaderParam<T>` reads only the current queue. This mirrors Bevy's current/next event visibility rule without adding observer/lifecycle hooks in this milestone.

## Query Model

The M3 query foundation adds `QueryState<D, F>` under `zircon_runtime::scene::ecs::query` and exposes it through `World::query<D>()` and `World::query_filtered<D, F>()`. Query state is a compiled access descriptor rather than an owning view of the world: it records the component reads, writes, `With<T>` filters, and `Without<T>` filters needed by a system or manual caller, while `World` remains the authority for entity storage and component values.

Query data marker types are intentionally allowed to be borrowed shapes such as `&T`, `&mut T`, and `Option<&T>` rather than `'static` marker structs. `Component` itself remains `'static + Send + Sync`; the query marker only describes access, while the fetched item lifetime comes from the `World` borrow passed to `QueryState::iter(...)` or `QueryState::for_each_mut(...)`.

The initial supported data forms are deliberately narrow:

- `&T` reads a required `Component`.
- `&mut T` mutably reads a required `Component` through `QueryState::for_each_mut`.
- `Ref<T>` reads a required component with `is_added()`, `is_changed()`, and `last_changed()` metadata.
- `Mut<T>` mutably reads a required component with the same metadata surface as `Ref<T>` and uses the existing mutable fetch path that marks the component changed.
- `Option<&T>` reads an optional component without filtering out entities that lack it.
- `EntityId` returns the stable public scene entity id.
- tuples up to four items compose read-only query data.

`With<T>` and `Without<T>` compose as query filters and also participate in access disjointness. `Added<T>` and `Changed<T>` read component tick metadata against the current system run window; `Changed<T>` includes newly added components, matching Bevy's behavior. `QueryAccess::conflicts_with` reports a conflict when two states both touch the same component and at least one writes it, unless their filter sets prove the matching entity sets are disjoint, such as `With<Player>` versus `Without<Player>`. `QueryState::try_new` rejects conflicting access inside one query, including duplicate `&mut T` access.

This is Bevy-inspired but intentionally smaller than Bevy's full `WorldQuery` system. Mutable tuple query data is still deferred; the current mutable path supports one mutable data item (`&mut T` or `Mut<T>`) so M3/M5 can establish borrow-safety diagnostics and change-tick behavior before wider system execution.

## System Params, Commands, And Change Detection

`SystemState<P>` is the local system execution wrapper. It caches `P::State`, records `SystemParamAccess`, tracks the system's `last_run` tick, advances `World` to a fresh `ChangeTick` for each run, and fetches `P::Item<'_>` for the user closure. Tuple params compose up to eight items and share one access descriptor so duplicate mutable resource, component, or event access is rejected before a system runs.

Supported system params in this slice are:

- `QueryState<D, F>` as a param, yielding `Query<D, F>` with `iter()` and `for_each_mut(...)` over the current tick window.
- `ResParam<T>` and `ResMutParam<T>`, yielding `Res<T>` and `ResMut<T>` wrappers with `Deref`, `DerefMut` for mutable access, and `is_added()` / `is_changed()` / `last_changed()` helpers.
- `Option<ResParam<T>>` and `Option<ResMutParam<T>>`, returning `None` when a resource is absent while still recording the declared access. Required `ResParam<T>` and `ResMutParam<T>` fail `SystemState::new(...)` with `SystemParamError::MissingResource` if the resource is missing.
- `CommandsParam`, yielding `Commands`, which queues deferred `spawn`, `spawn_empty`, `entity`, `entity_or_spawn`, `despawn`, component insert/remove, bundle insert, resource insert/remove, or custom `Command`/closure work.
- `EventReaderParam<T>` and `EventWriterParam<T>`, yielding current-queue readers and next-queue writers over runtime `EventStore`.
- `RemovedComponentsParam<T>`, yielding a stateful `RemovedComponents<T>` reader that reports entities whose `T` was removed after the reader's last cursor.
- `ParamSet<(P0, P1, ...)>` for arities one through eight, allowing potentially conflicting params to be declared together while exposing only segmented `p0()` through `p7()` access.
- `LocalParam<T>`, yielding persistent local state initialized from `T::default()` and preserved across `SystemState::run(...)` calls.

`ParamSet` checks each child parameter against the access baseline from outside the set, then merges child access into the outer descriptor without preserving sibling query filters. This keeps sibling params allowed to conflict with each other while making the aggregate access conservative when another system compares against the whole `ParamSet`.

`World::commands()` exposes the same deferred queue outside system params, and `World::apply_deferred()` applies queued commands at a fresh active change tick. Command effects are intentionally invisible until `apply_deferred()` runs, which gives schedule work a deterministic sync point equivalent to Bevy's `ApplyDeferred`. `Commands::spawn(...)` and `spawn_empty()` reserve stable `EntityId` values from `World::next_id` before application, so `EntityCommands::id()` is stable even while the entity is not yet visible in the world. The current command queue stores ordered typed `Command` values behind an erased trait, with `FnCommand` keeping closure commands available; the semantic contract is ordered application and deferred visibility, not Bevy byte-buffer storage parity.

Change detection uses a simple monotonic `u64`-backed `ChangeTick` rather than Bevy's wrapped `u32` maintenance path. Direct world mutations outside a system advance the world tick immediately. Mutations inside `SystemState::run(...)` use that system's active tick, so the mutating system does not see its own previous run as changed on the next run while other systems with older `last_run` windows can observe the change. `Changed<T>` intentionally includes newly added components, matching Bevy. `World::get_mut`, `World::insert`, resource mutation, deferred command application, fixed scene component maps, and typed ECS storage all flow through this tick path.

Removed-component events are runtime-only world state. Direct `World::remove<T>`, recursive/despawn entity removal, deferred `EntityCommands::remove<T>()`, and deferred despawn all record the removed component type and stable entity id. The reader cursor lives in `RemovedComponentReader<T>` / `RemovedComponentsParam<T>`, so each system observes only events it has not already consumed.

## Schedule Model

`Schedule` owns both the public stage order and a `SceneSystemRegistry`. The fixed stage order is:

`First -> PreUpdate -> FixedUpdate -> Update -> PostUpdate -> Last -> RenderExtract`.

`LateUpdate` is intentionally absent. This is a hard cutover to the Bevy-style `PostUpdate` name plus Zircon's explicit `RenderExtract` stage.

`SceneSystemDescriptor` records a stable system id, target stage, deterministic order, and the internal built-in system enum. `SceneSystemRegistry` rejects blank or whitespace-padded ids, rejects duplicate ids, and sorts systems by stage rank, order, and id. The built-in registry currently installs:

- `zircon.scene.hierarchy_validity` in `PostUpdate`;
- `zircon.scene.active_hierarchy` in `PostUpdate`;
- `zircon.scene.world_transform` in `PostUpdate`;
- `zircon.scene.node_cache` in `PostUpdate`;
- `zircon.scene.render_extract_prepare` in `RenderExtract`.

`InternalSceneSystem::ApplyDeferred` is a runtime-only synchronization system that calls `World::apply_deferred()`. It can be registered explicitly when a stage needs a named sync point. The current runner also flushes deferred commands after every internal system except `ApplyDeferred` itself and after every plugin hook, so command effects become visible to later ordered steps in the same stage. This is a deterministic local integration point, not Bevy's full graph analysis or automatic insertion algorithm.

`SceneScheduleRunner` merges native built-in descriptors with plugin `SceneRuntimeHookRegistration` values for each stage. It defers eager derived-state flushing while a stage is running so plugin hooks can mutate local scene state through the normal dirty path. When the stage finishes successfully, the runner disables defer mode and runs that stage's built-in systems once more. This preserves plugin hook order semantics while still making hook mutations and deferred ECS command effects visible before the next stage or frame boundary.

## Derived State Systems

`DerivedStateDirty` is the dirty-bit gate for built-in systems. Hierarchy mutations mark hierarchy, active state, world transforms, node cache, and render extract dirty. Active changes mark active state, node cache, and render extract dirty. Transform changes mark world transforms, node cache, and render extract dirty. Mobility and render-layer updates mark the node cache and render extract dirty without forcing transform propagation.

M2 makes scene mutators dirty-only. Spawn, parent changes, transform setters, active setters, fixed component setters, typed API writes, project record insertion, and property writes no longer eagerly flush pending scene systems. `World::new()` and project-load normalization still call the explicit `flush_scene_systems_now()` boundary so bootstrapped and loaded worlds start from coherent cached state.

`World::run_internal_scene_system` consumes one `InternalSceneSystem` and clears only that system's dirty bit after it runs. `flush_pending_scene_systems()` and `flush_scene_systems_now()` are explicit flush boundaries; scheduled ticks run native systems through `WorldDriver` and `SceneScheduleRunner` rather than hiding propagation inside every mutator.

Direct reads have a split contract during dirty windows. `World::world_transform()` and `World::active_in_hierarchy()` project current values from authoritative local transform, hierarchy, and active-self maps without clearing dirty bits. `World::world_matrix()` and `World::nodes()` remain schedule-maintained cached outputs. `World::node_records()` and owned `World::find_node()` project current direct records for callers that need up-to-date node data before `PostUpdate` refreshes the cache.

The `RenderExtractPrepare` built-in flushes pending hierarchy/active/transform/node-cache state before the frame extract is built, then clears the render-extract dirty bit. M3 makes the prepared scene frame extract canonical: `World` now fills `RenderFrameExtract` sections directly after the dirty-state flush, while `SceneViewportRenderPacket` and `RenderFrameExtract::from_snapshot(...)` remain limited to snapshot preview, roundtrip, and legacy adapter callers.

## Validation Scope

The M1 tests cover slot reuse with stale generation rejection, stable ID to internal location mapping in `World`, skipped registry serialization with roundtrip rebuild, table swap-remove behavior, sparse-set removal behavior, typed resource/event behavior, schedule stage order, duplicate/blank system rejection, plugin hook mutation ordering, render-extract stage flushing, and existing scene roundtrip paths.

The typed ECS M2 tests cover tuple bundle spawn, typed insert/get/get_mut/remove, typed resource registration/replacement, fixed component setter convergence through `ComponentId`, dynamic plugin component presence through `ComponentId`, and runtime-only typed ECS state staying out of scene project serialization.

The typed ECS M3 query tests cover required component reads, optional component reads, stable `EntityId` query items, `With<T>` and `Without<T>` filters, single-component mutable iteration, fixed scene component queries, access conflict detection, filter-proven disjointness, and duplicate mutable access rejection.

The typed ECS system-param/change-detection tests cover deferred command visibility before/after `World::apply_deferred()`, entity command builder ordering, tuple system params up to eight items, tuple params combining query/resource/commands access, optional and required resource params, `ParamSet` segmented conflicting access up to eight items, event reader/writer current/next queues, persistent local params, `Ref<T>`/`Mut<T>` query wrappers, `Added<T>` windows, `Changed<T>` windows after direct mutable component access, removed-component readers, and explicit `ApplyDeferred` schedule flushing.

The transform/active dirty-state M2 tests cover dirty-only mutators, direct read projection without clearing dirty bits, `PostUpdate` cache propagation, `RenderExtract` flushing after parent reorder and active changes, and mobility changes that dirty cached node/render state without forcing transform propagation.

The canonical render-extract M3 tests cover direct `RenderFrameExtract` section population from `World`, including request-driven camera aspect, visibility buckets, postprocess defaults, VG debug/default sidebands, and disabled Hybrid GI sidebands. A structure guard also rejects reintroducing `RenderFrameExtract::from_snapshot(...)` in the scene render-extract producer.

Historical focused evidence before unrelated active-lane blockers appeared: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked` passed with `3 passed; 0 failed; 1016 filtered out`, and `cargo test -p zircon_runtime --lib world_mutations_mark_derived_state_dirty_until_post_update_systems_flush --locked` passed for the dirty-only mutator regression.

Fresh 2026-05-08 support-fix evidence: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api::runtime_only_typed_ecs_state_is_not_serialized --locked --target-dir target\codex-shared-a -- --nocapture` passed after direct serde deserialization began rebuilding runtime-only ECS identity/presence. The package-level `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime` also passed Cargo build and Cargo test.

Fresh 2026-05-08 M1 render-profile support evidence: `cargo check -p zircon_runtime --lib --locked` and `cargo check -p zircon_app --locked --all-targets` both finished successfully with warning-only output after the query marker lifetime bound was narrowed to the access descriptor layer. The production ECS query module tree and public `World` query entrypoints compile.

Fresh 2026-05-08 M2 boundary evidence passed after active asset importer and UI focus compile drift were cut to their current contracts. `cargo test -p zircon_runtime --lib scene::tests::ecs_schedule::render_extract_prepare_flushes_parent_reorder_and_active_changes --locked --message-format short` passed with `1 passed; 0 failed; 1061 filtered out`. `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short` passed with `45 passed; 0 failed; 1018 filtered out`. `cargo test -p zircon_runtime --lib graphics::tests --locked --message-format short` passed with `107 passed; 0 failed; 956 filtered out`.

Fresh 2026-05-08 M3 boundary evidence passed in `E:\cargo-targets\zircon-ecs-render-m3` after the direct extract test fixture was aligned with the shared static-mobility mutation guard. The focused direct `RenderFrameExtract` population test, structural snapshot-adapter guard, and scene-produced M5 flagship sideband test each passed with `1 passed; 0 failed; 1070 filtered out`. The broader `scene::tests` filter passed with `47 passed; 0 failed; 1024 filtered out`, and `graphics::tests` passed with `108 passed; 0 failed; 963 filtered out`.

Fresh 2026-05-08 SystemParam/Commands/change-detection evidence: `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully with warning-only output. `cargo test -p zircon_runtime --lib scene::tests::ecs_systems --locked --message-format short` passed with `4 passed; 0 failed; 1080 filtered out`. `cargo test -p zircon_runtime --lib scene::tests::ecs_query --locked --message-format short` passed with `5 passed; 0 failed; 1079 filtered out`. `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short` passed with `4 passed; 0 failed; 1080 filtered out`. Broader scene validation with `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short` passed with `51 passed; 0 failed; 1033 filtered out`.

Fresh focused evidence for this document is recorded in `tests/acceptance/ecs-to-render-chain.md`.
