use super::*;

#[test]
fn runtime_15_review_guard_code_review_structure_status_mirrors_are_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let structure_guard_row_data_owner = read_runtime_src(STRUCTURE_GUARD_ROW_DATA_OWNER_PATH);
    let review_status_map = review_guard_status_map_source_blob();
    let review_date_map = review_guard_date_map_source_blob();

    let structure_guard_status_anchors = [
        STRUCTURE_GUARD_ROW_DATA_STATUS_NAME,
        STRUCTURE_GUARD_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/status_docs.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/folder_backed_summary.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/typed_error.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/row_data_owner.rs",
        STRUCTURE_GUARD_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "structure-guard row-data owner",
            structure_guard_row_data_owner.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &structure_guard_status_anchors);
    }
    assert_contains_all(
        "Runtime 15 review-guard status map",
        &review_status_map,
        &[
            STRUCTURE_GUARD_ROW_DATA_STATUS_NAME,
            STRUCTURE_GUARD_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 review-guard date map",
        &review_date_map,
        &[STRUCTURE_GUARD_ROW_DATA_STATUS_NAME, "2026-07-02"],
    );

    let root_and_children_status_anchors = [
        ROOT_AND_CHILDREN_ROW_DATA_STATUS_NAME,
        ROOT_AND_CHILDREN_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/code_review_findings.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_robustness.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/plugin_importer_dx.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/p0_native_fixture.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/late_api_cleanup.rs",
        ROOT_AND_CHILDREN_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "structure-guard row-data owner",
            structure_guard_row_data_owner.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &root_and_children_status_anchors);
    }
    assert_contains_all(
        "Runtime 15 review-guard status map records root-and-children split",
        &review_status_map,
        &[
            ROOT_AND_CHILDREN_ROW_DATA_STATUS_NAME,
            ROOT_AND_CHILDREN_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 review-guard date map records root-and-children split",
        &review_date_map,
        &[ROOT_AND_CHILDREN_ROW_DATA_STATUS_NAME, "2026-07-03"],
    );
}
