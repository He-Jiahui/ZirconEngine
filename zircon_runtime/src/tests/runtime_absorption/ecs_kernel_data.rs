use std::path::Path;

const EXPECTED_RUNTIME_08_SOURCE_FILES: &[&str] = &[
    "src/scene/ecs/storage_type.rs",
    "src/scene/ecs/storage/component_storage.rs",
    "src/scene/ecs/entity_registry.rs",
    "src/scene/ecs/despawned_entity.rs",
    "src/scene/ecs/stable_entity_location.rs",
    "src/scene/ecs/internal_entity.rs",
    "src/scene/ecs/observer.rs",
    "src/scene/ecs/commands/command.rs",
    "src/scene/ecs/commands/command_queue.rs",
    "src/scene/ecs/commands/commands.rs",
    "src/scene/ecs/events.rs",
    "src/scene/ecs/messages.rs",
    "src/scene/ecs/change_detection/change_tick.rs",
    "src/scene/ecs/change_detection/change_tick_window.rs",
    "src/scene/ecs/change_detection/component_ticks.rs",
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
    "src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
    "src/tests/runtime_absorption/ecs_kernel_data.rs",
];

#[test]
fn runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_08_SOURCE_FILES.len(), 20);
    assert_eq!(EXPECTED_RUNTIME_08_TEST_FILES.len(), 7);

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
        "Runtime 08 storage",
        &[
            include_str!("../../scene/ecs/storage_type.rs"),
            include_str!("../../scene/ecs/storage/component_storage.rs"),
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
    assert_source_anchors(
        "Runtime 08 entity lifecycle",
        &[
            include_str!("../../scene/ecs/entity_registry.rs"),
            include_str!("../../scene/ecs/internal_entity.rs"),
            include_str!("../../scene/ecs/stable_entity_location.rs"),
            include_str!("../../scene/ecs/despawned_entity.rs"),
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
            include_str!("../../scene/ecs/observer.rs"),
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
            include_str!("../../scene/ecs/commands/commands.rs"),
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
            include_str!("../../scene/ecs/events.rs"),
            include_str!("../../scene/ecs/messages.rs"),
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
            "pub struct MessageId<T>",
            "pub struct Messages<T>",
            "next_id: usize",
            "pub fn clear(&mut self)",
        ],
    );
    assert_source_anchors(
        "Runtime 08 change tick",
        &[
            include_str!("../../scene/ecs/change_detection/change_tick.rs"),
            include_str!("../../scene/ecs/change_detection/change_tick_window.rs"),
            include_str!("../../scene/ecs/change_detection/component_ticks.rs"),
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
            "clear_events_prunes_current_and_next_event_queues",
            "messages_are_retained_until_explicit_clear_independent_of_event_updates",
            "event_and_message_clear_boundaries_do_not_cross_channels",
            "change_tick_comparison_survives_wraparound",
            "tick_window_clamps_stale_ticks",
            "runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation",
            "runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts",
        ],
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
            "expected_source_file_count = 20",
            "expected_test_file_count = 7",
            "storage_anchors = 9/9",
            "entity_lifecycle_anchors = 10/10",
            "observer_anchors = 8/8",
            "deferred_command_anchors = 11/11",
            "event_message_anchors = 11/11",
            "change_tick_anchors = 6/6",
            "runtime_08_guard_anchors = 17/17",
            "doc_anchors = 7/7",
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
