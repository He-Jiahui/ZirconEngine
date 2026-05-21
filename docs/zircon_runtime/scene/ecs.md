---
related_code:
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/ecs/archetype_id.rs
  - zircon_runtime/src/scene/ecs/archetype_index.rs
  - zircon_runtime/src/scene/ecs/archetype_signature.rs
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
  - zircon_runtime/src/scene/ecs/lifecycle.rs
  - zircon_runtime/src/scene/ecs/messages.rs
  - zircon_runtime/src/scene/ecs/observer.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/ecs/query/cached_query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_access.rs
  - zircon_runtime/src/scene/ecs/query/query_access_error.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/query/query_entity_error.rs
  - zircon_runtime/src/scene/ecs/query/query_filter.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_single_error.rs
  - zircon_runtime/src/scene/ecs/query/query_state.rs
  - zircon_runtime/src/scene/ecs/removal.rs
  - zircon_runtime/src/scene/ecs/resource.rs
  - zircon_runtime/src/scene/ecs/resource_id.rs
  - zircon_runtime/src/scene/ecs/resource_registry.rs
  - zircon_runtime/src/scene/ecs/resource_store.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/schedule_error.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
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
  - zircon_runtime/src/scene/ecs/system/messages.rs
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
  - zircon_runtime/src/scene/world/messages.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
implementation_files:
  - zircon_runtime/src/scene/ecs/archetype_id.rs
  - zircon_runtime/src/scene/ecs/archetype_index.rs
  - zircon_runtime/src/scene/ecs/archetype_signature.rs
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
  - zircon_runtime/src/scene/ecs/lifecycle.rs
  - zircon_runtime/src/scene/ecs/messages.rs
  - zircon_runtime/src/scene/ecs/observer.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/ecs/query/cached_query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_access.rs
  - zircon_runtime/src/scene/ecs/query/query_access_error.rs
  - zircon_runtime/src/scene/ecs/query/query_data.rs
  - zircon_runtime/src/scene/ecs/query/query_entity_error.rs
  - zircon_runtime/src/scene/ecs/query/query_filter.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_single_error.rs
  - zircon_runtime/src/scene/ecs/query/query_state.rs
  - zircon_runtime/src/scene/ecs/removal.rs
  - zircon_runtime/src/scene/ecs/resource.rs
  - zircon_runtime/src/scene/ecs/resource_id.rs
  - zircon_runtime/src/scene/ecs/resource_registry.rs
  - zircon_runtime/src/scene/ecs/resource_store.rs
  - zircon_runtime/src/scene/ecs/scene_system_descriptor.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/schedule_error.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
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
  - zircon_runtime/src/scene/ecs/system/messages.rs
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
  - zircon_runtime/src/scene/world/messages.rs
  - zircon_runtime/src/scene/world/observers.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
plan_sources:
  - user: 2026-05-08 ECS to render chain milestone execution
  - .codex/plans/ZirconEngine ECS 到渲染链路完善里程碑计划.md
  - .codex/plans/ECS SystemParam Commands Change Detection 下一阶段计划.md
  - user: 2026-05-08 Bevy-grade ECS / Reflect / Scene / Transform roadmap implementation
  - user: 2026-05-16 continue Bevy-grade M6 observers and messages
  - user: 2026-05-16 M8.8 reflection documentation cleanup
  - user: 2026-05-17 M11 query cache performance-parity slice
  - user: 2026-05-20 M11 schedule conflict graph performance-parity slice
  - user: 2026-05-20 M11 query iter_many helper slice
  - user: 2026-05-20 M11 query iter_many cached helper slice
  - user: 2026-05-21 M11 query iter_many_mut helper slice
  - user: 2026-05-21 M11 query iter_many cached-direct helper slice
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/ZirconEngine Bevy-Style 自研 ECS 与场景编辑模式计划.md
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/bevy/crates/bevy_ecs/src/query/iter.rs
  - dev/bevy/crates/bevy_ecs/src/query/fetch.rs
  - dev/bevy/crates/bevy_ecs/src/query/error.rs
  - dev/bevy/crates/bevy_ecs/src/system/query.rs
  - dev/bevy/crates/bevy_ecs/src/archetype.rs
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/bevy/crates/bevy_ecs/src/storage/sparse_set.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/schedule.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/error.rs
  - zircon_runtime/src/core/job_scheduler.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_identity_storage.rs
  - zircon_runtime/src/scene/tests/ecs_observers_messages.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/scene/tests/ecs_schedule.rs
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/resources.rs
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
- cached query state metadata and runtime archetype signatures for the M11 performance-parity path;
- typed deferred command queues, Bevy-style system parameter state, event params, and per-system run windows for M4-style systems;
- per-component/resource change ticks plus `Added<T>`, `Changed<T>`, `Ref<T>`, `Mut<T>`, and removed-component readers for M5 change detection;
- stage/system descriptors plus same-stage conflict graph, conservative parallel-batch foundations, and a `JobScheduler`-backed batch executor for independent registered tasks while `WorldDriver` continues to run native scene systems and plugin hooks in one deterministic schedule.

The implementation follows Bevy's split between external entity values, entity locations, component IDs, storage type selection, and table/sparse backing stores. It deliberately diverges by keeping Zircon's serialized `EntityId` as the stable project-facing key instead of making the generational handle the saved identity. Fyrox's generational pool model also informs the stale-handle behavior: an index alone is never enough to prove liveness after despawn.

## Identity Model

- `InternalEntity` is a compact generational handle used only inside the runtime ECS kernel.
- `EntityRegistry` maps stable scene `EntityId` values to `InternalEntity` and `EntityLocation` records.
- `EntityLocation` records the entity's current `ArchetypeId` plus the row in that archetype's runtime entity list. Fixed scene maps remain the serialization/editor authority for existing scene components, while the typed ECS storage mirrors component presence and typed values by `ComponentId`.
- `World::internal_entity`, `World::internal_entity_location`, and `World::contains_internal_entity` expose read-only inspection for tests and later runtime systems without replacing the public stable ID API.
- `World::refresh_entity_archetype(...)` reads the optional `StableEntityLocation` directly and preserves the previous archetype id when it exists, so structural component changes move entities through `ArchetypeIndex` without treating a missing stable id as a registry error.

## Storage Model

- `ComponentId`, `ResourceId`, and `ArchetypeId` are typed indices scoped to the ECS kernel. They are intentionally separate from asset `ResourceId` under `zircon_runtime::core::resource`.
- `ArchetypeSignature` splits table and sparse-set component ids for a unique component-set identity, following Bevy's distinction between archetypes and tables. `ArchetypeIndex` owns runtime archetype records, maps signatures to `ArchetypeId`, and indexes components to candidate archetypes for query cache rebuilds.
- `StorageType::Table` is the default fast-iteration path. Removing a table component uses swap-remove and reports the moved entity so callers can update table rows.
- `StorageType::SparseSet` is the random-access insertion/removal path and does not move unrelated entities on removal.
- `ComponentRegistry` assigns `ComponentId` values for Rust `Component` types and dynamic plugin component ids. Rust components use their `Component::STORAGE_TYPE`; dynamic plugin component presence is represented as sparse-set storage.
- `ResourceRegistry` assigns `ResourceId` values for Rust `Resource` types. This ECS `ResourceId` is separate from asset/resource-manager ids.
- `ComponentStorage` is a type-erased lower-layer storage primitive for M1 tests and the M2 typed component API. It enforces one storage type per component ID and reports type mismatch rather than silently changing storage semantics. Each stored component also carries `ComponentTicks { added, changed }` so query filters can evaluate Bevy-style `Added<T>` and `Changed<T>` windows. Despawning a scene entity removes all typed component storage entries for its internal entity. M11 exposes `ComponentStorageLocation`, `get_table_row(...)`, and `get_with_ticks_at_location(...)` so later dense query iteration can use table rows directly instead of rediscovering component rows through entity maps. Table-row location reads verify the cached entity before returning a value, so stale rows caused by swap-remove fail closed.

## Registry And Reflection Relationship

The ECS registries and the reflection registry are deliberately separate layers:

- `ComponentRegistry` assigns runtime `ComponentId` values for typed Rust components and dynamic plugin component presence. It is the storage/query identity layer.
- `ResourceRegistry` assigns runtime `ResourceId` values for typed scene resources. It is the system-param/resource-store identity layer.
- `ComponentTypeRegistry` remains the plugin-facing JSON descriptor input/cache for dynamic components. Plugins still submit `ComponentTypeDescriptor` values there.
- `TypeRegistry` is the schema/read/write reflection authority. It stores neutral `ReflectTypeRegistration` metadata plus optional component/resource adapter slots used by `WorldReflection`.

Dynamic plugin component registration now feeds both paths in one mutation sequence: validate/project the `ComponentTypeDescriptor` into a reflected JSON schema, reject duplicate reflected type paths before mutating descriptor state, register the descriptor in `ComponentTypeRegistry`, then insert the dynamic `ReflectComponent` adapter into `TypeRegistry`. This lets ECS storage/query code keep stable component IDs while editor inspectors, resources, remote DTO tests, and future scene serialization use one reflected schema surface.

`WorldReflection` is not a storage owner. It resolves type paths through `TypeRegistry`, routes component addresses to `ReflectComponent`, routes resource addresses to `ReflectResource`, and calls normal `World` mutation/read APIs so change ticks, dynamic JSON property validation, fixed component dirty bits, and resource ticks stay coherent with the ECS layer.

## World Integration

`World` owns `EntityRegistry`, `ComponentRegistry`, `ComponentStorage`, `ResourceRegistry`, `ResourceStore`, `CommandQueue`, and the current `ChangeTick` as skipped runtime state. Spawning, inserting saved node records, despawning, and project loading update or rebuild those runtime-only maps while serialized scene files remain unchanged. This keeps current scene roundtrips stable and gives later milestones live internal identity/component/resource maps for archetype movement, queries, deferred commands, and change detection.

Project scene assets now persist the full M5 fixed light set that the world can author: ambient, directional, point, rect, and spot lights. `World::from_scene_asset(...)` converts `SceneAmbientLightAsset` and `SceneRectLightAsset` into the same fixed component maps used by editor-created nodes, and `World::to_scene_asset(...)` writes them back so save/load does not lose the Bevy-aligned ambient and rectangular area light product fields.

Direct `World` serde deserialization follows the same runtime-only rehydration rule without invoking full project-load normalization. The custom deserialize path restores the persistent scene maps, then rebuilds `EntityRegistry` and typed component presence so `contains_component_id(...)` observes serialized fixed/dynamic scene components immediately, without adding default cameras/lights or other project policy defaults.

The M2 typed API is intentionally narrow:

- `World::spawn(bundle)` creates a stable scene entity and inserts a `Bundle`.
- `World::insert`, `World::get`, `World::get_mut`, and `World::remove` work for Rust `Component` types.
- `World::component_id`, `World::registered_component_id`, `World::registered_dynamic_component_id`, `World::component_count_for_id`, and `World::contains_component_id` expose component identity/presence for tests and later query work.
- Existing fixed scene setters such as rigid body, transform, active state, render-layer mask, and dynamic plugin component attachment/property writes now flow through the typed component presence path instead of maintaining a separate identity model.
- Fixed scene component maps still back serialization, property paths, editor projections, and render extract data. Typed storage mirrors these values so M3+ queries can converge without breaking the existing public scene model. The light map set now includes Bevy-informed ambient and rect light authoring data beside directional, point, and spot lights; rect light direction is transform-derived during render extraction rather than stored as a separate field.

## Resources And Events

`ResourceStore` and `EventStore` are type-indexed support layers for scene systems. They use `TypeId` internally but expose typed `insert`, `get`, `get_mut`, `remove`, `send`, `update`, and `drain` operations so systems avoid ad hoc global maps. `ResourceStore` records `ComponentTicks` for resources too, allowing `Res<T>` and `ResMut<T>` system params to expose `is_added()`, `is_changed()`, and `last_changed()`. `Events<T>` keeps a `next` queue and a `current` queue; `send` publishes into the next frame, while `update` swaps it into the readable queue. The explicit `Default` implementation intentionally does not require `T: Default`.

`World::resource_id`, `World::registered_resource_id`, `World::insert_resource`, `World::resource`, `World::get_resource`, `World::resource_mut`, `World::get_resource_mut`, and `World::remove_resource` expose the M2 resource API over `ResourceRegistry` and `ResourceStore`. `World::resource` and `World::resource_mut` intentionally panic on missing resources; use the `get_*` variants for optional access.

`World::send_event`, `World::update_events`, and `World::events` expose the older runtime event queue. `EventWriterParam<T>` writes through `EventStore::send` into `next`, and `EventReaderParam<T>` reads only the current queue. This mirrors Bevy's current/next event visibility rule and remains separate from the M6 immediate observer and cursor-message paths below.

## Observers, Lifecycle, And Messages

M6 adds the first Bevy-style push observer and scheduled message foundations without replacing the older `Events<T>` queue. The split is intentional:

- `World::trigger_event(event)` and `World::trigger_entity_event(entity, event)` are immediate observer paths. Registered callbacks run during the trigger call, before control returns to the caller.
- `World::send_message(message)` writes to a retained `Messages<T>` buffer. `MessageReaderParam<T>` owns a persistent `MessageCursor<T>` so a system only reads messages it has not consumed before.
- Existing `Events<T>` remains the current/next frame queue used by earlier system-param tests and plugin-adjacent runtime flows.

`ObserverStore` is runtime-only skipped world state. It stores cloneable function-table callbacks for three scopes:

- component lifecycle observers keyed by `LifecycleEventKind` plus `ComponentId`;
- global immediate event observers keyed by Rust `TypeId`;
- entity-targeted immediate event observers keyed by Rust `TypeId` plus stable `EntityId`.

Observer callbacks are cloned out of the store before invocation. This avoids borrowing `World::observers` while user code receives `&mut World`, and allows observers to mutate the world through the normal typed API. `ObserverId` is an opaque runtime handle; `World::remove_observer` removes matching callbacks across all observer scopes.

`LifecycleEventKind` currently covers `Add`, `Insert`, `Replace`, `Remove`, and `Despawn`, matching the milestone language while staying close to Bevy's lifecycle split between add/insert and removal/despawn behavior. `World::insert<T>` triggers `Add` for first presence, `Replace` for an existing component, and `Insert` after every successful insert. `World::remove<T>` triggers `Remove` before the component leaves storage. `World::remove_entity` triggers `Remove` and `Despawn` for every component id still present on that entity before storage and fixed scene maps are cleared.

`MessageStore` is the scheduled counterpart to immediate observers. It stores `Messages<T>` by `TypeId`, assigns monotonically increasing `MessageId<T>` values, and exposes typed `MessageWriterParam<T>` / `MessageReaderParam<T>` system params. Message reads do not clear the shared buffer; the reader cursor advances per system state, matching Bevy's reader-cursor model. `World::clear_messages<T>` is the explicit retention boundary for later schedule maintenance.

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
- `StableEntityLocation` returns the internal stable-to-generational location cached by `World`.
- tuples up to eight items compose query data/filter shapes in the current hand-written implementation ceiling.

`With<T>` and `Without<T>` compose as query filters and also participate in access disjointness. `Added<T>` and `Changed<T>` read component tick metadata against the current system run window; `Changed<T>` includes newly added components, matching Bevy's behavior. `QueryAccess::conflicts_with` reports a conflict when two states both touch the same component and at least one writes it, unless their filter sets prove the matching entity sets are disjoint, such as `With<Player>` versus `Without<Player>`. `QueryAccess::conflicting_components_with(...)` returns the concrete `ComponentId` set for the schedule-conflict layer. `QueryState::try_new` rejects conflicting access inside one query, including duplicate `&mut T` access.

This is Bevy-inspired but intentionally smaller than Bevy's full `WorldQuery` system. Mutable tuple query data is still deferred; the current mutable path supports one mutable data item (`&mut T` or `Mut<T>`) so M3/M5 can establish borrow-safety diagnostics and change-tick behavior before wider system execution.

M11 starts the performance-parity path by adding explicit query cache metadata without changing the default query API. Bevy's `QueryState` stores matched table/archetype bitsets plus an archetype generation and updates them when the world gains new archetypes; Bevy's query iterators then set a table or archetype storage context before fetching item rows. Zircon's first M11 slice added an entity-set cache: `QueryState::update_cache(world)` stores stable entity ids that structurally match the query data and archetype membership, and `QueryState::iter_cached(world)` iterates that cached set while still fetching current component values and evaluating filters such as `Added<T>` and `Changed<T>` against the active tick window. Dynamic tick filters are intentionally rechecked during iteration rather than baked into the cached entity list. `World` owns a runtime-only query cache revision that advances on entity spawn/despawn and component presence add/remove, but not on value replacement. This means component replacement can reuse the cached entity list while still reading fresh component values from the world. The revision is ignored for `World` equality because it is runtime execution metadata, not scene data.

The second M11 slice adds the first Bevy-style archetype metadata beneath that cache. `World` now maintains an `ArchetypeIndex` keyed by `ArchetypeSignature`, and structural component insert/remove moves each live entity to the signature matching its table/sparse component set. `QueryAccess` now separates component reads/writes from required `With` membership, so `Option<&T>` participates in borrow access without falsely requiring `T` during archetype candidate selection. `QueryState::update_cache(world)` records the matched archetype ids and archetype generation before collecting candidate entity locations in scene order. It caches stable entity ids, `StableEntityLocation` records, and per-entity `ComponentStorageLocation` records for the accessed read/write components that are present. Cache rebuild now evaluates `QueryDataAccess::matches_component_locations(...)` against those locations before accepting a candidate entity, so required data (`&T`, `&mut T`, `Ref<T>`, and `Mut<T>`) can be checked from storage metadata while optional/entity/unit data stays non-structural. This gives later dense-iteration work internal entity ids plus component table rows/sparse-set addresses without changing query results.

The latest M11 slice connects those storage locations to an explicit read-only iteration path. `CachedQueryData` is implemented for the current read-only item shapes (`EntityId`, `StableEntityLocation`, `&T`, `Ref<T>`, `Option<&T>`, `()`, and tuples up to eight items), and `QueryState::iter_cached_direct(world)` returns `CachedQueryIter`. This iterator uses cached `StableEntityLocation` and `ComponentStorageLocation` entries plus the stale-safe `ComponentStorage::get_with_ticks_at_location(...)` helper, while still rechecking filters and required data before yielding an item. `StableEntityLocation` is Zircon's Bevy-inspired counterpart to Bevy's `EntityLocation` query data in `bevy_ecs/src/query/fetch.rs`: the normal query path resolves it from `World`, while cached-direct iteration returns the already cached location from `QueryState`. `CachedQueryData::matches_cached_data(...)` defaults to the same `QueryDataAccess::matches_component_locations(...)` predicate used by cache rebuilds; concrete direct-fetch implementations still use location reads for `&T`, `Ref<T>`, and optional `&T`. `CachedQueryFilter` keeps filter rechecks on the same cached-location path for the built-in filters: structural `With<T>` / `Without<T>` rely on the cache's archetype candidate set, while `Added<T>` / `Changed<T>` read ticks from cached component locations. `CachedQueryManyIter` extends the same direct-location path to caller-provided read-only entity lists. Mutable query data still fetches through the existing `fetch_mut` path, but `QueryState::for_each_mut(...)` and system `Query::for_each_mut(...)` now refresh and consume the persistent cached structural candidate entity list before rechecking dynamic filters, so mutable system queries no longer start from a full-world entity scan.

The tuple-arity slice expands `QueryData`, `QueryFilter`, `CachedQueryData`, and `CachedQueryFilter` tuple implementations from four to eight elements. Bevy uses `all_tuples!` to implement query data and query filters from 0 to 15 elements in `bevy_ecs/src/query/fetch.rs` and `bevy_ecs/src/query/filter.rs`; Zircon deliberately stops at eight in this slice to match the local system-param tuple ceiling while leaving derive-style query data/filter as the later Bevy-parity answer for wider shapes.

The read-only single-result slice adds `QuerySingleError::{NoEntities, MultipleEntities}` plus `QueryState::single(...)`, `QueryState::single_cached(...)`, `QueryState::single_cached_direct(...)`, `Query::single(...)`, and cached system variants. The implementation follows Bevy's `Query::single` / `single_inner` shape in `dev/bevy/crates/bevy_ecs/src/system/query.rs` and Bevy's `QuerySingleError` location in `dev/bevy/crates/bevy_ecs/src/query/error.rs`: Zircon peeks at the first two yielded items and succeeds only when exactly one entity matches. Zircon intentionally keeps the error variants payload-free for now because the local query API does not yet carry Bevy's debug query type names.

The targeted-get slice adds `QueryEntityError::{NotSpawned, QueryDoesNotMatch, AliasedMutability}` plus read-only `QueryState::get(...)`, `QueryState::get_cached(...)`, `QueryState::get_cached_direct(...)`, `Query::get(...)`, and cached system variants. This follows Bevy's `Query::get(entity)` and `QueryState::get(...)` shape in `dev/bevy/crates/bevy_ecs/src/system/query.rs` / `dev/bevy/crates/bevy_ecs/src/query/state.rs`, and Bevy's `QueryEntityError` location in `dev/bevy/crates/bevy_ecs/src/query/error.rs`. Zircon keeps the first mismatch shape intentionally small: `NotSpawned(entity)` means the stable scene id is not live, while `QueryDoesNotMatch(entity)` means the entity exists but lacks required data or is filtered out by `With<T>`, `Without<T>`, `Added<T>`, or `Changed<T>`.

The many-get slice adds read-only `QueryState::get_many(...)`, `QueryState::get_many_cached(...)`, `QueryState::get_many_cached_direct(...)`, `Query::get_many(...)`, `Query::get_many_cached(...)`, and `Query::get_many_cached_direct(...)`. It follows Bevy's array-returning `QueryState::get_many` in `dev/bevy/crates/bevy_ecs/src/query/state.rs` and system `Query::get_many` / `get_many_inner` in `dev/bevy/crates/bevy_ecs/src/system/query.rs`: the output order matches the input entity array, and read-only duplicate entities are allowed because no mutable alias can be created. Zircon's cached variants refresh the persistent candidate list once and then fetch every requested entity against the same tick window, so `Changed<T>` / `Added<T>` filters stay consistent across the array.

The read-only `iter_many` slice adds `QueryManyIter`, `QueryState::iter_many(...)`, and system `Query::iter_many(...)`. It follows Bevy's `Query::iter_many` / `QueryState::iter_many` APIs in `dev/bevy/crates/bevy_ecs/src/system/query.rs` and `dev/bevy/crates/bevy_ecs/src/query/state.rs`, with the core Bevy contract preserved: requested entities are visited in caller-provided order, duplicate read-only entity IDs can yield duplicate items, and missing or non-matching entities are skipped instead of reported as errors. `QueryEntityItem` accepts both owned `EntityId` values and borrowed `&EntityId` items, so callers can pass arrays, vectors, or borrowed entity lists without allocating an intermediate array.

The cached read-only `iter_many` slice adds `QueryState::iter_many_cached(...)` and system `Query::iter_many_cached(...)`. This is Zircon's explicit counterpart to Bevy's `QueryState::iter_many_manual(...)` shape: it refreshes the local structural cache once, filters the caller's entity list through `cached_entities`, then uses the same `QueryManyIter` fetch path to recheck dynamic filters such as `Added<T>` and `Changed<T>` against the active tick window. The resulting iterator still preserves input order and duplicate read-only IDs, but skips entities absent from the cached structural candidate set before fetching values. Unique-entity specializations remain a later slice because they need the same unique entity-set reasoning as Bevy's `iter_many_unique` families.

The cached-direct read-only `iter_many` slice adds `QueryState::iter_many_cached_direct(...)` and system `Query::iter_many_cached_direct(...)`. It uses the same caller-order and duplicate read-only semantics as `iter_many(...)`, but resolves the requested entity list to cached candidate indices once and then fetches each item through `CachedQueryManyIter` using cached stable/component locations. This keeps Zircon aligned with Bevy's `QueryState::iter_many_manual(...)` motivation of reusing already-updated query state, while intentionally diverging by exposing a named cached-direct path that can prove table/sparse location reads before default query iteration changes.

The mutable targeted-get slice adds `QueryState::get_mut(...)`, `QueryState::get_many_mut(...)`, `Query::get_mut(...)`, and `Query::get_many_mut(...)` for the current single-item `QueryMutData` surface. This follows Bevy's `Query::get_mut` and `Query::get_many_mut_inner` in `dev/bevy/crates/bevy_ecs/src/system/query.rs`, plus `QueryState::get_mut` / `get_many_mut` in `dev/bevy/crates/bevy_ecs/src/query/state.rs`. Zircon rejects duplicate stable entity ids before fetching mutable items and returns `QueryEntityError::AliasedMutability(entity)`, matching Bevy's aliasing diagnostic intent while staying narrower than Bevy's full `IterQueryData` surface. The focused regression tests extract expected `Err` values through an explicit helper instead of `unwrap_err()` because the `Ok` variant carries mutable world/query borrows; keeping that drop boundary explicit avoids compiler diagnostics obscuring the actual alias/mismatch assertions during profiling-profile lib-test builds.

The mutable `iter_many_mut` slice adds `QueryManyMutIter`, `QueryState::iter_many_mut(...)`, and system `Query::iter_many_mut(...)`. It follows Bevy's `Query::iter_many_mut` and `QueryManyIter::fetch_next()` contract in `dev/bevy/crates/bevy_ecs/src/system/query.rs` and `dev/bevy/crates/bevy_ecs/src/query/iter.rs`: requested entities are visited in caller order, missing or non-matching entities are skipped, and duplicate requested entity IDs are allowed only through a cursor-style `fetch_next()` API that ties the returned mutable item to the iterator borrow. Zircon deliberately does not implement `Iterator` for `QueryManyMutIter`; this keeps the aliasing boundary explicit while the local mutable query surface remains single-item `QueryMutData`. The iterator refreshes and clones the `QueryState` structural candidate list at creation, then rechecks required data and dynamic filters against the active tick window before each mutable fetch.

System `Query::iter_cached(...)` / `Query::single_cached(...)` provide the non-direct cached path for normal read-only `QueryData` over the active system tick window. This mirrors Bevy's `QueryState::iter(&mut self, world)` cache-refresh contract in `dev/bevy/crates/bevy_ecs/src/query/state.rs`, while staying explicit in Zircon because default system `Query::iter()` currently takes `&self`. The explicit cached methods take `&mut self`, refresh the persistent `QueryState`, and return a `QueryIter` over the cached candidate entity slice without changing the normal fetch semantics.

The count/empty/contains helper slices add `QueryState::count(...)`, `QueryState::is_empty(...)`, `QueryState::contains(...)`, cached normal variants, cached-direct variants, and matching system `Query` methods. This follows Bevy's system `Query::is_empty` / `Query::count` / `Query::contains` convenience API in `dev/bevy/crates/bevy_ecs/src/system/query.rs` and Bevy's tick-aware `QueryState::is_empty(world, last_run, this_run)` / `QueryState::contains(entity, world, last_run, this_run)` shape in `dev/bevy/crates/bevy_ecs/src/query/state.rs`. Zircon's cached helper variants deliberately keep the cache choice explicit: non-cached helpers use the normal query predicates, while `*_cached` and `*_cached_direct` refresh and reuse the persistent candidate list before still rechecking run-window filters such as `Changed<T>`. `contains(...)` first rejects missing scene entities, so optional-data queries such as `(EntityId, Option<&T>)` cannot accidentally match stale or nonexistent ids.

This is still a stepping stone, not the final M11 hot path. The runtime records table and sparse-set signatures plus cached stable/component locations, explicit cached system iteration avoids full-world scans, and the direct iterator can read table/sparse rows for read-only queries, but the default `QueryIter` path remains unchanged. The next performance slice can move more system-param query execution toward the cached iterator model and use the archetype component index to avoid map lookups during dense query iteration.

## System Params, Commands, And Change Detection

`SystemState<P>` is the local system execution wrapper. It caches `P::State`, records `SystemParamAccess`, tracks the system's `last_run` tick, advances `World` to a fresh `ChangeTick` for each run, and fetches `P::Item<'_>` for the user closure. Tuple params compose up to eight items and share one access descriptor so duplicate mutable resource, component, or event access is rejected before a system runs. A crate-private state accessor exists for regression tests that need to verify persistent query cache rebuild counts without exposing that state outside `zircon_runtime`.

Supported system params in this slice are:

- `QueryState<D, F>` as a param, yielding `Query<D, F>` with `iter()`, `iter_many(entities)`, `single()`, `get(entity)`, `get_many([entities])`, `count()`, `is_empty()`, `contains(entity)`, explicit read-only `iter_cached()` / `iter_many_cached(entities)` / `single_cached()` / `get_cached(entity)` / `get_many_cached([entities])` / `count_cached()` / `is_empty_cached()` / `contains_cached(entity)`, direct read-only `iter_cached_direct()` / `iter_many_cached_direct(entities)` / `single_cached_direct()` / `get_cached_direct(entity)` / `get_many_cached_direct([entities])` / `count_cached_direct()` / `is_empty_cached_direct()` / `contains_cached_direct(entity)`, and mutable `get_mut(entity)` / `get_many_mut([entities])` / `iter_many_mut(entities)` / cache-backed `for_each_mut(...)` over the current tick window. The cached normal, cached direct, and mutable system-query paths reuse the persistent `QueryState` stored inside `SystemState`, mirroring Bevy's separation between query state and each run's iterator cursor.
- `ResParam<T>` and `ResMutParam<T>`, yielding `Res<T>` and `ResMut<T>` wrappers with `Deref`, `DerefMut` for mutable access, and `is_added()` / `is_changed()` / `last_changed()` helpers.
- `Option<ResParam<T>>` and `Option<ResMutParam<T>>`, returning `None` when a resource is absent while still recording the declared access. Required `ResParam<T>` and `ResMutParam<T>` fail `SystemState::new(...)` with `SystemParamError::MissingResource` if the resource is missing.
- `CommandsParam`, yielding `Commands`, which queues deferred `spawn`, `spawn_empty`, `entity`, `entity_or_spawn`, `despawn`, component insert/remove, bundle insert, resource insert/remove, or custom `Command`/closure work.
- `EventReaderParam<T>` and `EventWriterParam<T>`, yielding current-queue readers and next-queue writers over runtime `EventStore`.
- `MessageReaderParam<T>` and `MessageWriterParam<T>`, yielding cursor-based scheduled message reads and immediate writes to runtime `MessageStore`.
- `RemovedComponentsParam<T>`, yielding a stateful `RemovedComponents<T>` reader that reports entities whose `T` was removed after the reader's last cursor.
- `ParamSet<(P0, P1, ...)>` for arities one through eight, allowing potentially conflicting params to be declared together while exposing only segmented `p0()` through `p7()` access.
- `LocalParam<T>`, yielding persistent local state initialized from `T::default()` and preserved across `SystemState::run(...)` calls.

`ParamSet` checks each child parameter against the access baseline from outside the set, then merges child access into the outer descriptor without preserving sibling query filters. This keeps sibling params allowed to conflict with each other while making the aggregate access conservative when another system compares against the whole `ParamSet`.

`SystemParamAccess::conflict_kinds_with(...)` is the M11 bridge from system parameters into schedule diagnostics. It reports component conflicts from `QueryAccess`, resource conflicts by `ResourceId`, and event/message conflicts by `TypeId` whenever two access descriptors share data and at least one side writes. Deferred commands are not treated as data conflicts in this graph; they continue to require explicit `ApplyDeferred` synchronization until a later scheduler slice adds automatic sync insertion.

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

`ScheduleConflictGraph` is the first local graph-analysis primitive for M11. Callers provide `ScheduleConflictNode` values containing a system id, `SystemStage`, and `SystemParamAccess`; the graph compares only systems in the same stage and emits `ScheduleConflictEdge` records with the left/right system ids, stage, and concrete `SystemParamConflictKind` entries. This follows Bevy's schedule-build separation between system access metadata and ambiguity/conflicting-system diagnostics, but it is currently diagnostic-only. It does not reorder systems or insert sync points.

`ScheduleConflictGraph::conservative_parallel_batches()` produces the first batch plan for later `JobScheduler` integration. It walks nodes in deterministic input order, groups adjacent same-stage systems into a `ScheduleParallelBatch` only when the recorded graph has no conflict between the new system and any system already in the last batch, and starts a new batch at every conflict or stage boundary. This is intentionally conservative: it preserves existing order semantics and avoids moving a later non-conflicting system ahead of an intervening conflicting system.

`ScheduleParallelExecutor` is the first M11 `JobScheduler` execution bridge. It consumes `ScheduleParallelBatch` values and a `ScheduleParallelTaskRegistry`, verifies that every system id in a batch has a registered task before starting that batch, runs single-task batches inline, and runs multi-task batches inside `JobScheduler::install(...)` using scoped Rayon tasks. Batch boundaries remain sequential: all tasks in one batch finish before the next batch starts, and task failures are reported in deterministic batch order through `ScheduleParallelExecutorError`. This executor is deliberately limited to independent registered closures; it does not yet run mutable `World` systems or plugin hooks in parallel.

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

The typed ECS system-param/change-detection tests cover deferred command visibility before/after `World::apply_deferred()`, entity command builder ordering, tuple system params up to eight items, tuple params combining query/resource/commands access, optional and required resource params, `ParamSet` segmented conflicting access up to eight items, event reader/writer current/next queues, persistent local params, `Ref<T>`/`Mut<T>` query wrappers, `Added<T>` windows, `Changed<T>` windows after direct mutable component access, cached and cached-direct system query iteration, cache-backed system mutable query iteration, single-result helpers, targeted get helpers, many-get helpers, read-only iter-many helpers including cached and cached-direct run-window variants, mutable iter-many helpers with fetch-next alias discipline and run-window filters, mutable get/many-get helpers with alias rejection, count/empty/contains helpers against run-window filters, removed-component readers, and explicit `ApplyDeferred` schedule flushing.

The M11 query-cache tests in `ecs_query.rs` verify that cached iteration is built once for an existing query, reuses the cache across component value replacement, rebuilds after a matching entity is spawned, rebuilds after a queried component is removed, keeps cached stable and component storage locations aligned with world entity/component rows, moves entity locations across archetype signatures on component add/remove, does not treat optional reads as required archetype membership, preserves write-access component locations during mutable-query cache rebuild, drives mutable `for_each_mut(...)` from the cached structural candidate list, supports five-plus query data/filter tuple shapes, can query `StableEntityLocation`, reports read-only single-query zero/one/many outcomes with `QuerySingleError`, exposes targeted get, many-get, and count/empty/contains helpers over cached and cached-direct candidates, adds normal/cached/cached-direct read-only iter-many helpers, preserves many-get and iter-many input order plus duplicate read-only entity requests, supports mutable targeted get and many-get while rejecting duplicate mutable entities, supports mutable iter-many with cursor-style duplicate entity requests, rejects nonexistent entities for get/contains checks, skips nonexistent entities during read-only and mutable iter-many, and can use `iter_cached_direct(...)` / `single_cached_direct(...)` for read-only `EntityId`, `StableEntityLocation`, `&T`, `Option<&T>`, and `Ref<T>` items over both table and sparse-set storage. The storage tests verify table-row location lookup, table swap-remove row updates, stale table-location rejection, sparse-set location reporting, direct table-row reads, and direct location reads with ticks. The M11 schedule-conflict tests in `ecs_schedule.rs` verify same-stage component read/write conflicts, `With<T>`/`Without<T>` disjointness, stage isolation, resource read/write conflicts, event/message write conflicts, concrete `SystemParamConflictKind` values, conflict lookup, conservative batch grouping, batch separation at stage boundaries, `JobScheduler` batch execution, missing-task diagnostics, and task-failure diagnostics.

The transform/active dirty-state M2 tests cover dirty-only mutators, direct read projection without clearing dirty bits, `PostUpdate` cache propagation, `RenderExtract` flushing after parent reorder and active changes, and mobility changes that dirty cached node/render state without forcing transform propagation.

The canonical render-extract M3 tests cover direct `RenderFrameExtract` section population from `World`, including request-driven camera aspect, visibility buckets, postprocess defaults, VG debug/default sidebands, and disabled Hybrid GI sidebands. A structure guard also rejects reintroducing `RenderFrameExtract::from_snapshot(...)` in the scene render-extract producer.

Historical focused evidence before unrelated active-lane blockers appeared: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked` passed with `3 passed; 0 failed; 1016 filtered out`, and `cargo test -p zircon_runtime --lib world_mutations_mark_derived_state_dirty_until_post_update_systems_flush --locked` passed for the dirty-only mutator regression.

Fresh 2026-05-08 support-fix evidence: `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api::runtime_only_typed_ecs_state_is_not_serialized --locked --target-dir target\codex-shared-a -- --nocapture` passed after direct serde deserialization began rebuilding runtime-only ECS identity/presence. The package-level `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime` also passed Cargo build and Cargo test.

Fresh 2026-05-08 M1 render-profile support evidence: `cargo check -p zircon_runtime --lib --locked` and `cargo check -p zircon_app --locked --all-targets` both finished successfully with warning-only output after the query marker lifetime bound was narrowed to the access descriptor layer. The production ECS query module tree and public `World` query entrypoints compile.

Fresh 2026-05-08 M2 boundary evidence passed after active asset importer and UI focus compile drift were cut to their current contracts. `cargo test -p zircon_runtime --lib scene::tests::ecs_schedule::render_extract_prepare_flushes_parent_reorder_and_active_changes --locked --message-format short` passed with `1 passed; 0 failed; 1061 filtered out`. `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short` passed with `45 passed; 0 failed; 1018 filtered out`. `cargo test -p zircon_runtime --lib graphics::tests --locked --message-format short` passed with `107 passed; 0 failed; 956 filtered out`.

Fresh 2026-05-08 M3 boundary evidence passed in `E:\cargo-targets\zircon-ecs-render-m3` after the direct extract test fixture was aligned with the shared static-mobility mutation guard. The focused direct `RenderFrameExtract` population test, structural snapshot-adapter guard, and scene-produced M5 flagship sideband test each passed with `1 passed; 0 failed; 1070 filtered out`. The broader `scene::tests` filter passed with `47 passed; 0 failed; 1024 filtered out`, and `graphics::tests` passed with `108 passed; 0 failed; 963 filtered out`.

Fresh 2026-05-08 SystemParam/Commands/change-detection evidence: `cargo check -p zircon_runtime --lib --locked --message-format short` finished successfully with warning-only output. `cargo test -p zircon_runtime --lib scene::tests::ecs_systems --locked --message-format short` passed with `4 passed; 0 failed; 1080 filtered out`. `cargo test -p zircon_runtime --lib scene::tests::ecs_query --locked --message-format short` passed with `5 passed; 0 failed; 1079 filtered out`. `cargo test -p zircon_runtime --lib scene::tests::ecs_typed_api --locked --message-format short` passed with `4 passed; 0 failed; 1080 filtered out`. Broader scene validation with `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short` passed with `51 passed; 0 failed; 1033 filtered out`.

Fresh focused evidence for this document is recorded in `tests/acceptance/ecs-to-render-chain.md`.

Fresh 2026-05-16 M6 observer/message implementation evidence: `cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir target/codex-ecs-m6 --message-format short` passed through a hidden background Cargo process with `EXIT=0`. The focused `cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir target/codex-ecs-m6 --message-format short` test-configuration check also passed with `EXIT=0`. The first isolated focused test attempts were interrupted with process exit `-1` while compiling third-party dependencies, before `zircon_runtime` test diagnostics executed. After reusing the warmed default target, `cargo test -p zircon_runtime --lib scene::tests::ecs_observers_messages --locked --jobs 1 --message-format short` passed with `4 passed; 0 failed; 1399 filtered out`.

Fresh 2026-05-20 M11 archetype-cache implementation evidence: `rustfmt --check` passed for the touched ECS/world/test Rust files, and `git diff --check` passed with line-ending warnings only. The focused `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-ecs-m11-archetype-coremin --message-format short --color never` reached `zircon_runtime` but is currently blocked by active asset-lane edits adding `ResourceKind::Mesh`: `zircon_runtime/src/asset/artifact/store.rs` and `zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs` have non-exhaustive matches for that new kind. No ECS M11 diagnostics appeared before that unrelated blocker.

Fresh 2026-05-20 M11 schedule-conflict/executor implementation evidence: scoped `rustfmt --check` passed for `QueryAccess`, `SystemParamAccess`, `ScheduleConflictGraph`, `ScheduleParallelBatch`, `ScheduleParallelExecutor`, ECS exports, and `ecs_schedule.rs`; scoped `git diff --check` passed with line-ending warnings only. Focused `cargo test -p zircon_runtime schedule_conflict_graph --no-run` and lightweight `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-ecs-m11-schedule-conflicts --message-format short --color never` attempts both exceeded the local wait window while many unrelated Cargo jobs were already compiling in parallel, before producing Zircon ECS diagnostics. The follow-up executor-focused `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-ecs-m11-parallel-executor --message-format short --color never` attempt also exceeded the local wait window under the same parallel-build load before producing Zircon ECS diagnostics.

Fresh 2026-05-20 M11 cached-location implementation evidence: scoped `rustfmt --check` passed for the touched query/world/test Rust files, and scoped `git diff --check` passed with line-ending warnings only. Cargo validation remains deferred because the local machine was still running many unrelated Cargo/rustc jobs and the preceding M11 executor check had just exceeded the wait window before producing Zircon ECS diagnostics.

Fresh 2026-05-20 M11 component-storage-location implementation evidence: scoped `rustfmt --check` passed for the touched storage/query/world/test Rust files, and scoped `git diff --check` passed with line-ending warnings only. Cargo validation remains deferred while unrelated Cargo/rustc jobs are active; this slice only adds read-only storage metadata and tests, and does not change `QueryIter` fetch semantics.

Fresh 2026-05-20 M11 location-read implementation evidence: scoped `rustfmt --check` passed for `component_storage.rs` and `ecs_identity_storage.rs`, and scoped `git diff --check` passed with line-ending warnings only. Cargo validation remains deferred while unrelated Cargo/rustc jobs are active; this slice adds stale-safe read helpers but does not change query iteration semantics.

Fresh 2026-05-20 M11 cached-direct-query implementation evidence: scoped `rustfmt --check` passed for `cached_query_iter.rs`, query exports/state, ECS exports, `world/query.rs`, and `ecs_query.rs`; scoped `git diff --check` passed for the touched ECS/query/world/test/doc files with line-ending warnings only. A low-concurrency `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-ecs-m11-cached-direct --message-format short --color never` attempt exceeded the 4-minute local wait window while many unrelated Cargo/rustc jobs were already active, before producing Zircon ECS diagnostics.

Fresh 2026-05-20 M11 system-query cached-direct wiring evidence: scoped `rustfmt --check` and scoped `git diff --check` passed for `system/query.rs`, `query_state.rs`, `ecs_systems.rs`, and this document after wiring `Query` to its persistent `QueryState`. A low-concurrency `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-ecs-m11-system-query-cached-direct --message-format short --color never` attempt exceeded the 4-minute local wait window while unrelated Cargo/rustc jobs were active, before producing Zircon ECS diagnostics; no `zircon-ecs-m11-system-query-cached-direct` cargo/rustc process remained afterward.

Fresh 2026-05-20 M11 sparse cached-direct query evidence: scoped `rustfmt` and scoped `git diff --check` passed for `ecs_query.rs` after adding sparse-set direct cached-location coverage. Cargo validation remains pending the shared Cargo/rustc queue described above.

Fresh 2026-05-20 M11 cached-filter direct query evidence: scoped `rustfmt --check` passed for `cached_query_iter.rs`, query exports/state, ECS exports, and `system/query.rs`; scoped `git diff --check` passed for the tracked touched files with line-ending warnings only; and a trailing-whitespace scan passed for the untracked `cached_query_iter.rs` file. Cargo validation is deferred because unrelated Hub/plugin/runtime Cargo jobs were active again before the lightweight gate could run without adding more compile load.

Fresh 2026-05-20 M11 cached-data direct query evidence: scoped `rustfmt` passed for `cached_query_iter.rs` after adding `CachedQueryData::matches_cached_data(...)`; scoped `git diff --check` and the untracked-file trailing-whitespace scan remained clean for the touched query/doc paths. Cargo validation remains deferred while unrelated Hub/plugin/runtime Cargo jobs are active.

Fresh 2026-05-20 M11 cache-rebuild location-data evidence: scoped `rustfmt --check` passed for `query_data.rs`, `query_state.rs`, `cached_query_iter.rs`, and `ecs_query.rs`; scoped `git diff --check` passed for the tracked touched query/test/doc files with line-ending warnings only; and the untracked `cached_query_iter.rs` trailing-whitespace scan stayed clean. Cargo validation remains deferred because many unrelated Cargo/rustc jobs were active before this slice reached validation, so adding another `zircon_runtime` compile would contend with parallel sessions.

Fresh 2026-05-20 M11 query tuple-arity evidence: scoped `rustfmt --check` passed for `query_data.rs`, `query_filter.rs`, `cached_query_iter.rs`, and `ecs_query.rs`; scoped `git diff --check` passed for the tracked touched query/test/doc files with line-ending warnings only; and the untracked `cached_query_iter.rs` trailing-whitespace scan stayed clean. Cargo validation remains deferred because unrelated Cargo/rustc jobs were still active during this slice.

Fresh 2026-05-20 M11 stable-location query-data evidence: scoped `rustfmt --check` passed for `query_data.rs`, `query_state.rs`, `cached_query_iter.rs`, and `ecs_query.rs`; scoped `git diff --check` passed for the tracked touched query/test/doc files with line-ending warnings only; and the untracked `cached_query_iter.rs` trailing-whitespace scan stayed clean. A repository scan found all `fetch_cached(...)` implementations live in `cached_query_iter.rs` after the signature change. Cargo validation remains deferred because unrelated Cargo/rustc jobs were active and increasing during validation.

Fresh 2026-05-20 M11 mutable-query cache evidence: scoped `rustfmt --check` passed for `query_state.rs` and `ecs_query.rs`; scoped `git diff --check` passed for the tracked touched query/test/doc files with line-ending warnings only. Cargo validation remains deferred because unrelated Cargo/rustc jobs were active throughout validation.

Fresh 2026-05-20 M11 read-only query single evidence: scoped `rustfmt --check` passed for the touched query/system/test Rust files, scoped `git diff --check` passed for tracked touched files with line-ending warnings only, and trailing-whitespace scans passed for untracked query files. Cargo validation remains deferred while unrelated Cargo/rustc jobs are active; this slice only adds read-only iterator adapters and tests.

Fresh 2026-05-20 M11 system mutable-query cache evidence: scoped `rustfmt --check` passed for `query_state.rs`, `system/query.rs`, `system_state.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the tracked touched query/system/test/doc files with line-ending warnings only. Cargo validation remains deferred while unrelated Cargo/rustc jobs are active; this slice reuses existing mutable fetch semantics and changes only the system query candidate source.

Fresh 2026-05-20 M11 system cached-iteration evidence: scoped `rustfmt --check` passed for `query_state.rs`, `system/query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the tracked touched query/system/test/doc files with line-ending warnings only. Cargo validation remains deferred while unrelated Cargo/rustc jobs are active; this slice adds explicit `&mut Query` cached read-only iteration and leaves default `Query::iter()` unchanged.

Fresh 2026-05-20 M11 query count/empty helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the tracked touched query/system/test/doc files with line-ending warnings only; and trailing-whitespace scans passed for untracked query/session files. Cargo validation remains deferred because 15 unrelated Cargo/rustc/rustdoc processes were still active during closeout; this slice adds helper adapters over existing normal/cached/direct iterators.

Fresh 2026-05-20 M11 query contains helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the tracked touched query/system/test/doc files with line-ending warnings only; and trailing-whitespace scans passed for untracked query/session files. Cargo validation remains deferred because 18 unrelated Cargo/rustc/rustdoc processes were active during closeout; this slice adds entity-membership helpers over existing normal/cached/direct query predicates and keeps `Changed<T>` run-window checks active.

Fresh 2026-05-20 M11 query get helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_entity_error.rs`, query/ECS exports, `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for tracked touched query/system/test/doc files with line-ending warnings only; and trailing-whitespace scans passed for untracked query/session files. Cargo validation remains deferred because unrelated Cargo/rustc/rustdoc jobs were still active during closeout; this slice adds read-only targeted get helpers and a local `QueryEntityError`, while deferring mutable get and many-get alias diagnostics.

Fresh 2026-05-20 M11 query get-many helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the touched query/system/test/doc files with line-ending warnings only; and a trailing-whitespace scan passed for the touched Rust/doc/session files. Cargo validation remains deferred because 18 unrelated Cargo/rustc/rustdoc jobs were active during validation; this slice adds read-only `get_many` helpers for normal, cached, and cached-direct query paths, preserves input order, permits duplicate read-only entities, and keeps run-window filters active.

Fresh 2026-05-20 M11 query get-mut helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_entity_error.rs`, `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the touched query/system/test/doc files with line-ending warnings only; and a trailing-whitespace scan passed for the touched Rust/doc/session files. Cargo validation remains deferred because 13 unrelated Cargo/rustc/rustdoc jobs were active during validation; this slice adds mutable `get_mut` / `get_many_mut` helpers and `AliasedMutability` diagnostics for duplicate mutable stable ids.

Fresh 2026-05-21 M11 mutable-query diagnostic evidence: the mutable targeted-get tests now avoid `unwrap_err()` on `Result` values whose `Ok` branch carries mutable world/query borrows, and focused WSL validation passed for both lower-layer regressions. `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-ecs-query-ice-debug-wsl cargo test -p zircon_runtime --lib scene::tests::ecs_query::query_state_get_mut_helpers_mutate_targets_and_reject_aliases --no-default-features --features core-min --locked --message-format=short --jobs 1 -- --exact --test-threads=1 --nocapture` passed with `1 passed; 0 failed; 1745 filtered out`. `CARGO_TARGET_DIR=/mnt/d/cargo-targets/zircon-ecs-query-ice-debug-wsl cargo test -p zircon_runtime --lib scene::tests::ecs_systems::system_query_get_mut_helpers_mutate_targets_and_reject_aliases --no-default-features --features core-min --locked --message-format=short --jobs 1 -- --exact --test-threads=1 --nocapture` also passed with `1 passed; 0 failed; 1745 filtered out`. Both runs emitted only existing unused-code warnings outside the mutable-query helper behavior.

Fresh 2026-05-20 M11 query iter-many helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_many_iter.rs`, query/ECS exports, `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the touched query/system/test/doc files with line-ending warnings only; and a trailing-whitespace scan passed for the touched Rust/doc/session files. Cargo validation remains deferred because the unrelated Cargo/rustc/rustdoc queue grew from 9 to 17 jobs during validation; this slice adds read-only `iter_many` helpers that skip missing/non-matching entities while preserving requested order and duplicate read-only entity ids.

Fresh 2026-05-20 M11 query cached iter-many helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_many_iter.rs`, `query_state.rs`, `system/query.rs`, `ecs_query.rs`, and `ecs_systems.rs`; scoped `git diff --check` passed for the touched query/system/test/doc/session files with line-ending warnings only; and a trailing-whitespace scan passed for the touched Rust/doc/session files. Cargo validation remains deferred because unrelated Cargo/rustc/rustdoc jobs stayed active throughout final validation, so this slice did not start another Cargo job; this slice adds cached read-only `iter_many_cached` helpers that first filter requested entities through the persistent `QueryState` candidate list and then recheck run-window filters.

Fresh 2026-05-21 M11 query mutable iter-many helper evidence: scoped `rustfmt --edition 2021 --check` passed for `query_many_mut_iter.rs`, query/ECS exports, `query_state.rs`, `system/query.rs`, and `ecs_query.rs`; scoped `git diff --check` passed for the touched query/system/test/doc/session files with line-ending warnings only; and a trailing-whitespace scan passed for the touched Rust/doc/session files. Cargo validation remains deferred because unrelated Cargo/rustc/rustdoc jobs were active during lightweight validation, so this slice did not start another Cargo job; this slice adds `QueryManyMutIter::fetch_next()` plus `QueryState::iter_many_mut(...)` and system `Query::iter_many_mut(...)` for cache-backed mutable targeted iteration.

Fresh 2026-05-21 M11 query cached-direct iter-many helper evidence: scoped `rustfmt --edition 2021 --check` passed for `cached_query_iter.rs`, query/ECS exports, `query_state.rs`, `system/query.rs`, and `ecs_query.rs`; scoped `git diff --check` passed for the touched query/system/test/doc/session files with line-ending warnings only; and a trailing-whitespace scan passed for the touched Rust/doc/session files. Cargo validation remains deferred because unrelated Cargo/rustc/rustdoc jobs were active during lightweight validation, so this slice did not start another Cargo job; this slice adds `CachedQueryManyIter` plus `QueryState::iter_many_cached_direct(...)` and system `Query::iter_many_cached_direct(...)` for read-only targeted iteration over cached storage locations.
