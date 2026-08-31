use super::support::*;

#[test]
fn workbench_toolbar_window_menus_open_exclusively_and_toggle_closed() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id } if action_id == "workbench.run_mode.menu.open"
    ));
    assert!(control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRunMode", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Visible)
    );
    assert!(
        bridge.control_frame("WorkbenchRunModeMenu").is_some(),
        "opened run mode menu should have a native frame"
    );

    bridge
        .dispatch_control_state("WorkbenchLayoutGrid", UiEventKind::Click)
        .expect("layout menu should dispatch")
        .expect("layout menu should expose a menu binding");
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert!(control_bool(&bridge, "WorkbenchLayoutGrid", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLayoutMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Visible)
    );

    bridge
        .dispatch_control_state("WorkbenchLayoutGrid", UiEventKind::Click)
        .expect("layout menu should dispatch when toggled closed")
        .expect("layout menu should expose a menu binding");
    assert!(!control_bool(&bridge, "WorkbenchLayoutGrid", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchLayoutMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn workbench_toolbar_window_menu_item_selection_closes_trigger_and_menu() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");

    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchRunModeMenu", "menu.item.simulate")
            .expect("run mode menu item should select"),
        Some(true)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value_text").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string_array(&bridge, "WorkbenchRunModeMenu", "menu_items"),
        vec![
            "Play In Editor|action=menu.item.play_in_editor,icon=play",
            "Simulate|action=menu.item.simulate,icon=play,checked",
            "Standalone|action=menu.item.standalone,disabled,icon=grid",
            "Network Preview|action=menu.item.network_preview,disabled,icon=route",
        ]
    );
    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchRunModeMenu", "menu.item.standalone")
            .expect("disabled run mode should resolve without mutation"),
        Some(false)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn mixed_command_menu_preserves_its_projected_default_indicator() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");

    let expected_items = vec![
        "Default Layout|checked,disabled,icon=grid",
        "Gameplay Layout|disabled,icon=grid",
        "Rendering Layout|disabled,icon=grid",
        "Reset Layout|danger,icon=trash",
    ];
    assert_eq!(
        control_string_array(&bridge, "WorkbenchLayoutMenu", "menu_items"),
        expected_items
    );

    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchLayoutMenu", "menu.item.reset_layout")
            .expect("Reset Layout should select as a command row"),
        Some(true)
    );
    assert_eq!(
        control_string_array(&bridge, "WorkbenchLayoutMenu", "menu_items"),
        expected_items
    );
}

#[test]
fn workbench_run_mode_selection_drives_the_next_play_session() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_run_mode_simulate");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchRunModeMenu",
        "menu.item.simulate",
    )
    .expect("Simulate menu item should be handled")
    .expect("Simulate menu item should dispatch");

    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::SelectPlayMode(PlayKind::Simulate))
    );
    assert_eq!(
        harness.runtime.play_sessions().preferred_kind(),
        PlayKind::Simulate
    );
    assert!(effects.presentation_dirty);

    let mut rebuilt_bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
            .expect("rebuilt workbench template should project");
    dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut rebuilt_bridge,
        "WorkbenchRunMode",
        UiEventKind::Click,
    )
    .expect("rebuilt Run Mode should be handled")
    .expect("rebuilt Run Mode should open");
    assert_eq!(
        control_string(&rebuilt_bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string_array(&rebuilt_bridge, "WorkbenchRunModeMenu", "menu_items"),
        vec![
            "Play In Editor|action=menu.item.play_in_editor,icon=play",
            "Simulate|action=menu.item.simulate,icon=play,checked",
            "Standalone|action=menu.item.standalone,disabled,icon=grid",
            "Network Preview|action=menu.item.network_preview,disabled,icon=route",
        ]
    );

    dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "WorkbenchRunPlay",
        UiEventKind::Click,
    )
    .expect("Play should be handled")
    .expect("Play should enter the selected run mode");

    assert_eq!(
        harness.runtime.play_sessions().mode_snapshot(),
        PlayMode::Playing {
            kind: PlayKind::Simulate,
        }
    );
}
