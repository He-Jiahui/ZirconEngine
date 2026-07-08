use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_split_layout_routes_are_child_owned(
) {
    let split_layout = read_runtime_15_map("split_layout.rs");
    let split_layout_children = [
        read_runtime_15_map("split/sources.rs"),
        read_runtime_15_map("split/guard_body.rs"),
        read_runtime_15_map("split/status_mirrors.rs"),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 expected-slice split-layout route child",
        &split_layout,
        &[
            "#[path = \"split/sources.rs\"]",
            "mod sources;",
            "#[path = \"split/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"split/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use sources::*;",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice split-layout guard children",
        &split_layout_children,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}
