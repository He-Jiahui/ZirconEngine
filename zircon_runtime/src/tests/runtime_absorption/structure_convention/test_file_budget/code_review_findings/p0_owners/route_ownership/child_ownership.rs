use super::super::*;
use super::*;

pub(super) fn assert_p0_route_ownership_guard_is_child_backed() {
    let route_parent = read_runtime_src(P0_ROUTE_OWNERSHIP_CHILD);
    let child_blob = p0_route_ownership_child_source_blob();

    assert_contains_all(
        "P0 route ownership parent mounts focused child owners",
        &route_parent,
        &[
            "#[path = \"route_ownership/parent_routes.rs\"]",
            "#[path = \"route_ownership/leaf_owners.rs\"]",
            "#[path = \"route_ownership/child_ownership.rs\"]",
            "#[path = \"route_ownership/status_mirrors.rs\"]",
            P0_ROUTE_PARENT_ROUTES_CHILD,
            P0_ROUTE_LEAF_OWNERS_CHILD,
            P0_ROUTE_CHILD_OWNERSHIP_CHILD,
            P0_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS,
            P0_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD,
        ],
    );
    for moved_body in [
        "P0 robustness parent mounts focused child owners",
        "native host callback child owns F1 panic-boundary review guard",
        "lock poison child owns F2 scene/EventBus review guard",
        "render submit child owns F4 viewport/provider typed-error review guard",
        "native fixture child owns fixture review sync guards",
        "native fixture SDK macro leaf owns D-S8/D3 review sync guard",
        "native fixture importer leaf owns D13 manifest self-description guard",
        "priority recommendation child owns cross-review priority sync",
    ] {
        assert!(
            !route_parent.contains(moved_body),
            "route_ownership.rs should delegate moved assertion body `{moved_body}` to focused children"
        );
    }
    for (_, child_path, child_guard) in P0_ROUTE_OWNERSHIP_CHILDREN {
        assert!(
            route_parent.contains(child_path),
            "route_ownership.rs should inventory child owner path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "P0 route-ownership child source blob should contain child guard {child_guard}"
        );
    }
}
