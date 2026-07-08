use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_children_own_moved_checks() {
    let structure_support_guard_child =
        read_runtime_src(STRUCTURE_REVIEW_STRUCTURE_SUPPORT_GUARD_CHILD);
    let structure_support_literal =
        read_runtime_src(STRUCTURE_REVIEW_STRUCTURE_SUPPORT_LITERAL_CHILD);
    let structure_support_literal_children =
        read_review_root_sources(STRUCTURE_REVIEW_STRUCTURE_SUPPORT_LITERAL_CHILDREN);
    let structure_support_split_layout =
        read_runtime_src(STRUCTURE_REVIEW_STRUCTURE_SUPPORT_SPLIT_LAYOUT_CHILD);
    let typed_error_guard_child = read_runtime_src(STRUCTURE_REVIEW_TYPED_ERROR_GUARD_CHILD);
    let typed_error_guard_body = read_runtime_src(STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY);
    let typed_error_guard_body_children =
        read_review_root_sources(STRUCTURE_REVIEW_TYPED_ERROR_GUARD_BODY_CHILDREN);
    let status_support_guard_child = read_runtime_src(STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_CHILD);
    let status_support_guard_body = read_runtime_src(STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_BODY);
    let status_support_guard_body_children =
        read_review_root_sources(STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_BODY_CHILDREN);
    let route_guard_child = read_runtime_src(STRUCTURE_REVIEW_ROUTE_CHILD);
    let route_guard_body = read_runtime_src(STRUCTURE_REVIEW_ROUTE_GUARD_BODY);
    let route_guard_body_children =
        read_review_root_sources(STRUCTURE_REVIEW_ROUTE_GUARD_BODY_CHILDREN);

    assert_contains_all(
        "expected-slice guard children own moved tests",
        &format!(
            "{structure_support_guard_child}\n{structure_support_literal}\n{structure_support_literal_children}\n{structure_support_split_layout}\n{typed_error_guard_child}\n{typed_error_guard_body}\n{typed_error_guard_body_children}\n{status_support_guard_child}\n{status_support_guard_body}\n{status_support_guard_body_children}\n{route_guard_child}\n{route_guard_body}\n{route_guard_body_children}"
        ),
        &[
            "runtime_15_structure_support_expected_slice_maps_are_child_owners",
            "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
            "runtime_15_status_support_expected_slice_maps_are_child_owned",
            "runtime_15_review_guard_expected_slice_maps_are_folder_backed",
            concat!(
                "Runtime 15 M3 structure-support",
                " expected-slice map child-owner split"
            ),
            "Runtime 15 M3 review guard typed-error expected-slice map child split",
            "Runtime 15 M3 status-support expected-slice map child split",
            "Runtime 15 M3 review-guard expected-slice maps folder-backed split",
        ],
    );
}
