use super::super::super::support::runtime_numbered_archive_sources;
use super::{assert_contains_all, CHILD_PATHS, PARENT_PATH, TEST_ATTRIBUTE};

#[test]
fn runtime_15_plan_status_index_tables_guard_child_owner_split() {
    let parent = include_str!("../../index_tables.rs");
    let subplan_map = include_str!("../subplan_map.rs");
    let status_anchors = include_str!("../status_anchors.rs");
    let status_anchor_runtime03_module_doc =
        include_str!("../status_anchors/runtime03_module_doc.rs");
    let status_anchor_runtime07_scene_asset =
        include_str!("../status_anchors/runtime07_scene_asset.rs");
    let status_anchor_runtime07_owner_budget =
        include_str!("../status_anchors/runtime07_owner_budget.rs");
    let status_anchor_generated_status = include_str!("../status_anchors/generated_status.rs");
    let status_anchor_runtime10_behavior = include_str!("../status_anchors/runtime10_behavior.rs");
    let status_anchor_cargo_attempt = include_str!("../status_anchors/cargo_attempt.rs");
    let status_anchor_split_layout = include_str!("../status_anchors/split_layout.rs");
    let index_consistency = include_str!("../index_consistency.rs");
    let split_layout_route = include_str!("../split_layout.rs");
    let split_layout_child_owner = include_str!("child_owner.rs");
    let split_layout_parent_guard = include_str!("parent_guard.rs");
    let split_layout_split_guard = include_str!("split_guard.rs");
    let status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/plan_status_children.rs",
    );
    let status_map = [
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs",
        ),
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs",
        ),
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs",
        ),
    ]
    .join("\n");

    assert_contains_all(
        "plan-status index table parent mounts child owners",
        parent,
        &[
            "mod index_consistency;",
            "mod split_layout;",
            "mod status_anchors;",
            "mod subplan_map;",
        ],
    );

    let moved_guard_names = [
        "runtime_index_subplan_map_covers_existing_plan_files_without_stale_rows",
        "runtime_15_runtime_index_subplan_map_covers_01_15_status_locked",
        "runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked",
        "runtime_15_runtime_03_module_doc_status_index_anchors_are_locked",
        "runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked",
        "runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked",
        "runtime_15_runtime_02_generated_status_index_anchors_are_locked",
        "runtime_15_runtime_10_behavior_status_index_anchors_are_locked",
        "runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked",
        "runtime_index_problem_rows_reference_existing_subplans",
        "runtime_index_execution_dependencies_reference_existing_subplans",
        "runtime_index_status_map_matches_subplan_frontmatter",
        "runtime_index_in_progress_rows_record_remaining_gate",
        "runtime_known_backlog_gaps_keep_owner_and_trigger_columns",
    ];
    let status_anchor_children = [
        status_anchor_runtime03_module_doc,
        status_anchor_runtime07_scene_asset,
        status_anchor_runtime07_owner_budget,
        status_anchor_generated_status,
        status_anchor_runtime10_behavior,
        status_anchor_cargo_attempt,
        status_anchor_split_layout,
    ]
    .join("\n");
    let children = format!(
        "{subplan_map}\n{status_anchors}\n{status_anchor_children}\n{index_consistency}\n{split_layout_route}\n{split_layout_child_owner}\n{split_layout_parent_guard}\n{split_layout_split_guard}"
    );
    for moved_guard_name in moved_guard_names {
        assert!(
            !parent.contains(&format!("fn {moved_guard_name}")),
            "plan-status index table parent should not retain moved guard `{moved_guard_name}`"
        );
        assert!(
            children.contains(&format!("fn {moved_guard_name}")),
            "plan-status index table children should retain moved guard `{moved_guard_name}`"
        );
    }

    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count()
            + subplan_map.matches(TEST_ATTRIBUTE).count()
            + status_anchors.matches(TEST_ATTRIBUTE).count()
            + status_anchor_children.matches(TEST_ATTRIBUTE).count()
            + index_consistency.matches(TEST_ATTRIBUTE).count()
            + split_layout_route.matches(TEST_ATTRIBUTE).count()
            + split_layout_child_owner.matches(TEST_ATTRIBUTE).count()
            + split_layout_parent_guard.matches(TEST_ATTRIBUTE).count()
            + split_layout_split_guard.matches(TEST_ATTRIBUTE).count(),
        18,
        "plan-status index table split should preserve the 14 moved tests plus four layout guards"
    );
    for (path, source) in [
        (PARENT_PATH, parent),
        ("plan_status/index_tables/subplan_map.rs", subplan_map),
        ("plan_status/index_tables/status_anchors.rs", status_anchors),
        (
            "plan_status/index_tables/status_anchors/runtime03_module_doc.rs",
            status_anchor_runtime03_module_doc,
        ),
        (
            "plan_status/index_tables/status_anchors/runtime07_scene_asset.rs",
            status_anchor_runtime07_scene_asset,
        ),
        (
            "plan_status/index_tables/status_anchors/runtime07_owner_budget.rs",
            status_anchor_runtime07_owner_budget,
        ),
        (
            "plan_status/index_tables/status_anchors/generated_status.rs",
            status_anchor_generated_status,
        ),
        (
            "plan_status/index_tables/status_anchors/runtime10_behavior.rs",
            status_anchor_runtime10_behavior,
        ),
        (
            "plan_status/index_tables/status_anchors/cargo_attempt.rs",
            status_anchor_cargo_attempt,
        ),
        (
            "plan_status/index_tables/status_anchors/split_layout.rs",
            status_anchor_split_layout,
        ),
        (
            "plan_status/index_tables/index_consistency.rs",
            index_consistency,
        ),
        (CHILD_PATHS[1], split_layout_route),
        (
            "plan_status/index_tables/split_layout/child_owner.rs",
            split_layout_child_owner,
        ),
        (
            "plan_status/index_tables/split_layout/parent_guard.rs",
            split_layout_parent_guard,
        ),
        (
            "plan_status/index_tables/split_layout/split_guard.rs",
            split_layout_split_guard,
        ),
    ] {
        assert!(
            source.lines().count() < 800,
            "{path} should remain below the Runtime 15 test-owner budget after child-owner split"
        );
    }

    let status_anchors = [
        "Runtime 15 M3 plan-status index-tables child-owner split",
        "runtime_15_plan_status_index_tables_child_owner_split_static_passed_cargo_deferred",
        "plan_status/index_tables.rs",
        "plan_status/index_tables/subplan_map.rs",
        "plan_status/index_tables/status_anchors.rs",
        "plan_status/index_tables/index_consistency.rs",
        "runtime_15_plan_status_index_tables_guard_child_owner_split",
    ];
    let archive_source = runtime_numbered_archive_sources();
    for (label, source) in [
        ("runtime numbered archives", archive_source.as_str()),
        ("Runtime 15 status row data", status_row_data),
        ("Runtime 15 expected status map", status_map.as_str()),
        ("Runtime 15 expected date map", date_map.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
}
