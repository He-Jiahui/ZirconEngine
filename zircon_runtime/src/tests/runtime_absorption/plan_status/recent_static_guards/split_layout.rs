use super::super::support::{assert_contains_all, runtime_numbered_archive_sources};

const STATUS: &str =
    "runtime_15_plan_status_recent_static_guards_folder_backed_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 plan-status recent static guards folder-backed split";
const GUARD: &str = "runtime_15_plan_status_recent_static_guards_are_folder_backed";

const PARENT_PATH: &str = "plan_status/recent_static_guards.rs";
const CHILD_PATHS: &[&str] = &[
    "plan_status/recent_static_guards/document_sources.rs",
    "plan_status/recent_static_guards/parent_routing.rs",
    "plan_status/recent_static_guards/runtime_01_to_04.rs",
    "plan_status/recent_static_guards/runtime_05_to_08.rs",
    "plan_status/recent_static_guards/runtime_09_to_12.rs",
    "plan_status/recent_static_guards/runtime_13_14_review_index.rs",
    "plan_status/recent_static_guards/split_layout.rs",
];

#[test]
fn runtime_15_plan_status_recent_static_guards_are_folder_backed() {
    let parent = include_str!("../recent_static_guards.rs");
    let child_sources = [
        include_str!("document_sources.rs"),
        include_str!("parent_routing.rs"),
        include_str!("runtime_01_to_04.rs"),
        include_str!("runtime_05_to_08.rs"),
        include_str!("runtime_09_to_12.rs"),
        include_str!("runtime_13_14_review_index.rs"),
        include_str!("split_layout.rs"),
    ];

    assert_contains_all(
        "recent static guards parent mounts folder-backed owners",
        parent,
        &[
            "mod document_sources;",
            "mod parent_routing;",
            "mod runtime_01_to_04;",
            "mod runtime_05_to_08;",
            "mod runtime_09_to_12;",
            "mod runtime_13_14_review_index;",
            "mod split_layout;",
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
        ],
    );

    for moved_anchor in [
        "runtime_01_plan_anchors",
        "runtime_05_anchors",
        "runtime_09_anchors",
        "runtime_13_anchors",
        "Runtime architecture review",
        "../../../../../../docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "recent static guard parent should not retain moved anchor/source `{moved_anchor}`"
        );
        assert!(
            child_sources.iter().any(|source| source.contains(moved_anchor)),
            "recent static guard children should own moved anchor/source `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 40usize),
        (CHILD_PATHS[0], child_sources[0], 190),
        (CHILD_PATHS[1], child_sources[1], 80),
        (CHILD_PATHS[2], child_sources[2], 190),
        (CHILD_PATHS[3], child_sources[3], 120),
        (CHILD_PATHS[4], child_sources[4], 190),
        (CHILD_PATHS[5], child_sources[5], 190),
        (CHILD_PATHS[6], child_sources[6], 190),
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
        "runtime index anchor row data parent exports recent static guard split",
        row_data_parent,
        &["plan_status_children::RECENT_STATIC_GUARDS_FOLDER_BACKED_SPLIT"],
    );

    let row_data = include_str!(
        "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/plan_status_children.rs"
    );
    assert_contains_all(
        "runtime index anchor row data records recent static guard split",
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

    let archive_source = runtime_numbered_archive_sources();
    assert_contains_all(
        "runtime numbered archives",
        &archive_source,
        &[SLICE, STATUS, GUARD, CHILD_PATHS[6]],
    );
}
