use super::*;

#[test]
fn tab_navigation_uses_index_order_and_modal_group_trap() {
    let mut surface = navigation_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(3)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(2)));

    surface.focus_node(id(5)).unwrap();
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(6)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Previous,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn tab_navigation_crosses_non_modal_groups_by_group_order() {
    let mut surface = non_modal_group_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn directional_navigation_honors_manual_overrides_and_blocked_edges() {
    let mut surface = navigation_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Left,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn modal_directional_navigation_rejects_manual_targets_outside_modal_group() {
    let mut surface = navigation_surface();
    let modal_b = surface.tree.nodes.get_mut(&id(6)).unwrap();
    modal_b.navigation.directional = Some(UiDirectionalNavigation {
        right: UiDirectionalNavigationTarget::Node(id(2)),
        ..Default::default()
    });
    surface.focus_node(id(6)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(6)));
}
