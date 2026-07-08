use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned() {
    let structure_parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_PARENT);

    assert_contains_all(
        "review_guard_maps.rs mounts expected-slice guard children",
        &structure_parent,
        &[
            "#[path = \"review/sources.rs\"]",
            "mod sources;",
            "#[path = \"review/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"review/route_metadata.rs\"]",
            "mod route_metadata;",
            "#[path = \"review/structure_support_expected_slice.rs\"]",
            "mod structure_support_expected_slice;",
            "#[path = \"review/status_support_expected_slice.rs\"]",
            "mod status_support_expected_slice;",
            "#[path = \"review/typed_error_expected_slice.rs\"]",
            "mod typed_error_expected_slice;",
            "#[path = \"review/review_route_children.rs\"]",
            "mod review_route_children;",
            "use sources::*;",
        ],
    );
    for moved_anchor in [
        concat!(
            "fn ",
            "runtime_15_structure_support_expected_slice_maps_are_child_owners"
        ),
        concat!(
            "M3 structure-support",
            " status expected-slice parent mounts map children"
        ),
        concat!(
            "Runtime 15 M3 structure-support",
            " expected-slice map child-owner split"
        ),
    ] {
        assert!(
            !structure_parent.contains(moved_anchor),
            "review_guard_maps.rs should mount structure_support_expected_slice instead of keeping {moved_anchor}"
        );
    }
}
