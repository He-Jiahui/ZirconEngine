use super::*;

#[test]
fn runtime_15_status_support_route_metadata_row_data_is_folder_backed() {
    let parent = read_runtime_src(EXPECTED_SLICE_ROUTE_METADATA_PATH);
    let row_children = EXPECTED_SLICE_ROUTE_METADATA_CHILDREN
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
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "route metadata row-data parent mounts focused children",
        &parent,
        &[
            "#[path = \"route_meta/naming_boundary_rows.rs\"]",
            "#[path = \"route_meta/child_owner_guard_rows.rs\"]",
            "#[path = \"route_meta/child_owner_budget_rows.rs\"]",
            "#[path = \"route_meta/row_data_owner_rows.rs\"]",
            "row_data_owner_rows::EXPECTED_STATUS_OUTPUT_SLICES[0]",
        ],
    );
    assert!(
        !parent.contains(
            "Runtime 15 M3 naming-boundary expected-slice guard body folder-backed split"
        ),
        "route_metadata.rs should route row groups instead of retaining concrete row tuples"
    );
    assert_contains_all(
        "route metadata row-data children retain historical and current rows",
        &row_children,
        &[
            "Runtime 15 M3 naming-boundary expected-slice guard body folder-backed split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner budget source inventory folder-backed split",
            ROUTE_METADATA_ROW_DATA_STATUS_NAME,
            ROUTE_METADATA_ROW_DATA_STATUS_ID,
            ROUTE_METADATA_ROW_DATA_GUARD_NAME,
        ],
    );

    let status_anchors = [
        ROUTE_METADATA_ROW_DATA_STATUS_NAME,
        ROUTE_METADATA_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/naming_boundary_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/child_owner_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/child_owner_budget_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_meta/row_data_owner_rows.rs",
        ROUTE_METADATA_ROW_DATA_GUARD_NAME,
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
        "status map records route metadata row-data split",
        &status_map,
        &[
            ROUTE_METADATA_ROW_DATA_STATUS_NAME,
            ROUTE_METADATA_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records route metadata row-data split",
        &date_map,
        &[ROUTE_METADATA_ROW_DATA_STATUS_NAME, "2026-07-07"],
    );
}
