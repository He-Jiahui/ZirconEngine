use super::*;

#[test]
fn runtime_15_expected_slice_module_layout_guard_body_routes_are_child_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{EXPECTED_SLICES_PARENT}"
    ));
    let module_layout =
        read_runtime_src(&format!("tests/runtime_absorption/{MODULE_LAYOUT_PARENT}"));
    let legacy_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps.rs",
    );
    let status_scan = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
    );

    assert_contains_all(
        "expected-slice module-layout parent mounts guard body child",
        &module_layout,
        &[
            "#[path = \"module_layout/guard_body.rs\"]",
            "mod guard_body;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_status_output_expected_slice_guard_child_owner_split",
        "let parent = read_runtime_src(",
        "expected-slice guard parent mounts child guard owners",
    ] {
        assert!(
            !module_layout.contains(moved_anchor),
            "status_slices/module_layout.rs should mount guard_body instead of keeping {moved_anchor}"
        );
    }

    assert_contains_all(
        "expected-slice guard parent mounts child guard owners",
        &parent,
        &[
            "#[path = \"status_slices/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"status_slices/maps.rs\"]",
            "mod maps;",
            "#[path = \"status_slices/legacy_maps.rs\"]",
            "mod legacy_maps;",
            "#[path = \"status_slices/legacy_group_maps.rs\"]",
            "mod legacy_group_maps;",
        ],
    );
    for moved_guard in [
        "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
        "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
        "fn runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "status_output_expected_slices.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "expected-slice legacy parent mounts guard body child",
        &legacy_maps,
        &["#[path = \"legacy_maps/guard_body.rs\"]", "mod guard_body;"],
    );
    for moved_anchor in [
        "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
        "legacy status/date children own pre-Runtime-15 branches",
        "Runtime 14 Cargo 验证窗口探测",
    ] {
        assert!(
            !legacy_maps.contains(moved_anchor),
            "status_slices/legacy_maps.rs should mount guard_body instead of keeping {moved_anchor}"
        );
    }

    assert_contains_all(
        "root-layout status scan includes expected-slice guard children",
        &status_scan,
        &[
            "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/module_layout.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/module_layout/guard_body.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_group_maps.rs",
        ],
    );
}
