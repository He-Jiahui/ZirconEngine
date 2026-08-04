use super::super::*;
use super::*;

pub(super) fn assert_f8_route_ownership_guard_is_child_backed() {
    let route_parent = read_runtime_src(F8_ROUTE_OWNERSHIP_CHILD);
    let child_blob = format!(
        "{}\n{}",
        route_parent,
        f8_route_ownership_child_source_blob()
    );

    assert_contains_all(
        "F8 route ownership parent mounts focused child owners",
        &route_parent,
        &[
            "#[path = \"route_ownership/parent_routes.rs\"]",
            "#[path = \"route_ownership/descriptor_builder_routes.rs\"]",
            "#[path = \"route_ownership/descriptor_privacy_routes.rs\"]",
            "#[path = \"route_ownership/leaf_owners.rs\"]",
            "#[path = \"route_ownership/child_ownership.rs\"]",
            "#[path = \"route_ownership/status_mirrors.rs\"]",
            F8_ROUTE_PARENT_ROUTES_CHILD,
            F8_ROUTE_DESCRIPTOR_BUILDER_ROUTES_CHILD,
            F8_ROUTE_DESCRIPTOR_PRIVACY_ROUTES_CHILD,
            F8_ROUTE_LEAF_OWNERS_CHILD,
            F8_ROUTE_CHILD_OWNERSHIP_CHILD,
            F8_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS,
            F8_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD,
        ],
    );
    for moved_body in [
        "F8 API convergence parent mounts focused child owners",
        "F8 descriptor builder route mounts focused child owners",
        "F8 descriptor privacy route mounts focused child owners",
        "F8 texture child owns texture apply review guard",
        "F8 descriptor status child owns status mirror cleanup guard",
    ] {
        assert!(
            !route_parent.contains(moved_body),
            "route_ownership.rs should delegate moved assertion body `{moved_body}` to focused children"
        );
    }
    for (_, child_path, child_guard) in F8_ROUTE_OWNERSHIP_CHILDREN {
        assert!(
            route_parent.contains(child_path),
            "route_ownership.rs should inventory child owner path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "F8 route-ownership child source blob should contain child guard {child_guard}"
        );
    }
}
