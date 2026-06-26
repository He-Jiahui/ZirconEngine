use super::*;

#[test]
fn runtime_15_scene_ecs_query_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/ecs_query.rs");
    let cached_queries = read_runtime_src("scene/tests/ecs_query/cached_queries.rs");
    let fixed_ticks = read_runtime_src("scene/tests/ecs_query/fixed_ticks.rs");
    let iter_many = read_runtime_src("scene/tests/ecs_query/iter_many.rs");
    let mutation_access = read_runtime_src("scene/tests/ecs_query/mutation_access.rs");
    let read_items = read_runtime_src("scene/tests/ecs_query/read_items.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "scene ECS query parent test module mounts",
        &parent,
        &[
            "mod cached_queries;",
            "mod fixed_ticks;",
            "mod iter_many;",
            "mod mutation_access;",
            "mod read_items;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/ecs_query.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn query_state_reads_required_optional_and_entity_items_with_filters",
        "fn query_state_mutates_matching_components_without_touching_filtered_entities",
        "fn fixed_scene_components_are_queryable_through_m3_api",
        "fn system_query_iter_many_mut_preserves_order_duplicates_and_run_window_filters",
        "fn query_state_cached_iteration_rebuilds_only_for_structural_changes",
    ] {
        assert!(
            !parent.contains(moved_test),
            "scene/tests/ecs_query.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "read-items child owns query data read contracts",
        &read_items,
        &[
            "use super::*;",
            "fn query_state_reads_required_optional_and_entity_items_with_filters",
            "fn query_state_supports_five_item_data_and_filter_tuples",
            "fn query_state_reads_stable_entity_location_as_query_data",
            "fn query_state_single_reports_zero_one_many_matches",
        ],
    );
    assert_contains_all(
        "mutation/access child owns mutable query and access contracts",
        &mutation_access,
        &[
            "use super::*;",
            "fn query_state_mutates_matching_components_without_touching_filtered_entities",
            "fn query_state_get_mut_helpers_mutate_targets_and_reject_aliases",
            "fn query_access_detects_conflicts_and_filter_disjointness",
            "fn query_access_rejects_duplicate_mutable_component_in_one_query",
        ],
    );
    assert_contains_all(
        "fixed/ticks child owns fixed component and change tick contracts",
        &fixed_ticks,
        &[
            "use super::*;",
            "fn fixed_scene_components_are_queryable_through_m3_api",
            "fn ref_and_mut_query_items_report_change_ticks",
        ],
    );
    assert_contains_all(
        "iter-many child owns targeted iteration contracts",
        &iter_many,
        &[
            "use super::*;",
            "fn system_query_iter_many_mut_preserves_order_duplicates_and_run_window_filters",
            "fn system_query_iter_many_cached_direct_preserves_order_duplicates_and_run_window_filters",
        ],
    );
    assert_contains_all(
        "cached query child owns cached query contracts",
        &cached_queries,
        &[
            "use super::*;",
            "fn query_state_cached_iteration_rebuilds_only_for_structural_changes",
            "fn query_state_count_and_empty_helpers_can_use_cached_candidates",
            "fn query_state_cached_direct_iteration_reads_storage_locations",
            "fn query_state_cached_direct_iteration_reads_sparse_locations",
            "fn query_state_cached_archetypes_do_not_require_optional_reads",
        ],
    );

    let migrated_test_count = [
        read_items.as_str(),
        mutation_access.as_str(),
        fixed_ticks.as_str(),
        iter_many.as_str(),
        cached_queries.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 19,
        "scene ECS query child owners should preserve all 19 tests moved out of the parent"
    );

    for (path, source) in [
        ("scene/tests/ecs_query.rs", parent.as_str()),
        (
            "scene/tests/ecs_query/cached_queries.rs",
            cached_queries.as_str(),
        ),
        ("scene/tests/ecs_query/fixed_ticks.rs", fixed_ticks.as_str()),
        ("scene/tests/ecs_query/iter_many.rs", iter_many.as_str()),
        (
            "scene/tests/ecs_query/mutation_access.rs",
            mutation_access.as_str(),
        ),
        ("scene/tests/ecs_query/read_items.rs", read_items.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("ECS doc", ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 scene ECS query test folder split",
                "runtime_15_scene_ecs_query_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/ecs_query.rs",
                "scene/tests/ecs_query/cached_queries.rs",
                "scene/tests/ecs_query/mutation_access.rs",
                "runtime_15_scene_ecs_query_tests_are_folder_backed",
            ],
        );
    }
}
