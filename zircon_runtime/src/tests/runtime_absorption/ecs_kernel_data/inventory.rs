use std::path::Path;

use super::{docs, guard_coverage, identity_storage, runtime_flow, support::assert_files_exist};

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
    "src/scene/ecs/storage/component_storage/component_results.rs",
    "src/scene/ecs/storage/component_storage/entry.rs",
    "src/scene/ecs/storage/component_storage/location.rs",
    "src/scene/ecs/storage/component_storage/sparse.rs",
    "src/scene/ecs/storage/component_storage/store.rs",
    "src/scene/ecs/storage/component_storage/table.rs",
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
    "src/scene/ecs/observer/callback_registry.rs",
    "src/scene/ecs/observer/callbacks.rs",
    "src/scene/ecs/observer/entry.rs",
    "src/scene/ecs/observer/id.rs",
    "src/scene/ecs/observer/store.rs",
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
    "src/core/framework/scene/resource.rs",
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
    "src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_08.rs",
    "src/tests/runtime_absorption/ecs_kernel_data.rs",
    "src/tests/runtime_absorption/ecs_kernel_data/inventory.rs",
];

#[test]
fn runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts() {
    assert_eq!(EXPECTED_RUNTIME_08_SOURCE_FILES.len(), 69);
    assert_eq!(EXPECTED_RUNTIME_08_TEST_FILES.len(), 10);
    assert_eq!(
        guard_coverage::EXPECTED_RUNTIME_08_BEHAVIOR_TEST_ANCHORS.len(),
        16
    );

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

    identity_storage::assert_runtime_08_identity_and_storage_anchors();
    runtime_flow::assert_runtime_08_flow_anchors();
    guard_coverage::assert_runtime_08_guard_and_behavior_anchors();
    docs::assert_runtime_08_mirror_docs();
}
