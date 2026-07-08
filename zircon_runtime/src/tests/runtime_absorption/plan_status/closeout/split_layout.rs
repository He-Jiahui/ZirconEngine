use super::super::support::assert_contains_all;

const STATUS: &str =
    "runtime_15_plan_status_closeout_guards_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_plan_status_closeout_guards_folder_backed_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 plan-status closeout guards folder-backed split";
const GUARD: &str = "runtime_15_plan_status_closeout_guards_are_folder_backed";

const PARENT_PATH: &str = "plan_status/closeout.rs";
const CHILD_PATHS: &[&str] = &[
    "plan_status/closeout/runtime_05_diagnostics.rs",
    "plan_status/closeout/runtime_05_source_anchors.rs",
    "plan_status/closeout/runtime_05_status.rs",
    "plan_status/closeout/runtime_05_support_first.rs",
    "plan_status/closeout/split_layout.rs",
];

#[test]
fn runtime_15_plan_status_closeout_guards_are_folder_backed() {
    let parent = include_str!("../closeout.rs");
    let child_sources = [
        include_str!("runtime_05_diagnostics.rs"),
        include_str!("runtime_05_source_anchors.rs"),
        include_str!("runtime_05_status.rs"),
        include_str!("runtime_05_support_first.rs"),
        include_str!("split_layout.rs"),
    ];

    assert_contains_all(
        "plan-status closeout parent mounts closeout guard families",
        parent,
        &[
            "mod runtime_05_diagnostics;",
            "mod runtime_05_source_anchors;",
            "mod runtime_05_status;",
            "mod runtime_05_support_first;",
            "mod split_layout;",
        ],
    );

    for moved_anchor in [
        "fn runtime_05_closeout_status_waits_for_full_scene_cargo_gate",
        "fn runtime_05_full_scene_failure_clusters_keep_support_first_triage_visible",
        "fn runtime_05_scene_failure_triage_records_minimum_lower_layer_diagnostics",
        "const GRAPHICS_SCENE_DIAGNOSTIC_SOURCE_ANCHORS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "plan-status closeout parent should not retain moved body `{moved_anchor}`"
        );
        assert!(
            child_sources
                .iter()
                .any(|source| source.contains(moved_anchor)),
            "plan-status closeout children should retain moved body `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 20usize),
        (CHILD_PATHS[0], child_sources[0], 70),
        (CHILD_PATHS[1], child_sources[1], 100),
        (CHILD_PATHS[2], child_sources[2], 35),
        (CHILD_PATHS[3], child_sources[3], 50),
        (CHILD_PATHS[4], child_sources[4], 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data_parent = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs"
    );
    assert_contains_all(
        "runtime index anchor row data parent exports closeout split",
        row_data_parent,
        &["support_inventory::CLOSEOUT_GUARDS_FOLDER_BACKED_SPLIT"],
    );
    let row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/support_inventory.rs"
    );
    assert_contains_all(
        "support inventory row data records closeout split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[4],
            GUARD,
        ],
    );

    let status_map = [
        include_str!(
            "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
        ),
    ]
    .join("\n");
    assert_contains_all(
        "runtime index anchor status map",
        status_map.as_str(),
        &[SLICE, STATUS],
    );

    let date_map = [
        include_str!(
            "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs"
        ),
    ]
    .join("\n");
    assert_contains_all(
        "runtime index anchor date map",
        date_map.as_str(),
        &[SLICE, "2026-07-05"],
    );

    for (label, source) in [
        (
            "Runtime 15 subplan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!(
                "../../../../../../docs/plans/engine-code-review-findings-2026-06.md"
            ),
        ),
        (
            "module convention doc",
            include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!(
                "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[4]]);
    }

    let frameworks = include_str!(
        "../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
    );
    assert_contains_all(
        "frameworks plan records closeout split",
        frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}
