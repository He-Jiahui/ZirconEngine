use super::*;

#[test]
fn runtime_15_status_support_expected_slice_maps_are_child_owned() {
    let status_support_child = read_runtime_src(STATUS_SUPPORT_CHILD);
    let status_support_row_data_child = read_runtime_src(STATUS_SUPPORT_ROW_DATA_CHILD);
    let status_support_plan_doc_child = read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD);
    let date_support_child = read_runtime_src(DATE_SUPPORT_CHILD);
    let date_support_row_data_child = read_runtime_src(DATE_SUPPORT_ROW_DATA_CHILD);
    let date_support_plan_doc_child = read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD);
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "status-support expected-slice parents mount child maps",
        &format!("{status_support_child}\n{date_support_child}"),
        &[
            "#[path = \"status_support_maps/row_data_maps.rs\"]",
            "#[path = \"status_support_maps/plan_doc_support_maps.rs\"]",
            "row_data_maps::expected_status_for_slice(slice)",
            "plan_doc_support_maps::expected_status_for_slice(slice)",
            "row_data_maps::expected_date_for_slice(slice)",
            "plan_doc_support_maps::expected_date_for_slice(slice)",
        ],
    );

    for moved_literal in [
        "Runtime 15 M3 status output M3 row data child-owner split",
        "Runtime 15 M3 status output expected-slice legacy child-owner split",
        "Runtime 15 M3 priority plan docs row-data owner child split",
        "Runtime 15 M3 status-support row-data root inventory child split",
        "Runtime 15 M3 asset-budget row-data root inventory child split",
    ] {
        assert!(
            !status_support_child.contains(moved_literal),
            "status_support_maps.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_support_child.contains(moved_literal),
            "date status_support_maps.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "status-support expected-slice child maps own moved literals",
        &format!(
            "{status_support_row_data_child}\n{status_support_plan_doc_child}\n{date_support_row_data_child}\n{date_support_plan_doc_child}"
        ),
        &[
            "Runtime 15 M3 status-support expected-slice map child split",
            "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
            "Runtime 15 M3 status output M3 row data child-owner split",
            "runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 status output expected-slice legacy child-owner split",
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 priority plan docs root inventory child split",
            "runtime_15_priority_plan_docs_root_inventory_child_split_static_passed_cargo_deferred",
            "Runtime 15 M3 asset-budget row-data root inventory child split",
            "runtime_15_asset_budget_row_data_root_inventory_child_split_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
        ],
    );

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status-support expected-slice map child split",
                "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps.rs",
                "runtime_15_status_support_expected_slice_maps_are_child_owned",
                "Cargo gate blocked by render environment exports",
            ],
        );
    }
}
