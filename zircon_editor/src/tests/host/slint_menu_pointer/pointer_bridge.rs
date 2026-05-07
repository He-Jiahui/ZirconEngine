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
            ..HostMenuPointerState::default()
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
            ..HostMenuPointerState::default()
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
            open_menu_index: Some(5),
            hovered_menu_index: Some(5),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let moved = bridge.handle_move(UiPoint::new(280.0, 110.0)).unwrap();
    assert_eq!(
        moved.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 5,
            item_index: 2,
            action_id: "LoadPreset.alpha-00".to_string(),
        })
    );
    assert_eq!(moved.state.hovered_menu_index, Some(5));
    assert_eq!(moved.state.hovered_item_index, Some(2));

    bridge.sync(window_menu_layout(10), moved.state);
    let scrolled = bridge
        .handle_scroll(UiPoint::new(280.0, 110.0), 96.0)
        .unwrap();
    assert_eq!(scrolled.route, Some(HostMenuPointerRoute::PopupSurface(5)));
    assert!(scrolled.state.popup_scroll_offset > 0.0);
    assert_eq!(scrolled.action_id, None);
    assert_eq!(scrolled.state.open_menu_index, Some(5));
}

#[test]
fn shared_menu_pointer_bridge_scrolls_overwide_menu_bar_to_extension_button() {
    let mut layout = default_menu_layout();
    layout.shell_frame = UiFrame::new(0.0, 0.0, 180.0, 120.0);
    layout.button_frames = (0..9)
        .map(|index| UiFrame::new(8.0 + index as f32 * 52.0, 2.0, 50.0, 22.0))
        .collect();
    layout.menu_bar_content_width = layout
        .button_frames
        .last()
        .map(|frame| frame.x + frame.width)
        .unwrap_or_default();
    layout.menus = (0..9)
        .map(|index| {
            vec![MenuItemSpec {
                action_id: Some(format!("Extension.Menu{index}")),
                enabled: true,
                children: Vec::new(),
            }]
        })
        .collect();

    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(layout.clone(), HostMenuPointerState::default());

    let scrolled = bridge
        .handle_scroll(UiPoint::new(20.0, 12.0), 280.0)
        .unwrap();
    assert!(scrolled.state.menu_bar_scroll_offset > 0.0);

    bridge.sync(layout, scrolled.state);
    let opened = bridge.handle_click(UiPoint::new(150.0, 12.0)).unwrap();
    assert_eq!(opened.route, Some(HostMenuPointerRoute::MenuButton(8)));
    assert_eq!(
        opened.state.open_menu_index,
        Some(8),
        "scrolling the overwide top menu bar should make extension menu buttons reachable through the shared hit surface"
    );
}

#[test]
fn shared_menu_pointer_bridge_recomputes_hovered_item_after_window_popup_scroll() {
    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        window_menu_layout(20),
        HostMenuPointerState {
            open_menu_index: Some(5),
            hovered_menu_index: Some(5),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let scrolled = bridge
        .handle_scroll(UiPoint::new(280.0, 110.0), 420.0)
        .unwrap();

    assert_eq!(scrolled.state.hovered_menu_index, Some(5));
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
            open_menu_index: Some(5),
            hovered_menu_index: Some(5),
            hovered_item_index: None,
            popup_scroll_offset: 96.0,
            ..HostMenuPointerState::default()
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

#[test]
fn shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell() {
    let mut layout = default_menu_layout();
    layout.shell_frame = UiFrame::new(0.0, 0.0, 120.0, 50.0);
    layout.button_frames[0] = UiFrame::new(100.0, 2.0, 52.0, 22.0);
    layout.menus = vec![overflow_menu_items(10)];

    let open_state = HostMenuPointerState {
        open_menu_index: Some(0),
        hovered_menu_index: Some(0),
        hovered_item_index: None,
        popup_scroll_offset: 0.0,
        ..HostMenuPointerState::default()
    };

    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(layout.clone(), open_state.clone());

    let outside_width = bridge.handle_click(UiPoint::new(140.0, 42.0)).unwrap();
    assert_eq!(outside_width.route, None);
    assert_eq!(outside_width.action_id, None);
    assert_eq!(outside_width.state.open_menu_index, None);

    bridge.sync(layout, open_state);
    let outside_height = bridge.handle_click(UiPoint::new(20.0, 60.0)).unwrap();
    assert_eq!(outside_height.route, None);
    assert_eq!(outside_height.action_id, None);
    assert_eq!(outside_height.state.open_menu_index, None);
}

#[test]
fn shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp() {
    let mut layout = default_menu_layout();
    layout.shell_frame = UiFrame::new(0.0, 0.0, 420.0, 260.0);
    layout.button_frames[0] = UiFrame::new(360.0, 2.0, 52.0, 22.0);
    layout.menu_overflow_mode = MenuOverflowMode::MultiColumn;
    layout.menus = vec![overflow_menu_items(18)];

    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        layout,
        HostMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let column_two_first_item = bridge.handle_click(UiPoint::new(230.0, 42.0)).unwrap();

    assert_eq!(
        column_two_first_item.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 9,
            action_id: "Overflow.Action09".to_string(),
        }),
        "multi-column popup hit testing should preserve absolute item indices after the popup is clamped into the shell"
    );
    assert_eq!(
        column_two_first_item.action_id.as_deref(),
        Some("Overflow.Action09")
    );
    assert_eq!(column_two_first_item.state.open_menu_index, None);
}

#[test]
fn shared_menu_pointer_bridge_opens_flipped_nested_popup_for_branch_hover() {
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let menu_bar = MenuBarModel {
        menus: vec![MenuModel {
            label: "Tools".to_string(),
            items: vec![MenuItemModel::branch(
                "Weather",
                vec![MenuItemModel::leaf(
                    "Refresh Clouds",
                    None,
                    EditorUiBinding::new(
                        "WorkbenchMenuBar",
                        operation_path.as_str(),
                        EditorUiEventKind::Click,
                        EditorUiBindingPayload::editor_operation(operation_path.as_str()),
                    ),
                    Some(operation_path.clone()),
                    Some("Ctrl+Alt+R".to_string()),
                    true,
                )],
            )],
        }],
    };
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_nested_popup_hit");
    let mut layout = build_host_menu_pointer_layout(
        &menu_bar,
        &harness.runtime.chrome_snapshot(),
        UiSize::new(420.0, 260.0),
        &[],
        None,
        None,
    );
    layout.shell_frame = UiFrame::new(0.0, 0.0, 420.0, 260.0);
    layout.button_frames[0] = UiFrame::new(360.0, 2.0, 52.0, 22.0);

    let mut bridge = HostMenuPointerBridge::new();
    bridge.sync(
        layout.clone(),
        HostMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let branch_hover = bridge.handle_move(UiPoint::new(250.0, 42.0)).unwrap();
    assert_eq!(
        branch_hover.route,
        Some(HostMenuPointerRoute::SubmenuBranch {
            menu_index: 0,
            item_index: 0,
        })
    );
    assert_eq!(
        branch_hover.state.open_submenu_path,
        vec![0],
        "hovering a branch should keep its child popup open"
    );

    bridge.sync(layout, branch_hover.state.clone());
    let nested_leaf = bridge.handle_click(UiPoint::new(70.0, 42.0)).unwrap();

    assert_eq!(
        nested_leaf.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 1,
            action_id: "Weather.CloudLayer.Refresh".to_string(),
        }),
        "the nested popup should flip left near the shell edge and keep legacy flattened item indices for dispatch"
    );
    assert_eq!(
        nested_leaf.action_id.as_deref(),
        Some("Weather.CloudLayer.Refresh")
    );
    assert_eq!(nested_leaf.state.open_menu_index, None);
}

fn overflow_menu_items(count: usize) -> Vec<MenuItemSpec> {
    (0..count)
        .map(|index| MenuItemSpec {
            action_id: Some(format!("Overflow.Action{index:02}")),
            enabled: true,
            children: Vec::new(),
        })
        .collect()
}
