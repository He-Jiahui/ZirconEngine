use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_child_owner_split() {
    let top_level_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps.rs",
    );
    let top_level_map_assertions = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions.rs",
    );
    let top_level_map_runtime_15_assertions = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/runtime_15_maps.rs",
    );
    let legacy_maps_guard_body = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/guard_body.rs",
    );
    let legacy_maps_guard_body_children = [
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/body/budgets.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/body/folder_backed.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/body/legacy_routes.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/body/paths.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps/body/status_mirrors.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n");
    let legacy_group_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_group_maps.rs",
    );

    assert_contains_all(
        "expected-slice maps child owns Runtime 15 split guard",
        &format!(
            "{top_level_maps}\n{top_level_map_assertions}\n{top_level_map_runtime_15_assertions}"
        ),
        &[
            "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
            "Runtime 15 status expected-slice topic owners preserve representative literals",
        ],
    );
    assert_contains_all(
        "expected-slice legacy guard body mounts focused children",
        &legacy_maps_guard_body,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/legacy_routes.rs\"]",
            "mod legacy_routes;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "expected-slice legacy guard body children own legacy split guard",
        &legacy_maps_guard_body_children,
        &[
            "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
            "runtime_15_status_output_expected_slice_legacy_guard_body_is_folder_backed",
            "legacy status/date children own pre-Runtime-15 branches",
        ],
    );
    assert_contains_all(
        "expected-slice legacy group child owns grouped legacy split guard",
        &legacy_group_maps,
        &[
            "fn runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners",
            "legacy expected-slice child groups own representative runtime ranges",
        ],
    );
}
