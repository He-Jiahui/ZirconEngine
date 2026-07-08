use super::*;

#[test]
fn runtime_15_review_guard_row_data_moved_rows_are_child_owned() {
    let review_guard_row_data_guard = read_runtime_src(REVIEW_GUARD_ROW_DATA_GUARD_PATH);
    let moved_rows_guard = read_runtime_src(MOVED_ROWS_GUARD_PATH);
    let moved_rows_children = moved_rows_child_source_blob();

    for moved_row_source in [
        concat!("Runtime 15 M3 code review findings ", "test folder split"),
        concat!(
            "Runtime 15 M3 code review findings ",
            "structure guard child-owner split"
        ),
        concat!(
            "Runtime 15 M3 code review findings ",
            "typed-error structure guard child-owner split"
        ),
        concat!(
            "Runtime 15 M3 code review findings ",
            "plugin-importer DX structure guard child-owner split"
        ),
        concat!(
            "Runtime 15 M3 native plugin loader ",
            "typed-error review guard child-owner split"
        ),
    ] {
        assert!(
            !review_guard_row_data_guard.contains(moved_row_source),
            "runtime_15_review_guard_row_data.rs should delegate moved-row assertion source {moved_row_source}"
        );
        assert!(
            moved_rows_children.contains(moved_row_source),
            "runtime_15_review_guard_row_data_moved_rows folder should own moved-row assertion source {moved_row_source}"
        );
    }
    assert_contains_all(
        "review-guard moved-row child records historical and folder-backed splits",
        &moved_rows_guard,
        &[
            "mod code_review_rows;",
            "mod delegation;",
            "mod status_mirrors;",
            "mod typed_error_rows;",
            "Runtime 15 M3 review-guard row-data moved-row guard child-owner split",
            "runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 review-guard moved-row guard folder-backed split",
            "runtime_15_review_guard_moved_row_guard_folder_backed_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "review-guard moved-row delegation child keeps historical guard name",
        &moved_rows_children,
        &[concat!(
            "fn runtime_15_status_output_m3_review_guard_",
            "row_data_moved_rows_are_child_owner"
        )],
    );
}
