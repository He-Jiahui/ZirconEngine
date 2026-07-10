use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_row_data_is_folder_backed() {
    let parent = read_runtime_src(EXPECTED_SLICE_STRUCTURE_SUPPORT_PATH);
    let row_children = EXPECTED_SLICE_STRUCTURE_SUPPORT_CHILDREN
        .iter()
        .chain(EXPECTED_SLICE_STRUCTURE_SUPPORT_NESTED_CHILDREN.iter())
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/expected_slice_support_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/expected_slice_support_maps.rs",
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
        "structure-support row-data parent mounts focused children",
        &parent,
        &[
            "#[path = \"structure_support/parent_route_rows.rs\"]",
            "#[path = \"structure_support/guard_rows.rs\"]",
            "#[path = \"structure_support/map_rows.rs\"]",
            "#[path = \"structure_support/foundation_rows.rs\"]",
            "#[path = \"structure_support/review_route_rows.rs\"]",
            "#[path = \"structure_support/typed_error_rows.rs\"]",
            "#[path = \"structure_support/row_data_owner_rows.rs\"]",
            "row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0]",
        ],
    );
    assert!(
        !parent.contains(
            "Runtime 15 M3 structure-support expected-slice parent-route guard body folder-backed split"
        ),
        "structure_support.rs should route row groups instead of retaining concrete row tuples"
    );
    assert_contains_all(
        "structure-support row-data children retain historical and current rows",
        &row_children,
        &[
            "Runtime 15 M3 structure-support expected-slice parent-route guard body folder-backed split",
            "Runtime 15 M3 review guard typed-error expected-slice map child split",
            STRUCTURE_SUPPORT_ROW_DATA_STATUS_NAME,
            STRUCTURE_SUPPORT_ROW_DATA_STATUS_ID,
            STRUCTURE_SUPPORT_ROW_DATA_GUARD_NAME,
        ],
    );

    let status_anchors = [
        STRUCTURE_SUPPORT_ROW_DATA_STATUS_NAME,
        STRUCTURE_SUPPORT_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/parent_route_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/map_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/foundation_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/review_route_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/typed_error_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/row_data_owner_rows.rs",
        STRUCTURE_SUPPORT_ROW_DATA_GUARD_NAME,
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
        "status map records structure-support row-data split",
        &status_map,
        &[
            STRUCTURE_SUPPORT_ROW_DATA_STATUS_NAME,
            STRUCTURE_SUPPORT_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records structure-support row-data split",
        &date_map,
        &[STRUCTURE_SUPPORT_ROW_DATA_STATUS_NAME, "2026-07-07"],
    );
}
