use super::*;

#[test]
fn runtime_15_status_output_review_guard_direct_assertion_rows_are_child_owner() {
    let review_guard_splits = read_runtime_src(REVIEW_GUARD_SPLITS_PATH);
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    let direct_assertion_rows = read_runtime_src(DIRECT_ASSERTION_ROWS_PATH);
    let plugin_importer_rows = read_runtime_src(PLUGIN_IMPORTER_ROWS_PATH);

    let direct_assertion_slices = [
        "Runtime 15 M3 code review findings direct assertions child-owner split",
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split",
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split",
        "Runtime 15 M3 code review findings render direct assertions child-owner split",
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split",
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split",
    ];
    for slice in direct_assertion_slices {
        assert!(
            direct_assertion_rows.contains(slice),
            "direct_assertion_rows.rs should own direct-assertion row literal {slice}"
        );
        assert!(
            !code_review_rows.contains(slice),
            "code_review_rows.rs should delegate direct-assertion row literal {slice}"
        );
        assert!(
            !review_guard_splits.contains(slice),
            "review_guard_splits.rs should remain route-only for direct-assertion row literal {slice}"
        );
    }
    assert_contains_all(
        "direct-assertion row-data child owns representative status anchors",
        &direct_assertion_rows,
        &[
            "runtime_15_code_review_findings_direct_assertions_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_f12_direct_assertions_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_render_direct_assertions_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_p0_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "plugin-importer row-data child owns representative status anchors",
        &plugin_importer_rows,
        &[
            "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split",
            "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split",
            "runtime_15_plugin_importer_d13_sdk_review_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
}
