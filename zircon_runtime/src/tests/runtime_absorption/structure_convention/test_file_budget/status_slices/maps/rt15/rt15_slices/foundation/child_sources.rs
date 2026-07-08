use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_are_folder_backed() {
    let status_parent = read_runtime_src(STATUS_PARENT);
    let date_parent = read_runtime_src(DATE_PARENT);
    for (label, parent) in [
        (
            "status runtime_15 foundation parent",
            status_parent.as_str(),
        ),
        ("date runtime_15 foundation parent", date_parent.as_str()),
    ] {
        for moved in [
            "Runtime 15 M3 core runtime lock poison guard child-owner split",
            "Runtime 15 F5 native plugin distribution compatibility typed errors",
            "Runtime 15 F13 provider registration shared owner",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate foundation slice row {moved}"
            );
        }
    }

    let status_children = read_plan_status_child_sources(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15",
    );
    let date_children = read_plan_status_child_sources(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15",
    );
    assert_contains_all(
        "status runtime_15 foundation children",
        &status_children,
        &[
            FOUNDATION_MAP_SLICE,
            FOUNDATION_MAP_STATUS,
            "runtime_15_core_runtime_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_native_plugin_distribution_compat_typed_errors_static_passed_cargo_deferred",
            "runtime_15_provider_registration_shared_owner_coremin_check_passed",
        ],
    );
    assert_contains_all(
        "date runtime_15 foundation children",
        &date_children,
        &[
            FOUNDATION_MAP_SLICE,
            "2026-07-07",
            "2026-06-25",
            "2026-06-29",
            "2026-06-22",
        ],
    );
}
