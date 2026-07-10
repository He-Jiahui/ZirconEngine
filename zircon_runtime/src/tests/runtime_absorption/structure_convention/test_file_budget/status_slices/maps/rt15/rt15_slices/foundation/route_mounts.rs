use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_route_mounts_are_child_owned() {
    let expected_slice_maps_parent = read_runtime_src(EXPECTED_SLICE_MAPS_PARENT);
    let foundation_parent = read_runtime_src(FOUNDATION_ROUTE_PARENT);
    let status_parent = read_runtime_src(STATUS_PARENT);
    let date_parent = read_runtime_src(DATE_PARENT);

    assert_contains_all(
        "Runtime 15 expected-slice maps parent mounts foundation route",
        &expected_slice_maps_parent,
        &["#[path = \"rt15_slices/foundation.rs\"]", "mod foundation;"],
    );
    assert_contains_all(
        "foundation expected-slice map guard parent mounts focused children",
        &foundation_parent,
        &[
            "#[path = \"foundation/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"foundation/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"foundation/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"foundation/paths.rs\"]",
            "mod paths;",
            "#[path = \"foundation/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"foundation/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );
    for parent in [status_parent.as_str(), date_parent.as_str()] {
        assert_contains_all(
            "plan-status foundation parent routes map children",
            parent,
            &[
                "#[path = \"foundation/asset_provider_cleanup.rs\"]",
                "#[path = \"foundation/core_cleanup.rs\"]",
                "#[path = \"foundation/graphics_diagnostics.rs\"]",
                "#[path = \"foundation/lock_poison.rs\"]",
                "#[path = \"foundation/map_rows.rs\"]",
                "#[path = \"foundation/typed_error_core.rs\"]",
                "#[path = \"foundation/typed_error_plugin.rs\"]",
            ],
        );
    }
}
