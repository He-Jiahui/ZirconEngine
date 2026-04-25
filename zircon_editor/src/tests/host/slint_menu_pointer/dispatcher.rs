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
    for index in 0..10 {
        dispatch_menu_action(&harness.runtime, &format!("SavePreset.alpha-{index:02}"))
            .expect("preset save setup should succeed");
    }
    let template_bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = HostMenuPointerBridge::new();
    let layout = window_menu_layout(10);
    pointer_bridge.sync(
        layout.clone(),
        HostMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(240.0, 110.0), 96.0)
        .expect("window popup should accept shared scroll input");
    assert!(scrolled.state.popup_scroll_offset > 0.0);

    pointer_bridge.sync(layout, scrolled.state.clone());
    let preset_index = 5usize;
    let click_y = 32.0 + preset_index as f32 * 30.0 - scrolled.state.popup_scroll_offset + 14.0;
    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(240.0, click_y),
    )
    .expect("shared pointer route should dispatch a scrolled preset selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(HostMenuPointerRoute::MenuItem {
            menu_index: 4,
            item_index: preset_index,
            action_id: "LoadPreset.alpha-03".to_string(),
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
            name: "alpha-03".to_string(),
        })
    );
}
