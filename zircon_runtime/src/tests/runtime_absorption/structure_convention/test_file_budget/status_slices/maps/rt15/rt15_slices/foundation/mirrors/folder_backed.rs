use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_status_mirrors_are_folder_backed() {
    let parent = read_foundation_status_mirror_parent();
    let children = read_foundation_status_mirror_children();

    for moved_anchor in [
        "#[test]",
        "let row_data = read_top_level_support_row_sources()",
        "Runtime 15 foundation expected-slice map row data",
        "Frameworks 02 foundation expected-slice mirrors",
        FOUNDATION_GUARD_FRAMEWORKS_STATUS,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "foundation/status_mirrors.rs should delegate moved mirror anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "foundation status mirror children keep moved checks",
        &children,
        &[
            "runtime_15_foundation_expected_slice_maps_status_mirrors_are_synced",
            "runtime_15_foundation_expected_slice_maps_docs_are_synced",
            "runtime_15_foundation_expected_slice_maps_status_mirrors_are_folder_backed",
            "runtime_15_foundation_expected_slice_maps_status_mirror_children_stay_budgeted",
            FOUNDATION_STATUS_MIRRORS_GUARD,
        ],
    );
}
