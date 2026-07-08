use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_maps_is_folder_backed() {
    let parent = read_runtime_src(MAPS_PARENT_PATH);
    let guard_body = read_runtime_src(GUARD_BODY_PARENT_PATH);
    let child_sources = GUARD_BODY_CHILD_PATHS
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !parent.contains(TEST_ATTRIBUTE),
        "maps.rs should stay route-only and should not define guard tests"
    );
    assert!(
        !guard_body.contains(TEST_ATTRIBUTE),
        "maps/guard_body.rs should stay route-only and should not define guard tests"
    );
    for moved_anchor in [
        "let runtime_15_plan =",
        "let review_findings =",
        "let line_count =",
        "status-output expected-slice map children preserve guards",
        "Runtime 15 M3 runtime-15 expected-slice topic guard child-module split",
    ] {
        assert!(
            !parent.contains(moved_anchor) && !guard_body.contains(moved_anchor),
            "route owners should not keep moved guard body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-output maps folder-backed child tree keeps actual guard checks",
        &child_sources,
        &[
            "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
            "runtime_15_status_output_expected_slice_guard_maps_children_preserve_guard_owners",
            "runtime_15_status_output_expected_slice_guard_maps_status_is_synced",
            "runtime_15_status_output_expected_slice_guard_maps_children_stay_budgeted",
            MAPS_GUARD_BODY_GUARD,
        ],
    );
}
