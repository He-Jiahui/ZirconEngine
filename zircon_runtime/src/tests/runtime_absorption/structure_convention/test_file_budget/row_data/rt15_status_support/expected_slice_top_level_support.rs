use super::*;

#[test]
fn runtime_15_status_support_top_level_support_row_data_is_folder_backed() {
    let parent = read_runtime_src(EXPECTED_SLICE_TOP_LEVEL_SUPPORT_PATH);
    let row_children = EXPECTED_SLICE_TOP_LEVEL_SUPPORT_CHILDREN
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/status_support_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/status_support_guard_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "top-level support row-data parent mounts focused children",
        &parent,
        &[
            "#[path = \"top_level_support/support_layout_rows.rs\"]",
            "#[path = \"top_level_support/child_owner_rows.rs\"]",
            "#[path = \"top_level_support/maps_guard_rows.rs\"]",
            "#[path = \"top_level_support/naming_boundary_rows.rs\"]",
            "#[path = \"top_level_support/row_data_owner_rows.rs\"]",
            "row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0]",
        ],
    );
    assert!(
        !parent.contains(
            "Runtime 15 M3 top-level expected-slice support-layout guard folder-backed split"
        ),
        "top_level_support.rs should route row groups instead of retaining concrete row tuples"
    );
    assert_contains_all(
        "top-level support row-data children retain historical and current rows",
        &row_children,
        &[
            "Runtime 15 M3 top-level expected-slice support-layout guard folder-backed split",
            "Runtime 15 M3 status output Runtime 15 expected-slice maps guard body budgets folder-backed split",
            "Runtime 15 M3 top-level expected-slice assertion helper child split",
            TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_NAME,
            TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_ID,
            TOP_LEVEL_SUPPORT_ROW_DATA_GUARD_NAME,
        ],
    );

    let status_anchors = [
        TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_NAME,
        TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/support_layout_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/child_owner_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/maps_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/naming_boundary_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support/row_data_owner_rows.rs",
        TOP_LEVEL_SUPPORT_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("status rows", row_children.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_02.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "status map records top-level support row-data split",
        &status_map,
        &[
            TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_NAME,
            TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records top-level support row-data split",
        &date_map,
        &[TOP_LEVEL_SUPPORT_ROW_DATA_STATUS_NAME, "2026-07-07"],
    );
}
