pub(super) fn assert_runtime_08_mirror_docs() {
    let mirror_docs = [
        (
            "Runtime 08 ECS module doc",
            include_str!("../../../../../docs/zircon_runtime/scene/ecs.md"),
        ),
        (
            "Runtime 08 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "ecs_kernel_data_boundary",
            "expected_source_file_count = 75",
            "expected_test_file_count = 10",
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
