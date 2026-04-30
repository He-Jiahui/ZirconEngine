use super::support::*;

#[test]
fn shared_menu_pointer_bridge_opens_and_closes_top_level_menu_from_shared_hit_test() {
    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(default_menu_layout(), HostMenuPointerState::default());

    let opened = bridge.handle_click(UiPoint::new(20.0, 12.0)).unwrap();
    assert_eq!(opened.route, Some(HostMenuPointerRoute::MenuButton(0)));
    assert_eq!(opened.state.open_menu_index, Some(0));
    assert_eq!(opened.state.hovered_menu_index, Some(0));
    assert_eq!(opened.state.hovered_item_index, None);
    assert_eq!(opened.action_id, None);

    bridge.sync(default_menu_layout(), opened.state);
    let closed = bridge.handle_click(UiPoint::new(20.0, 12.0)).unwrap();
    assert_eq!(closed.route, Some(HostMenuPointerRoute::MenuButton(0)));
    assert_eq!(closed.state.open_menu_index, None);
    assert_eq!(closed.state.hovered_menu_index, None);
    assert_eq!(closed.state.hovered_item_index, None);
    assert_eq!(closed.action_id, None);
}

#[test]
fn shared_menu_pointer_bridge_resolves_popup_item_and_dismiss_overlay_routes() {
    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        default_menu_layout(),
        HostMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let item = bridge.handle_click(UiPoint::new(60.0, 72.0)).unwrap();
    assert_eq!(
        item.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 1,
            action_id: "SaveProject".to_string(),
        })
    );
    assert_eq!(item.action_id.as_deref(), Some("SaveProject"));
    assert_eq!(item.state.open_menu_index, None);
    assert_eq!(item.state.hovered_menu_index, None);
    assert_eq!(item.state.hovered_item_index, None);

    bridge.sync(
        default_menu_layout(),
        HostMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );
    let dismiss = bridge.handle_click(UiPoint::new(420.0, 260.0)).unwrap();
    assert_eq!(dismiss.route, Some(HostMenuPointerRoute::DismissOverlay));
    assert_eq!(dismiss.action_id, None);
    assert_eq!(dismiss.state.open_menu_index, None);
}

#[test]
fn shared_menu_pointer_bridge_scrolls_window_popup_using_shared_scroll_state() {
    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        window_menu_layout(10),
        HostMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let moved = bridge.handle_move(UiPoint::new(240.0, 110.0)).unwrap();
    assert_eq!(
        moved.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 4,
            item_index: 2,
            action_id: "LoadPreset.alpha-00".to_string(),
        })
    );
    assert_eq!(moved.state.hovered_menu_index, Some(4));
    assert_eq!(moved.state.hovered_item_index, Some(2));

    bridge.sync(window_menu_layout(10), moved.state);
    let scrolled = bridge
        .handle_scroll(UiPoint::new(240.0, 110.0), 96.0)
        .unwrap();
    assert_eq!(scrolled.route, Some(HostMenuPointerRoute::PopupSurface(4)));
    assert!(scrolled.state.popup_scroll_offset > 0.0);
    assert_eq!(scrolled.action_id, None);
    assert_eq!(scrolled.state.open_menu_index, Some(4));
}

#[test]
fn shared_menu_pointer_bridge_recomputes_hovered_item_after_window_popup_scroll() {
    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        window_menu_layout(20),
        HostMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let scrolled = bridge
        .handle_scroll(UiPoint::new(240.0, 110.0), 420.0)
        .unwrap();

    assert_eq!(scrolled.state.hovered_menu_index, Some(4));
    assert_eq!(
        scrolled.state.hovered_item_index,
        Some(16),
        "scroll should re-hover the absolute Window menu row now under the pointer"
    );
}

#[test]
fn shared_menu_pointer_bridge_dismiss_keeps_window_popup_scroll_state() {
    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        window_menu_layout(10),
        HostMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 96.0,
        },
    );

    let dismissed = bridge.handle_click(UiPoint::new(520.0, 260.0)).unwrap();
    assert_eq!(dismissed.route, Some(HostMenuPointerRoute::DismissOverlay));
    assert_eq!(dismissed.action_id, None);
    assert_eq!(dismissed.state.open_menu_index, None);
    assert_eq!(dismissed.state.hovered_menu_index, None);
    assert_eq!(dismissed.state.hovered_item_index, None);
    assert_eq!(dismissed.state.popup_scroll_offset, 96.0);
}
