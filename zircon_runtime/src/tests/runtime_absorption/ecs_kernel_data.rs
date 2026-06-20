use std::path::Path;

const EXPECTED_RUNTIME_08_SOURCE_FILES: &[&str] = &[
    "src/scene/ecs/archetype/mod.rs",
    "src/scene/ecs/archetype/id.rs",
    "src/scene/ecs/archetype/index.rs",
    "src/scene/ecs/archetype/move_result.rs",
    "src/scene/ecs/archetype/record.rs",
    "src/scene/ecs/archetype/signature.rs",
    "src/scene/ecs/bundle.rs",
    "src/scene/ecs/storage_type.rs",
    "src/scene/ecs/storage/component_storage/mod.rs",
    "src/scene/ecs/storage/component_storage/entry.rs",
    "src/scene/ecs/storage/component_storage/location.rs",
    "src/scene/ecs/storage/component_storage/sparse.rs",
    "src/scene/ecs/storage/component_storage/store.rs",
    "src/scene/ecs/storage/component_storage/table.rs",
    "src/scene/ecs/storage/component_storage/utils.rs",
    "src/scene/ecs/component/mod.rs",
    "src/scene/ecs/component/id.rs",
    "src/scene/ecs/component/marker.rs",
    "src/scene/ecs/component/registry.rs",
    "src/scene/ecs/entity/mod.rs",
    "src/scene/ecs/entity/despawned.rs",
    "src/scene/ecs/entity/error.rs",
    "src/scene/ecs/entity/internal.rs",
    "src/scene/ecs/entity/location.rs",
    "src/scene/ecs/entity/registry.rs",
    "src/scene/ecs/entity/slot.rs",
    "src/scene/ecs/entity/stable_location.rs",
    "src/scene/ecs/observer/mod.rs",
    "src/scene/ecs/observer/callbacks.rs",
    "src/scene/ecs/observer/entry.rs",
    "src/scene/ecs/observer/id.rs",
    "src/scene/ecs/observer/store.rs",
    "src/scene/ecs/observer/utils.rs",
    "src/scene/ecs/commands/command.rs",
    "src/scene/ecs/commands/command_queue.rs",
    "src/scene/ecs/commands/commands/mod.rs",
    "src/scene/ecs/commands/commands/entity_commands.rs",
    "src/scene/ecs/commands/commands/facade.rs",
    "src/scene/ecs/commands/commands/param.rs",
    "src/scene/ecs/events/mod.rs",
    "src/scene/ecs/events/cursor.rs",
    "src/scene/ecs/events/id.rs",
    "src/scene/ecs/events/metrics.rs",
    "src/scene/ecs/events/queue.rs",
    "src/scene/ecs/events/store.rs",
    "src/scene/ecs/events/subscription.rs",
    "src/scene/ecs/messages/mod.rs",
    "src/scene/ecs/messages/cursor.rs",
    "src/scene/ecs/messages/id.rs",
    "src/scene/ecs/messages/queue.rs",
    "src/scene/ecs/messages/store.rs",
    "src/scene/ecs/resource/mod.rs",
    "src/scene/ecs/resource/id.rs",
    "src/scene/ecs/resource/marker.rs",
    "src/scene/ecs/resource/registry.rs",
    "src/scene/ecs/resource_store/mod.rs",
    "src/scene/ecs/resource_store/stored_resource.rs",
    "src/scene/ecs/resource_store/store.rs",
    "src/scene/ecs/change_detection/mod.rs",
    "src/scene/ecs/change_detection/change_tick.rs",
    "src/scene/ecs/change_detection/change_tick_window.rs",
    "src/scene/ecs/change_detection/component_ticks.rs",
    "src/scene/ecs/change_detection/stats.rs",
    "src/scene/ecs/change_detection/wrappers.rs",
    "src/scene/ecs/removal.rs",
    "src/scene/world/identity.rs",
    "src/scene/world/observers.rs",
    "src/scene/world/events.rs",
    "src/scene/world/messages.rs",
];
const EXPECTED_RUNTIME_08_TEST_FILES: &[&str] = &[
    "src/scene/tests/ecs_identity_storage.rs",
    "src/scene/tests/ecs_observers_messages.rs",
    "src/scene/tests/ecs_commands.rs",
    "src/scene/tests/ecs_events_messages.rs",
    "src/scene/tests/ecs_change_detection.rs",
    "src/scene/tests/component_structure/runtime_08_owner_tree.rs",
    "src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
    "src/tests/runtime_absorption/ecs_kernel_data.rs",
];
const EXPECTED_RUNTIME_08_BEHAVIOR_TEST_ANCHORS: &[&str] = &[
    "despawned_entity_handle_is_rejected_by_world_access",
    "entity_id_reuse_does_not_alias_previous_generation_handle",
    "stable_entity_location_survives_archetype_move_and_invalidates_on_despawn",
    "component_removal_emits_removal_record_in_same_frame",
    "lifecycle_observer_fires_immediately_during_component_mutation",
    "entity_event_observer_only_fires_for_target_entity",
    "observer_remove_during_dispatch_does_not_skip_or_double_fire",
    "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
    "deferred_command_success_report_counts_applied_commands_without_errors",
    "events_require_explicit_update_and_keep_next_queue_hidden",
    "first_stage_updates_all_registered_event_channels",
    "clear_events_prunes_current_and_next_event_queues",
    "messages_are_retained_until_explicit_clear_independent_of_event_updates",
    "event_and_message_clear_boundaries_do_not_cross_channels",
    "change_tick_comparison_survives_wraparound",
    "tick_window_clamps_stale_ticks",
];

#[test]
fn runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_08_SOURCE_FILES.len(), 69);
    assert_eq!(EXPECTED_RUNTIME_08_TEST_FILES.len(), 8);
    assert_eq!(EXPECTED_RUNTIME_08_BEHAVIOR_TEST_ANCHORS.len(), 16);

    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert_files_exist(
        runtime_root,
        EXPECTED_RUNTIME_08_SOURCE_FILES,
        "Runtime 08 ECS data-kernel source",
    );
    assert_files_exist(
        runtime_root,
        EXPECTED_RUNTIME_08_TEST_FILES,
        "Runtime 08 ECS data-kernel guard/test",
    );

    assert_source_anchors(
        "Runtime 08 archetype",
        &[
            include_str!("../../scene/ecs/archetype/mod.rs"),
            include_str!("../../scene/ecs/archetype/id.rs"),
            include_str!("../../scene/ecs/archetype/index.rs"),
            include_str!("../../scene/ecs/archetype/move_result.rs"),
            include_str!("../../scene/ecs/archetype/record.rs"),
            include_str!("../../scene/ecs/archetype/signature.rs"),
        ],
        &[
            "pub struct ArchetypeId(usize)",
            "pub const EMPTY: Self = Self(0);",
            "pub struct ArchetypeRecord",
            "pub(super) fn push_entity(&mut self, entity: EntityId) -> usize",
            "pub(super) fn swap_remove_entity(",
            "pub struct ArchetypeMove",
            "pub struct ArchetypeIndex",
            "by_signature: HashMap<ArchetypeSignature, ArchetypeId>",
            "by_component: HashMap<ComponentId, Vec<ArchetypeId>>",
            "pub fn matching_archetypes(",
            "fn shortest_required_archetype_ids(&self, required: &[ComponentId])",
            "fn insert_archetype_id(ids: &mut Vec<ArchetypeId>, id: ArchetypeId)",
            "fn entity_row(entities: &[EntityId], entity: EntityId) -> Option<usize>",
            "pub struct ArchetypeSignature",
            "fn normalize_components(mut components: Vec<ComponentId>)",
        ],
    );
    assert_source_anchors(
        "Runtime 08 storage",
        &[
            include_str!("../../scene/ecs/storage_type.rs"),
            include_str!("../../scene/ecs/storage/component_storage/mod.rs"),
            include_str!("../../scene/ecs/storage/component_storage/entry.rs"),
            include_str!("../../scene/ecs/storage/component_storage/location.rs"),
            include_str!("../../scene/ecs/storage/component_storage/sparse.rs"),
            include_str!("../../scene/ecs/storage/component_storage/store.rs"),
            include_str!("../../scene/ecs/storage/component_storage/table.rs"),
            include_str!("../../scene/ecs/storage/component_storage/utils.rs"),
        ],
        &[
            "pub enum StorageType",
            "Table,",
            "SparseSet,",
            "pub struct ComponentStorage",
            "table_components: HashMap<ComponentId, TableComponentStorage>,",
            "sparse_components: HashMap<ComponentId, SparseComponentStorage>,",
            "pub struct ComponentStorageLocation",
            "pub fn get_table_row<T>",
            "pub fn get_with_ticks_at_location<T>",
        ],
    );
    assert_component_storage_private_reexport_cleanup();
    assert_source_anchors(
        "Runtime 08 component identity",
        &[
            include_str!("../../scene/ecs/component/mod.rs"),
            include_str!("../../scene/ecs/component/id.rs"),
            include_str!("../../scene/ecs/component/marker.rs"),
            include_str!("../../scene/ecs/component/registry.rs"),
        ],
        &[
            "pub trait Component: 'static + Send + Sync",
            "const STORAGE_TYPE: StorageType = StorageType::Table;",
            "pub struct ComponentId(usize)",
            "pub const fn new(index: usize) -> Self",
            "pub const fn index(self) -> usize",
            "pub struct ComponentDescriptor",
            "pub enum ComponentDescriptorSource",
            "RustType { type_id: TypeId }",
            "DynamicPlugin { component_type_id: String }",
            "pub struct ComponentRegistry",
            "rust_ids_by_type_id: HashMap<TypeId, ComponentId>",
            "dynamic_ids_by_type_id: HashMap<String, ComponentId>",
            "pub fn component_id<T>(&mut self) -> ComponentId",
            "pub fn dynamic_component_id(&mut self, component_type_id: &str) -> ComponentId",
            "pub fn registered_component_id<T>(&self) -> Option<ComponentId>",
            "pub fn registered_dynamic_component_id(&self, component_type_id: &str) -> Option<ComponentId>",
            "pub(crate) fn rust_type_for_id(&self, id: ComponentId) -> Option<(TypeId, &str)>",
            "pub fn descriptors(&self) -> &[ComponentDescriptor]",
        ],
    );
    assert_source_anchors(
        "Runtime 08 entity lifecycle",
        &[
            include_str!("../../scene/ecs/entity/mod.rs"),
            include_str!("../../scene/ecs/entity/despawned.rs"),
            include_str!("../../scene/ecs/entity/error.rs"),
            include_str!("../../scene/ecs/entity/internal.rs"),
            include_str!("../../scene/ecs/entity/location.rs"),
            include_str!("../../scene/ecs/entity/registry.rs"),
            include_str!("../../scene/ecs/entity/slot.rs"),
            include_str!("../../scene/ecs/entity/stable_location.rs"),
            include_str!("../../scene/world/identity.rs"),
        ],
        &[
            "const FIRST_GENERATION: u32 = 1;",
            "free_slots: Vec<u32>",
            "stable_to_internal: HashMap<EntityId, InternalEntity>",
            "InternalEntity::new(slot_index, slot.generation)",
            "slot.generation = next_generation(slot.generation);",
            "self.free_slots.push(internal.index());",
            "pub const fn generation(self) -> u32",
            "self.index as u64 | ((self.generation as u64) << 32)",
            "pub struct StableEntityLocation",
            "pub struct DespawnedEntity",
        ],
    );
    assert_source_anchors(
        "Runtime 08 observer",
        &[
            include_str!("../../scene/ecs/observer/mod.rs"),
            include_str!("../../scene/ecs/observer/callbacks.rs"),
            include_str!("../../scene/ecs/observer/entry.rs"),
            include_str!("../../scene/ecs/observer/id.rs"),
            include_str!("../../scene/ecs/observer/store.rs"),
            include_str!("../../scene/ecs/observer/utils.rs"),
            include_str!("../../scene/world/observers.rs"),
        ],
        &[
            "pub struct ObserverStore",
            "pub fn observe_lifecycle(",
            "pub fn observe_event<E>(",
            "pub fn observe_entity_event<E>(",
            "pub fn remove(&mut self, id: ObserverId) -> bool",
            "pub(crate) fn lifecycle_callbacks(",
            "let mut callbacks = Vec::with_capacity(callback_count);",
            "callbacks.push(observer.callback.clone());",
        ],
    );
    assert_source_anchors(
        "Runtime 08 deferred command",
        &[
            include_str!("../../scene/ecs/commands/command.rs"),
            include_str!("../../scene/ecs/commands/command_queue.rs"),
            include_str!("../../scene/ecs/commands/commands/mod.rs"),
            include_str!("../../scene/ecs/commands/commands/entity_commands.rs"),
            include_str!("../../scene/ecs/commands/commands/facade.rs"),
            include_str!("../../scene/ecs/commands/commands/param.rs"),
            include_str!("../../scene/world/commands.rs"),
        ],
        &[
            "pub enum DeferredCommandOperation",
            "pub struct DeferredCommandError",
            "pub struct DeferredCommandReport",
            "pub fn errors(&self) -> &[DeferredCommandError]",
            "pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport",
            "world.record_deferred_command_error(DeferredCommandError::new(",
            "DeferredCommandOperation::Despawn",
            "DeferredCommandOperation::Insert",
            "DeferredCommandOperation::Remove",
            "pub fn apply_deferred(&mut self) -> DeferredCommandReport",
            "std::mem::take(&mut self.deferred_command_errors)",
        ],
    );
    assert_source_anchors(
        "Runtime 08 event/message",
        &[
            include_str!("../../scene/ecs/events/mod.rs"),
            include_str!("../../scene/ecs/events/cursor.rs"),
            include_str!("../../scene/ecs/events/id.rs"),
            include_str!("../../scene/ecs/events/metrics.rs"),
            include_str!("../../scene/ecs/events/queue.rs"),
            include_str!("../../scene/ecs/events/store.rs"),
            include_str!("../../scene/ecs/events/subscription.rs"),
            include_str!("../../scene/ecs/messages/mod.rs"),
            include_str!("../../scene/ecs/messages/cursor.rs"),
            include_str!("../../scene/ecs/messages/id.rs"),
            include_str!("../../scene/ecs/messages/queue.rs"),
            include_str!("../../scene/ecs/messages/store.rs"),
            include_str!("../../scene/world/events.rs"),
            include_str!("../../scene/world/messages.rs"),
        ],
        &[
            "pub struct Events<T>",
            "current: Vec<T>",
            "next: Vec<T>",
            "pub fn update(&mut self)",
            "std::mem::swap(&mut self.current, &mut self.next);",
            "self.current.clear();",
            "self.next.clear();",
            "pub fn update_all(&mut self)",
            "pub struct MessageId<T>",
            "pub struct Messages<T>",
            "next_id: usize",
            "pub fn clear(&mut self)",
        ],
    );
    assert_source_anchors(
        "Runtime 08 resource identity",
        &[
            include_str!("../../scene/ecs/resource/mod.rs"),
            include_str!("../../scene/ecs/resource/id.rs"),
            include_str!("../../scene/ecs/resource/marker.rs"),
            include_str!("../../scene/ecs/resource/registry.rs"),
        ],
        &[
            "pub trait Resource: 'static + Send + Sync",
            "pub struct ResourceId(usize)",
            "pub const fn new(index: usize) -> Self",
            "pub const fn index(self) -> usize",
            "pub struct ResourceDescriptor",
            "pub struct ResourceRegistry",
            "ids_by_type: HashMap<TypeId, ResourceId>",
            "pub fn resource_id<T>(&mut self) -> ResourceId",
            "type_name::<T>().to_string()",
            "pub fn registered_resource_id<T>(&self) -> Option<ResourceId>",
            "pub fn descriptor(&self, id: ResourceId) -> Option<&ResourceDescriptor>",
            "pub fn descriptors(&self) -> &[ResourceDescriptor]",
        ],
    );
    assert_source_anchors(
        "Runtime 08 change tick",
        &[
            include_str!("../../scene/ecs/change_detection/mod.rs"),
            include_str!("../../scene/ecs/change_detection/change_tick.rs"),
            include_str!("../../scene/ecs/change_detection/change_tick_window.rs"),
            include_str!("../../scene/ecs/change_detection/component_ticks.rs"),
            include_str!("../../scene/ecs/change_detection/stats.rs"),
            include_str!("../../scene/ecs/change_detection/wrappers.rs"),
        ],
        &[
            "pub const MAX_CHANGE_AGE: u64",
            "Self(self.0.wrapping_add(1))",
            "Self(self.0.wrapping_sub(older.0))",
            "pub fn is_newer_than(self, last_run: Self, this_run: Self) -> bool",
            "this_run.relative_to(self).0.min(Self::MAX_CHANGE_AGE)",
            "last_run: last_run.clamp_older_than(this_run)",
        ],
    );

    assert_source_anchors(
        "Runtime 08 guard/test",
        &[
            include_str!("../../scene/tests/ecs_identity_storage.rs"),
            include_str!("../../scene/tests/ecs_observers_messages.rs"),
            include_str!("../../scene/tests/ecs_commands.rs"),
            include_str!("../../scene/tests/ecs_events_messages.rs"),
            include_str!("../../scene/tests/ecs_change_detection.rs"),
            include_str!("../../scene/tests/component_structure/runtime_08_owner_tree.rs"),
            include_str!("plan_status/cargo_gates/early.rs"),
            include_str!("ecs_kernel_data.rs"),
        ],
        &[
            "despawned_entity_handle_is_rejected_by_world_access",
            "entity_id_reuse_does_not_alias_previous_generation_handle",
            "stable_entity_location_survives_archetype_move_and_invalidates_on_despawn",
            "component_removal_emits_removal_record_in_same_frame",
            "lifecycle_observer_fires_immediately_during_component_mutation",
            "entity_event_observer_only_fires_for_target_entity",
            "observer_remove_during_dispatch_does_not_skip_or_double_fire",
            "command_queue_on_despawned_entity_target_is_reported_not_silently_dropped",
            "deferred_command_success_report_counts_applied_commands_without_errors",
            "events_require_explicit_update_and_keep_next_queue_hidden",
            "first_stage_updates_all_registered_event_channels",
            "clear_events_prunes_current_and_next_event_queues",
            "messages_are_retained_until_explicit_clear_independent_of_event_updates",
            "event_and_message_clear_boundaries_do_not_cross_channels",
            "change_tick_comparison_survives_wraparound",
            "tick_window_clamps_stale_ticks",
            "runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover",
            "runtime_08_ecs_change_detection_owner_tree_stays_folder_backed_after_cutover",
            "runtime_08_ecs_root_leaf_owners_stay_explicit_after_data_cutover",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
        ],
    );
    assert_source_anchors(
        "Runtime 08 behavior test",
        &[
            include_str!("../../scene/tests/ecs_identity_storage.rs"),
            include_str!("../../scene/tests/ecs_observers_messages.rs"),
            include_str!("../../scene/tests/ecs_commands.rs"),
            include_str!("../../scene/tests/ecs_events_messages.rs"),
            include_str!("../../scene/tests/ecs_change_detection.rs"),
        ],
        EXPECTED_RUNTIME_08_BEHAVIOR_TEST_ANCHORS,
    );

    let mirror_docs = [
        (
            "Runtime 08 ECS module doc",
            include_str!("../../../../docs/zircon_runtime/scene/ecs.md"),
        ),
        (
            "Runtime 08 plan",
            include_str!(
                "../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "ecs_kernel_data_boundary",
            "expected_source_file_count = 69",
            "expected_test_file_count = 8",
            "archetype_anchors = 15/15",
            "storage_anchors = 9/9",
            "component_identity_anchors = 18/18",
            "entity_lifecycle_anchors = 10/10",
            "observer_anchors = 8/8",
            "deferred_command_anchors = 11/11",
            "event_message_anchors = 12/12",
            "resource_identity_anchors = 12/12",
            "change_tick_anchors = 6/6",
            "runtime_08_guard_anchors = 21/21",
            "behavior_test_anchor_count = 16",
            "missing_behavior_test_anchors = []",
            "component_storage_private_reexport_anchors = 9/9",
            "doc_anchors = 13/13",
            "pending_cargo_gate_anchors = 6/6",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 08 ECS data-kernel audit anchor `{required_anchor}`"
            );
        }
    }
}

fn assert_component_storage_private_reexport_cleanup() {
    let mod_source = include_str!("../../scene/ecs/storage/component_storage/mod.rs");
    assert!(
        mod_source.contains("pub use location::ComponentStorageLocation;"),
        "component_storage/mod.rs should keep ComponentStorageLocation as a public re-export"
    );
    assert!(
        mod_source.contains("pub use store::ComponentStorage;"),
        "component_storage/mod.rs should keep ComponentStorage as a public re-export"
    );

    for forbidden_reexport in [
        "pub(super) use entry::",
        "pub(super) use sparse::",
        "pub(super) use table::",
        "pub(super) use utils::",
        "pub(in crate::scene::ecs::storage) use entry::",
        "pub(in crate::scene::ecs::storage) use sparse::",
        "pub(in crate::scene::ecs::storage) use table::",
        "pub(in crate::scene::ecs::storage) use utils::",
    ] {
        assert!(
            !mod_source.contains(forbidden_reexport),
            "component_storage/mod.rs should not return to parent private re-export hub form: `{forbidden_reexport}`"
        );
    }

    let sparse_source = include_str!("../../scene/ecs/storage/component_storage/sparse.rs");
    let table_source = include_str!("../../scene/ecs/storage/component_storage/table.rs");
    let utils_source = include_str!("../../scene/ecs/storage/component_storage/utils.rs");
    let store_source = include_str!("../../scene/ecs/storage/component_storage/store.rs");

    assert!(
        sparse_source.contains("use super::entry::{RawRemoveResult, StoredComponent};"),
        "sparse.rs should import erased entry/remove-result owners directly"
    );
    assert!(
        table_source.contains("use super::entry::{RawRemoveResult, StoredComponent};"),
        "table.rs should import erased entry/remove-result owners directly"
    );
    assert!(
        utils_source.contains("use super::entry::StoredComponent;"),
        "utils.rs should import StoredComponent from the entry owner directly"
    );

    for required_import in [
        "use super::location::ComponentStorageLocation;",
        "use super::sparse::SparseComponentStorage;",
        "use super::table::TableComponentStorage;",
        "use super::utils::{downcast_component, sort_component_ids_if_needed};",
    ] {
        assert!(
            store_source.contains(required_import),
            "store.rs should retain sibling owner import `{required_import}`"
        );
    }

    for (file_name, source, forbidden_import) in [
        (
            "sparse.rs",
            sparse_source,
            "use super::{RawRemoveResult, StoredComponent};",
        ),
        (
            "table.rs",
            table_source,
            "use super::{RawRemoveResult, StoredComponent};",
        ),
        ("utils.rs", utils_source, "use super::StoredComponent;"),
        (
            "store.rs",
            store_source,
            "use super::{downcast_component, sort_component_ids_if_needed};",
        ),
    ] {
        assert!(
            !source.contains(forbidden_import),
            "{file_name} should not depend on parent private re-export `{forbidden_import}`"
        );
    }
}

fn assert_files_exist(runtime_root: &Path, files: &[&str], label: &str) {
    for file in files {
        assert!(
            runtime_root.join(file).exists(),
            "{label} file `{file}` is missing; update ecs_kernel_data_boundary before changing the Runtime 08 owner set"
        );
    }
}

fn assert_source_anchors(label: &str, sources: &[&str], anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            sources.iter().any(|source| source.contains(anchor)),
            "{label} should retain `{anchor}`"
        );
    }
}
