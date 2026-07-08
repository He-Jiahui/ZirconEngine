use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_literals_are_child_owned() {
    let status_review_child = read_runtime_src(STATUS_REVIEW_CHILD);
    let date_review_child = read_runtime_src(DATE_REVIEW_CHILD);
    let status_typed_error_child = read_status_review_typed_error_sources();
    let date_typed_error_child = read_date_review_typed_error_sources();

    for moved_literal in [
        "Runtime 15 M3 typed-error convergence guard child-owner split",
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
        "Runtime 15 M3 typed-error source inventory guard child-owner split",
        "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
    ] {
        assert!(
            !status_review_child.contains(moved_literal),
            "status review_guard_maps.rs should delegate typed-error literal {moved_literal}"
        );
        assert!(
            !date_review_child.contains(moved_literal),
            "date review_guard_maps.rs should delegate typed-error literal {moved_literal}"
        );
    }
    assert_contains_all(
        "typed-error expected-slice map children own typed-error literals",
        &format!("{status_typed_error_child}\n{date_typed_error_child}"),
        &[
            "Runtime 15 M3 typed-error convergence guard child-owner split",
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
            "runtime_15_native_plugin_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 typed-error source inventory guard child-owner split",
            "runtime_15_typed_error_source_inventory_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split",
            "runtime_15_native_live_host_replay_runtime_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-30\")",
        ],
    );
}
