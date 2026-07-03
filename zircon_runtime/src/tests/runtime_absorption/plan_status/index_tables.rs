use super::support::{
    assert_contains_all, first_backtick_value, frontmatter_status, index_section_between,
    leading_plan_id, markdown_table_cells, referenced_plan_ids, runtime_index_row_for,
    runtime_subplan_sources,
};

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

#[path = "index_tables/index_consistency.rs"]
mod index_consistency;
#[path = "index_tables/status_anchors.rs"]
mod status_anchors;
#[path = "index_tables/subplan_map.rs"]
mod subplan_map;

#[test]
fn runtime_15_plan_status_index_tables_guard_child_owner_split() {
    let parent = include_str!("index_tables.rs");
    let subplan_map = include_str!("index_tables/subplan_map.rs");
    let status_anchors = include_str!("index_tables/status_anchors.rs");
    let index_consistency = include_str!("index_tables/index_consistency.rs");
    let runtime_15_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let structure_convention =
        include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    );
    let status_row_data = include_str!(
        "status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let status_map = include_str!(
        "status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_map = include_str!(
        "status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    assert_contains_all(
        "plan-status index table parent mounts child owners",
        parent,
        &[
            "mod index_consistency;",
            "mod status_anchors;",
            "mod subplan_map;",
            "runtime_15_plan_status_index_tables_guard_child_owner_split",
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
    let children = format!("{subplan_map}\n{status_anchors}\n{index_consistency}");
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
            + index_consistency.matches(TEST_ATTRIBUTE).count(),
        15,
        "plan-status index table split should preserve the 14 moved tests plus the parent layout guard"
    );
    for (path, source) in [
        ("plan_status/index_tables.rs", parent),
        ("plan_status/index_tables/subplan_map.rs", subplan_map),
        ("plan_status/index_tables/status_anchors.rs", status_anchors),
        (
            "plan_status/index_tables/index_consistency.rs",
            index_consistency,
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
    for (label, source) in [
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", status_row_data),
        ("Runtime 15 expected status map", status_map),
        ("Runtime 15 expected date map", date_map),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
}
