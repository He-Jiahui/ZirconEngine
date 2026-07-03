use super::*;

#[test]
fn runtime_15_scene_ecs_systems_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/ecs_systems.rs");
    let commands = read_runtime_src("scene/tests/ecs_systems/commands.rs");
    let events = read_runtime_src("scene/tests/ecs_systems/events.rs");
    let many_single_queries = read_runtime_src("scene/tests/ecs_systems/many_single_queries.rs");
    let removal_local = read_runtime_src("scene/tests/ecs_systems/removal_local.rs");
    let run_window_filters = read_runtime_src("scene/tests/ecs_systems/run_window_filters.rs");
    let state_params = read_runtime_src("scene/tests/ecs_systems/state_params.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "scene ECS systems parent test module mounts",
        &parent,
        &[
            "mod commands;",
            "mod events;",
            "mod many_single_queries;",
            "mod removal_local;",
            "mod run_window_filters;",
            "mod state_params;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/ecs_systems.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn commands_are_deferred_until_apply_deferred",
        "fn system_state_runs_query_resource_and_commands_params",
        "fn event_reader_and_writer_use_current_and_next_queues",
        "fn added_and_changed_filters_use_system_run_windows",
        "fn system_query_get_many_helpers_preserve_order_duplicates_and_run_window_filters",
        "fn removed_components_reader_observes_direct_and_deferred_removals",
    ] {
        assert!(
            !parent.contains(moved_test),
            "scene/tests/ecs_systems.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "commands child owns deferred command contracts",
        &commands,
        &[
            "use super::*;",
            "fn commands_are_deferred_until_apply_deferred",
            "fn entity_commands_spawn_empty_and_entity_or_spawn_apply_in_queue_order",
        ],
    );
    assert_contains_all(
        "state params child owns SystemState and ParamSet contracts",
        &state_params,
        &[
            "use super::*;",
            "fn system_state_runs_query_resource_and_commands_params",
            "fn system_query_get_mut_helpers_mutate_targets_and_reject_aliases",
            "fn param_set_supports_segmented_access_up_to_eight_items",
        ],
    );
    assert_contains_all(
        "events child owns reader and writer contracts",
        &events,
        &[
            "use super::*;",
            "fn event_reader_and_writer_use_current_and_next_queues",
            "fn event_reader_param_keeps_cursor_between_system_runs",
            "fn event_reader_param_observes_events_after_global_clear",
        ],
    );
    assert_contains_all(
        "run-window filter child owns change-detection query contracts",
        &run_window_filters,
        &[
            "use super::*;",
            "fn added_and_changed_filters_use_system_run_windows",
            "fn system_query_iter_cached_reuses_state_and_rechecks_run_window_filters",
            "fn system_query_count_and_empty_helpers_reuse_cache_and_run_window_filters",
        ],
    );
    assert_contains_all(
        "many/single query child owns many and single query contracts",
        &many_single_queries,
        &[
            "use super::*;",
            "fn system_query_get_many_helpers_preserve_order_duplicates_and_run_window_filters",
            "fn system_query_iter_many_preserves_order_duplicates_and_run_window_filters",
            "fn system_query_single_helpers_report_zero_one_many_matches",
        ],
    );
    assert_contains_all(
        "removal/local child owns removed-component and local state contracts",
        &removal_local,
        &[
            "use super::*;",
            "fn removed_components_reader_observes_direct_and_deferred_removals",
            "fn local_param_state_persists_between_system_runs",
            "fn scheduled_native_system_keeps_local_state_between_ticks",
        ],
    );

    let migrated_test_count = [
        commands.as_str(),
        state_params.as_str(),
        events.as_str(),
        run_window_filters.as_str(),
        many_single_queries.as_str(),
        removal_local.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 24,
        "scene ECS systems child owners should preserve all 24 tests moved out of the parent"
    );

    for (path, source) in [
        ("scene/tests/ecs_systems.rs", parent.as_str()),
        ("scene/tests/ecs_systems/commands.rs", commands.as_str()),
        ("scene/tests/ecs_systems/events.rs", events.as_str()),
        (
            "scene/tests/ecs_systems/many_single_queries.rs",
            many_single_queries.as_str(),
        ),
        (
            "scene/tests/ecs_systems/removal_local.rs",
            removal_local.as_str(),
        ),
        (
            "scene/tests/ecs_systems/run_window_filters.rs",
            run_window_filters.as_str(),
        ),
        (
            "scene/tests/ecs_systems/state_params.rs",
            state_params.as_str(),
        ),
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
                "Runtime 15 M3 scene ECS systems test folder split",
                "runtime_15_scene_ecs_systems_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/ecs_systems.rs",
                "scene/tests/ecs_systems/run_window_filters.rs",
                "scene/tests/ecs_systems/state_params.rs",
                "runtime_15_scene_ecs_systems_tests_are_folder_backed",
            ],
        );
    }
}
