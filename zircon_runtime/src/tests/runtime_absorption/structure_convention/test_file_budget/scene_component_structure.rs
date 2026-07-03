use super::*;

#[test]
fn runtime_15_scene_component_structure_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/component_structure.rs");
    let component_registry =
        read_runtime_src("scene/tests/component_structure/component_registry.rs");
    let component_storage_dispatch =
        read_runtime_src("scene/tests/component_structure/component_storage_dispatch.rs");
    let component_storage_indexing =
        read_runtime_src("scene/tests/component_structure/component_storage_indexing.rs");
    let dynamic_scene_owner_tree =
        read_runtime_src("scene/tests/component_structure/dynamic_scene_owner_tree.rs");
    let project_serialization =
        read_runtime_src("scene/tests/component_structure/project_serialization.rs");
    let runtime_08_owner_tree =
        read_runtime_src("scene/tests/component_structure/runtime_08_owner_tree.rs");
    let runtime_world_domains =
        read_runtime_src("scene/tests/component_structure/runtime_world_domains.rs");

    assert_contains_all(
        "scene component-structure parent mounts folder-backed children",
        &parent,
        &[
            "mod component_registry;",
            "mod component_storage_dispatch;",
            "mod component_storage_indexing;",
            "mod dynamic_scene_owner_tree;",
            "mod project_serialization;",
            "mod runtime_08_owner_tree;",
            "mod runtime_world_domains;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/component_structure.rs should only mount child test owners"
    );
    for moved_test in [
        "scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover",
        "component_registry_rust_type_reverse_lookup_uses_descriptor_source",
        "scene_project_serialization_sources_do_not_store_editor_authoring_state",
        "dynamic_scene_root_owner_tree_stays_folder_backed_after_runtime_05_cutover",
        "component_storage_get_mut_at_tick_uses_single_storage_dispatch",
        "table_component_get_mut_uses_row_index_directly",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved component-structure test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "component-registry child owns registry lookup tests",
        &component_registry,
        &[
            "fn component_registry_rust_type_reverse_lookup_uses_descriptor_source",
            "fn component_registry_dynamic_lookup_uses_borrowed_type_id_map",
        ],
    );
    assert_contains_all(
        "component-storage dispatch child owns storage dispatch tests",
        &component_storage_dispatch,
        &[
            "fn component_storage_sparse_location_reads_value_and_ticks_from_single_entry",
            "fn component_storage_get_mut_at_tick_uses_single_storage_dispatch",
            "fn component_storage_result_vectors_are_pre_sized_to_storage_count",
        ],
    );
    assert_contains_all(
        "component-storage indexing child owns table/sparse indexing tests",
        &component_storage_indexing,
        &[
            "fn component_storage_type_guards_use_entry_lookup",
            "fn table_component_insert_uses_entry_lookup_for_row_index",
            "fn table_component_get_uses_row_index_directly",
            "fn sparse_component_insert_uses_entry_lookup_for_replacement",
            "fn table_component_ticks_uses_row_index_directly",
            "fn table_component_mark_changed_uses_row_index_directly",
            "fn table_component_get_mut_uses_row_index_directly",
        ],
    );
    assert_contains_all(
        "dynamic-scene owner-tree child owns Runtime 05 owner-tree guards",
        &dynamic_scene_owner_tree,
        &[
            "fn dynamic_scene_root_owner_tree_stays_folder_backed_after_runtime_05_cutover",
            "fn dynamic_scene_session_owner_tree_stays_folder_backed_after_runtime_05_cutover",
        ],
    );
    assert_contains_all(
        "project-serialization child owns authoring boundary guard",
        &project_serialization,
        &["fn scene_project_serialization_sources_do_not_store_editor_authoring_state"],
    );
    assert_contains_all(
        "runtime-world-domains child owns scene structure guards",
        &runtime_world_domains,
        &[
            "fn scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover",
            "fn world_property_access_moves_into_folder_backed_subtree",
            "fn scene_render_extract_does_not_use_snapshot_adapter_for_frame_extract",
            "fn runtime_scene_exposes_neutral_world_inspection_surface",
            "fn scene_ecs_does_not_reintroduce_late_update_stage_or_compatibility_path",
        ],
    );
    assert_contains_all(
        "Runtime 08 owner-tree child remains mounted",
        &runtime_08_owner_tree,
        &[
            "fn runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover",
            "fn runtime_08_ecs_change_detection_owner_tree_stays_folder_backed_after_cutover",
            "fn runtime_08_ecs_root_leaf_owners_stay_explicit_after_data_cutover",
        ],
    );

    let child_test_total = [
        component_registry.as_str(),
        component_storage_dispatch.as_str(),
        component_storage_indexing.as_str(),
        dynamic_scene_owner_tree.as_str(),
        project_serialization.as_str(),
        runtime_08_owner_tree.as_str(),
        runtime_world_domains.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 23,
        "component_structure children should preserve all 23 tests, including existing runtime_08_owner_tree coverage"
    );

    for (path, source) in [
        ("scene/tests/component_structure.rs", parent.as_str()),
        (
            "scene/tests/component_structure/component_registry.rs",
            component_registry.as_str(),
        ),
        (
            "scene/tests/component_structure/component_storage_dispatch.rs",
            component_storage_dispatch.as_str(),
        ),
        (
            "scene/tests/component_structure/component_storage_indexing.rs",
            component_storage_indexing.as_str(),
        ),
        (
            "scene/tests/component_structure/dynamic_scene_owner_tree.rs",
            dynamic_scene_owner_tree.as_str(),
        ),
        (
            "scene/tests/component_structure/project_serialization.rs",
            project_serialization.as_str(),
        ),
        (
            "scene/tests/component_structure/runtime_08_owner_tree.rs",
            runtime_08_owner_tree.as_str(),
        ),
        (
            "scene/tests/component_structure/runtime_world_domains.rs",
            runtime_world_domains.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let dynamic_scene_doc = read_repo("docs/zircon_runtime/scene/dynamic_scene.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", ecs_doc.as_str()),
        ("dynamic scene doc", dynamic_scene_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 scene component-structure test folder split",
                "runtime_15_scene_component_structure_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/component_structure.rs",
                "scene/tests/component_structure/component_storage_indexing.rs",
                "scene/tests/component_structure/dynamic_scene_owner_tree.rs",
                "runtime_15_scene_component_structure_tests_are_folder_backed",
            ],
        );
    }
}
