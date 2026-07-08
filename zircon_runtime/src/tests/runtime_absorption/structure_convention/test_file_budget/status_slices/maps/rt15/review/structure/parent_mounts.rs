use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_mounts_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_PARENT);
    let date_parent = read_runtime_src(DATE_PARENT);

    assert_contains_all(
        "M3 structure-support status expected-slice parent mounts map children",
        &status_parent,
        &[
            "#[path = \"m3_structure_support/review_guard_maps.rs\"]",
            "mod review_guard_maps;",
            "#[path = \"m3_structure_support/naming_guard_maps.rs\"]",
            "mod naming_guard_maps;",
            "#[path = \"m3_structure_support/status_support_maps.rs\"]",
            "mod status_support_maps;",
            "review_guard_maps::expected_status_for_slice(slice)",
            "naming_guard_maps::expected_status_for_slice(slice)",
            "status_support_maps::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "M3 structure-support date expected-slice parent mounts map children",
        &date_parent,
        &[
            "#[path = \"m3_structure_support/review_guard_maps.rs\"]",
            "mod review_guard_maps;",
            "#[path = \"m3_structure_support/naming_guard_maps.rs\"]",
            "mod naming_guard_maps;",
            "#[path = \"m3_structure_support/status_support_maps.rs\"]",
            "mod status_support_maps;",
            "review_guard_maps::expected_date_for_slice(slice)",
            "naming_guard_maps::expected_date_for_slice(slice)",
            "status_support_maps::expected_date_for_slice(slice)",
        ],
    );

    for moved_literal in [
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
        "Runtime 15 M3 graphics GPU-model guard child-owner split",
        "Runtime 15 M3 status output expected-slice guard child-owner split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "M3 structure-support status parent should delegate {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "M3 structure-support date parent should delegate {moved_literal}"
        );
    }
}
