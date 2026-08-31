use super::*;

#[test]
fn runtime_15_scene_ecs_schedule_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/ecs_schedule.rs");
    let conflict_graph = read_runtime_src("scene/tests/ecs_schedule/conflict_graph.rs");
    let conflict_access =
        read_runtime_src("scene/tests/ecs_schedule/conflict_graph/access_conflicts.rs");
    let conflict_parallel =
        read_runtime_src("scene/tests/ecs_schedule/conflict_graph/parallel_batches.rs");
    let fixed_update = read_runtime_src("scene/tests/ecs_schedule/fixed_update.rs");
    let parallel_executor = read_runtime_src("scene/tests/ecs_schedule/parallel_executor.rs");
    let render_extract = read_runtime_src("scene/tests/ecs_schedule/render_extract.rs");
    let resources_events = read_runtime_src("scene/tests/ecs_schedule/resources_events.rs");
    let schedule_plan = read_runtime_src("scene/tests/ecs_schedule/schedule_plan.rs");
    let world_driver = read_runtime_src("scene/tests/ecs_schedule/world_driver.rs");
    let world_time_controller =
        read_runtime_src("scene/tests/ecs_schedule/world_time_controller.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");

    assert_contains_all(
        "scene ECS schedule parent test module mounts",
        &parent,
        &[
            "mod conflict_graph;",
            "mod fixed_update;",
            "mod parallel_executor;",
            "mod render_extract;",
            "mod resources_events;",
            "mod schedule_plan;",
            "mod world_driver;",
            "mod world_time_controller;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/ecs_schedule.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn resource_store_keeps_resources_by_concrete_type",
        "fn event_payload_profile_marks_large_payloads_for_arc_indirection",
        "fn schedule_uses_bevy_style_stage_order_and_builtin_post_update_systems",
        "fn schedule_maintains_executor_stage_plan_after_registration_and_load",
        "fn world_mutations_mark_derived_state_dirty_until_post_update_systems_flush",
        "fn canonical_render_frame_extract_populates_scene_sections_directly",
        "fn inactive_render_camera_extracts_no_scene_renderables",
        "fn world_driver_defers_hook_mutations_until_builtin_post_update_systems_run",
        "fn world_driver_runs_runtime_scene_systems_in_schedule_order",
    ] {
        assert!(
            !parent.contains(moved_test),
            "scene/tests/ecs_schedule.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "resources/events child owns ECS resource and event contracts",
        &resources_events,
        &[
            "use super::*;",
            "fn resource_store_keeps_resources_by_concrete_type",
            "fn event_store_tracks_each_event_type_independently",
            "fn dormant_subscription_connects_on_plugin_activate",
            "fn event_payload_profile_marks_large_payloads_for_arc_indirection",
        ],
    );
    assert_contains_all(
        "schedule plan child owns stage and registration contracts",
        &schedule_plan,
        &[
            "use super::*;",
            "fn schedule_uses_bevy_style_stage_order_and_builtin_post_update_systems",
            "fn stage_plan_orders_by_constraints_then_order",
            "fn schedule_maintains_executor_stage_plan_after_registration_and_load",
            "fn native_system_registration_reports_missing_required_resources",
        ],
    );
    assert_contains_all(
        "render extract child owns derived-state and camera extraction contracts",
        &render_extract,
        &[
            "use super::*;",
            "fn world_mutations_mark_derived_state_dirty_until_post_update_systems_flush",
            "fn canonical_render_frame_extract_populates_scene_sections_directly",
            "fn render_extract_projects_scene_camera_component_product_fields",
            "fn inactive_render_camera_extracts_no_scene_renderables",
        ],
    );
    assert_contains_all(
        "world driver child owns hook and runtime scene system ordering contracts",
        &world_driver,
        &[
            "use super::*;",
            "fn world_driver_defers_hook_mutations_until_builtin_post_update_systems_run",
            "fn world_driver_runs_native_render_extract_system_before_render_extract_hooks",
            "fn world_driver_orders_native_systems_with_plugin_hooks",
            "fn world_driver_runs_runtime_scene_systems_in_schedule_order",
        ],
    );
    assert_contains_all(
        "world time controller child owns multi-World timing contracts",
        &world_time_controller,
        &[
            "fn worlds_derive_virtual_and_fixed_time_independently_from_one_outer_frame",
            "fn world_fixed_debt_uses_the_outer_budget_not_another_clock_step_count",
        ],
    );

    let migrated_test_count = [
        resources_events.as_str(),
        schedule_plan.as_str(),
        render_extract.as_str(),
        world_driver.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 42,
        "scene ECS schedule child owners should preserve the current 42 moved behavior tests"
    );
    let schedule_family_test_count = [
        conflict_graph.as_str(),
        conflict_access.as_str(),
        conflict_parallel.as_str(),
        fixed_update.as_str(),
        parallel_executor.as_str(),
        render_extract.as_str(),
        resources_events.as_str(),
        schedule_plan.as_str(),
        world_driver.as_str(),
        world_time_controller.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        schedule_family_test_count, 66,
        "scene ECS schedule folder should retain the current 66-test family"
    );

    for (path, source) in [
        ("scene/tests/ecs_schedule.rs", parent.as_str()),
        (
            "scene/tests/ecs_schedule/conflict_graph.rs",
            conflict_graph.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/conflict_graph/access_conflicts.rs",
            conflict_access.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/conflict_graph/parallel_batches.rs",
            conflict_parallel.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/fixed_update.rs",
            fixed_update.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/parallel_executor.rs",
            parallel_executor.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/render_extract.rs",
            render_extract.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/resources_events.rs",
            resources_events.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/schedule_plan.rs",
            schedule_plan.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/world_driver.rs",
            world_driver.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/world_time_controller.rs",
            world_time_controller.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}

#[test]
fn runtime_15_scene_ecs_schedule_conflict_graph_children_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/ecs_schedule/conflict_graph.rs");
    let access_conflicts =
        read_runtime_src("scene/tests/ecs_schedule/conflict_graph/access_conflicts.rs");
    let parallel_batches =
        read_runtime_src("scene/tests/ecs_schedule/conflict_graph/parallel_batches.rs");

    assert_contains_all(
        "scene ECS schedule conflict graph parent mounts child owners",
        &parent,
        &["mod access_conflicts;", "mod parallel_batches;"],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/ecs_schedule/conflict_graph.rs should only mount child test owners and shared fixtures"
    );
    for moved_test in [
        "schedule_conflict_graph_reports_component_write_conflicts_in_same_stage",
        "schedule_conflict_graph_reports_event_and_message_writer_conflicts",
        "schedule_conflict_graph_builds_conservative_parallel_batches",
        "schedule_conflict_graph_keeps_parallel_batches_inside_stage_boundaries",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved conflict graph test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "conflict graph access child owns component/resource/event/world access conflicts",
        &access_conflicts,
        &[
            "fn schedule_conflict_graph_reports_component_write_conflicts_in_same_stage",
            "fn schedule_conflict_graph_reports_resource_write_conflicts",
            "fn schedule_conflict_graph_reports_event_and_message_writer_conflicts",
            "fn schedule_conflict_graph_reports_conservative_world_access_conflicts",
        ],
    );
    assert_contains_all(
        "conflict graph parallel child owns batching and source guards",
        &parallel_batches,
        &[
            "fn schedule_conflict_graph_builds_conservative_parallel_batches",
            "fn schedule_conflict_graph_keeps_parallel_batches_inside_stage_boundaries",
            "include_str!(\"../../../ecs/schedule_conflict_graph.rs\")",
            "include_str!(\"../../../ecs/system/system_param_access.rs\")",
            "include_str!(\"../../../ecs/query/query_access.rs\")",
        ],
    );

    let child_test_total = [access_conflicts.as_str(), parallel_batches.as_str()]
        .into_iter()
        .map(|source| source.matches("#[test]").count())
        .sum::<usize>();
    assert_eq!(
        child_test_total, 9,
        "conflict graph children should preserve all 9 parent tests"
    );

    for (path, source) in [
        (
            "scene/tests/ecs_schedule/conflict_graph.rs",
            parent.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/conflict_graph/access_conflicts.rs",
            access_conflicts.as_str(),
        ),
        (
            "scene/tests/ecs_schedule/conflict_graph/parallel_batches.rs",
            parallel_batches.as_str(),
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
}
