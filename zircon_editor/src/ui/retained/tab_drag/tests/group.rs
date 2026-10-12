use super::*;

#[test]
fn typed_routes_match_their_materialized_group_keys() {
    let window_id = MainPageId::new("floating/alpha");
    let routes = [
        HostShellPointerRoute::DragTarget(HostDragTargetGroup::Left),
        HostShellPointerRoute::DocumentEdge(DockEdge::Bottom),
        HostShellPointerRoute::FloatingWindow(window_id.clone()),
        HostShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Right,
        },
    ];

    for route in routes {
        let key =
            host_shell_pointer_route_group_key(&route).expect("drag route should have a group key");
        assert!(host_shell_pointer_route_matches_group_key(&route, &key));
    }
}

#[test]
fn typed_route_match_rejects_other_groups_and_partial_floating_ids() {
    let route = HostShellPointerRoute::FloatingWindowEdge {
        window_id: MainPageId::new("alpha/beta"),
        edge: DockEdge::Top,
    };

    assert!(!host_shell_pointer_route_matches_group_key(
        &route,
        "floating-window-edge/alpha/top"
    ));
    assert!(!host_shell_pointer_route_matches_group_key(
        &route,
        "floating-window-edge/alpha/beta/bottom"
    ));
}
