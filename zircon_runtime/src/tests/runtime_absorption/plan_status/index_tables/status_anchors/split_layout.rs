use super::super::super::support::{assert_contains_all, runtime_numbered_archive_sources};

const STATUS: &str =
    "runtime_15_plan_status_index_status_anchors_folder_backed_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 plan-status index status-anchors folder-backed split";
const GUARD: &str = "runtime_15_plan_status_index_status_anchors_are_folder_backed";

const PARENT_PATH: &str = "plan_status/index_tables/status_anchors.rs";
const CHILD_PATHS: &[&str] = &[
    "plan_status/index_tables/status_anchors/runtime03_module_doc.rs",
    "plan_status/index_tables/status_anchors/runtime07_scene_asset.rs",
    "plan_status/index_tables/status_anchors/runtime07_owner_budget.rs",
    "plan_status/index_tables/status_anchors/generated_status.rs",
    "plan_status/index_tables/status_anchors/runtime10_behavior.rs",
    "plan_status/index_tables/status_anchors/cargo_attempt.rs",
    "plan_status/index_tables/status_anchors/split_layout.rs",
];

#[test]
fn runtime_15_plan_status_index_status_anchors_are_folder_backed() {
    let parent = include_str!("../status_anchors.rs");
    assert_contains_all(
        "plan-status index status-anchor parent mounts child owners",
        parent,
        &[
            "mod cargo_attempt;",
            "mod generated_status;",
            "mod runtime03_module_doc;",
            "mod runtime07_owner_budget;",
            "mod runtime07_scene_asset;",
            "mod runtime10_behavior;",
            "mod split_layout;",
        ],
    );

    let moved_guards = [
        "runtime_15_runtime_03_module_doc_status_index_anchors_are_locked",
        "runtime_15_runtime_07_scene_asset_status_index_anchors_are_locked",
        "runtime_15_runtime_07_owner_budget_status_index_anchors_are_locked",
        "runtime_15_runtime_02_generated_status_index_anchors_are_locked",
        "runtime_15_runtime_10_behavior_status_index_anchors_are_locked",
        "runtime_15_runtime_cargo_attempt_status_index_anchors_are_locked",
    ];
    for moved_guard in moved_guards {
        assert!(
            !parent.contains(&format!("fn {moved_guard}")),
            "plan-status index status-anchor parent should not retain moved guard `{moved_guard}`"
        );
    }

    let child_sources = [
        include_str!("runtime03_module_doc.rs"),
        include_str!("runtime07_scene_asset.rs"),
        include_str!("runtime07_owner_budget.rs"),
        include_str!("generated_status.rs"),
        include_str!("runtime10_behavior.rs"),
        include_str!("cargo_attempt.rs"),
        include_str!("split_layout.rs"),
    ];
    for moved_guard in moved_guards {
        assert!(
            child_sources
                .iter()
                .any(|source| source.contains(&format!("fn {moved_guard}"))),
            "plan-status index status-anchor child owners should retain moved guard `{moved_guard}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 20usize),
        (CHILD_PATHS[0], child_sources[0], 90),
        (CHILD_PATHS[1], child_sources[1], 120),
        (CHILD_PATHS[2], child_sources[2], 120),
        (CHILD_PATHS[3], child_sources[3], 110),
        (CHILD_PATHS[4], child_sources[4], 130),
        (CHILD_PATHS[5], child_sources[5], 150),
        (CHILD_PATHS[6], child_sources[6], 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let index_tables = include_str!("../../index_tables.rs");
    assert_contains_all(
        "plan-status index tables parent mounts status-anchor parent",
        index_tables,
        &["mod status_anchors;", "index_tables/status_anchors.rs"],
    );

    let row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/plan_status_children.rs"
    );
    assert_contains_all(
        "runtime index anchor row data records status-anchor folder split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[6],
            GUARD,
        ],
    );

    let status_map = include_str!(
        "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
    );
    assert_contains_all(
        "runtime index anchor status map",
        status_map,
        &[SLICE, STATUS],
    );

    let date_map = include_str!(
        "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
    );
    assert_contains_all(
        "runtime index anchor date map",
        date_map,
        &[SLICE, "2026-07-05"],
    );

    let archive_source = runtime_numbered_archive_sources();
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &[SLICE, STATUS, GUARD, CHILD_PATHS[6]],
    );
}
