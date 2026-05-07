use super::support::*;

#[test]
fn shared_menu_pointer_click_dispatches_reset_layout_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_reset_layout");
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = HostMenuPointerBridge::new();
    pointer_bridge.sync(default_menu_layout(), HostMenuPointerState::default());

    let opened = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(20.0, 12.0),
    )
    .expect("shared pointer route should open the file menu");
    assert_eq!(
        opened.pointer.route,
        Some(HostMenuPointerRoute::MenuButton(0))
    );
    assert!(opened.effects.is_none());

    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(60.0, 126.0),
    )
    .expect("shared pointer route should dispatch reset layout");
    assert_eq!(
        dispatched.pointer.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 3,
            action_id: "ResetLayout".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("menu item click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)
    );
}

#[test]
fn shared_menu_pointer_click_dispatches_scrolled_window_preset_selection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_load_preset");
    for index in 0..20 {
        dispatch_menu_action(&harness.runtime, &format!("SavePreset.alpha-{index:02}"))
            .expect("preset save setup should succeed");
    }
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = HostMenuPointerBridge::new();
    let layout = window_menu_layout(20);
    pointer_bridge.sync(
        layout.clone(),
        HostMenuPointerState {
            open_menu_index: Some(5),
            hovered_menu_index: Some(5),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(280.0, 110.0), 420.0)
        .expect("window popup should accept shared scroll input");
    assert!(scrolled.state.popup_scroll_offset > 0.0);

    pointer_bridge.sync(layout, scrolled.state.clone());
    let item_index = 17usize;
    let click_y = 32.0 + item_index as f32 * 30.0 - scrolled.state.popup_scroll_offset + 14.0;
    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(280.0, click_y),
    )
    .expect("shared pointer route should dispatch a scrolled preset selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 5,
            item_index,
            action_id: "LoadPreset.alpha-15".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("preset click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::LoadPreset {
            name: "alpha-15".to_string(),
        })
    );
}

#[test]
fn shared_menu_pointer_click_dispatches_editor_operation_payloads_from_extension_menu_items() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_editor_operation_dispatch");
    let operation_path = EditorOperationPath::parse("View.Weather.Open").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Open Weather")
                .with_event(EditorEvent::Layout(LayoutCommand::ActivateMainPage {
                    page_id: MainPageId::new("weather"),
                }))
                .with_undoable(crate::core::editor_operation::UndoableEditorOperation::new(
                    "Open Weather",
                )),
        )
        .expect("test operation should register in extension");
    harness
        .runtime
        .register_editor_extension(extension)
        .expect("test extension should register operation in runtime");

    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = HostMenuPointerBridge::new();
    let mut layout = default_menu_layout();
    layout.menus = vec![vec![crate::ui::slint_host::menu_pointer::MenuItemSpec {
        action_id: Some(operation_path.to_string()),
        enabled: true,
        children: Vec::new(),
    }]];
    pointer_bridge.sync(
        layout,
        HostMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(18.0, 42.0),
    )
    .expect("editor operation menu action should dispatch through operation runtime");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 0,
            action_id: "View.Weather.Open".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("operation-backed menu item should dispatch into runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    let stack = harness.runtime.operation_stack();
    assert_eq!(
        stack.undo_stack().last().map(|entry| entry.operation_id.as_str()),
        Some("View.Weather.Open"),
        "menu dispatch should invoke the EditorOperation id instead of parsing it as a legacy MenuAction"
    );
}

#[test]
fn shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_nested_operation_dispatch");
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::Layout(LayoutCommand::ActivateMainPage {
                    page_id: MainPageId::new("weather"),
                }))
                .with_undoable(crate::core::editor_operation::UndoableEditorOperation::new(
                    "Refresh Cloud Layers",
                )),
        )
        .expect("test operation should register in extension");
    harness
        .runtime
        .register_editor_extension(extension)
        .expect("test extension should register operation in runtime");

    let menu_bar = MenuBarModel {
        menus: vec![MenuModel {
            label: "Tools".to_string(),
            items: vec![MenuItemModel::branch(
                "Weather",
                vec![MenuItemModel::leaf(
                    "Refresh Cloud Layers",
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
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = HostMenuPointerBridge::new();
    let layout = build_host_menu_pointer_layout(
        &menu_bar,
        &harness.runtime.chrome_snapshot(),
        UiSize::new(1280.0, 720.0),
        &[],
        None,
        None,
    );
    let branch_x = layout.button_frames[0].x + 16.0;
    let branch_y = layout.button_frames[0].y + layout.button_frames[0].height + 3.0 + 6.0 + 14.0;
    let submenu_leaf_x = layout.button_frames[0].x + 208.0 + 16.0;
    let submenu_leaf_y = branch_y;
    pointer_bridge.sync(
        layout.clone(),
        HostMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
            ..HostMenuPointerState::default()
        },
    );

    let hovered_branch = pointer_bridge
        .handle_move(UiPoint::new(branch_x, branch_y))
        .expect("nested branch should open a child popup on hover");
    assert_eq!(
        hovered_branch.route,
        Some(HostMenuPointerRoute::SubmenuBranch {
            menu_index: 0,
            item_index: 0,
        })
    );
    pointer_bridge.sync(layout, hovered_branch.state);

    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(submenu_leaf_x, submenu_leaf_y),
    )
    .expect("nested editor operation leaf should dispatch through shared pointer route");

    assert_eq!(
        dispatched.pointer.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 1,
            action_id: "Weather.CloudLayer.Refresh".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("nested operation leaf should dispatch into runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness
            .runtime
            .operation_stack()
            .undo_stack()
            .last()
            .map(|entry| entry.operation_id.as_str()),
        Some("Weather.CloudLayer.Refresh")
    );
}
