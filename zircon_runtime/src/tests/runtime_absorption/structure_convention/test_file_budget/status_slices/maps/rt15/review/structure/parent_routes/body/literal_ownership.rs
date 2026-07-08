use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_literals_are_child_owned() {
    let status_parent = read_runtime_src(STATUS_PARENT);
    let date_parent = read_runtime_src(DATE_PARENT);
    let child_blob = format!(
        "{}\n{}\n{}\n{}",
        read_sources(STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN),
        read_sources(DATE_STRUCTURE_PARENT_ROUTE_CHILDREN),
        read_status_structure_route_map_sources(),
        read_date_structure_route_map_sources()
    );

    for moved_literal in [
        "Runtime 15 M3 foundation-guards row-data owner child split",
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard",
        "Runtime 15 M3 UI runtime input reply route guard child-owner split",
        "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split",
        "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split",
        "Runtime 15 M3 scene property paths test folder split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status m3_structure_support.rs should delegate moved literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date m3_structure_support.rs should delegate moved literal {moved_literal}"
        );
    }

    assert_contains_all(
        "structure-support route children own moved parent literals",
        &child_blob,
        &[
            "Runtime 15 M3 structure-support expected-slice parent maps folder-backed split",
            "runtime_15_structure_support_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
            "Some(\"2026-07-05\")",
            "Runtime 15 M3 foundation-guards row-data owner child split",
            "Runtime 15 M3 graphics dead-code guard module split",
            "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard",
            "Runtime 15 M3 UI runtime input reply route guard child-owner split",
            "Runtime 15 M3 asset project zmeta current 12-test guard sync",
            "Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split",
            "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split",
            "Runtime 15 M3 scene property paths test folder split",
        ],
    );
}
