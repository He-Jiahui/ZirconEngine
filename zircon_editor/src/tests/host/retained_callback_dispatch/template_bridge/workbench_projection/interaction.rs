use super::*;

#[test]
fn componentized_workbench_window_template_bridge_updates_tool_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge.recompute_layout(UiSize::new(1440.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolSelect", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchToolMove", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#171c20")
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchToolMove", UiEventKind::Click)
        .unwrap()
        .expect("move tool should have a binding");

    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::ViewportCommand(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Move)
        ))
    ));
    assert!(!control_bool(&bridge, "WorkbenchToolSelect", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolMove", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolMove", "checked"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolMove").as_deref(),
        Some("#12383d")
    );
    assert_eq!(
        bridge.frames().viewport,
        bridge
            .control_frame(EditorWorkbenchTemplateControlIds::VIEWPORT)
            .expect("resized viewport control frame")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchToolMove")
            .expect("move tool projection")
            .frame,
        bridge
            .control_frame("WorkbenchToolMove")
            .expect("move tool frame")
    );
}

#[test]
fn componentized_workbench_control_dispatch_updates_state_and_emits_typed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_tool_dispatch");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let effects = dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "WorkbenchToolRotate",
        UiEventKind::Click,
    )
    .expect("rotate tool should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolRotate", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolRotate", "checked"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchToolRotate").as_deref(),
        Some("#12383d")
    );
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::ActivateSceneMode {
            mode: SceneModeActivation::Transform(TransformHandleKind::Rotate),
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn componentized_workbench_pointer_dispatch_hits_tool_control_and_emits_typed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_tool");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let point = control_center(&bridge, "WorkbenchToolScale");

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    )
    .expect("tool pointer press should request paint-only feedback")
    .unwrap();
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    )
    .expect("tool pointer release should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchToolScale", "selected"));
    assert!(control_bool(&bridge, "WorkbenchToolScale", "checked"));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::ActivateSceneMode {
            mode: SceneModeActivation::Transform(TransformHandleKind::Scale),
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn componentized_workbench_pointer_dispatch_ignores_decorative_viewport_scene_layer() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_viewport_scene");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleScene", UiEventKind::Click)
        .unwrap()
        .expect("scene module should expose a preview binding");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchSceneWorkspace"),
        Some(UiVisibility::Visible)
    );
    let point = control_center(&bridge, "WorkbenchViewportFloorGrateRight");

    let press = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    );
    assert!(
        press.is_none(),
        "decorative viewport scene layer press should not request feedback or dispatch"
    );
    assert!(bridge.pointer_pressed_target().is_none());

    let release = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    );
    assert!(
        release.is_none(),
        "decorative viewport scene layer release should not dispatch"
    );
    assert!(harness.runtime.journal().records().is_empty());
}

#[test]
fn componentized_workbench_control_dispatch_records_component_drawer_preview_action() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_preview_dispatch");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let effects = dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "WorkbenchCheckboxOff",
        UiEventKind::Toggle,
    )
    .expect("component lab preview binding should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: "ComponentLab/CheckboxOffToggle".to_string(),
            pressed: false,
        })
    );
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_group
            .as_deref(),
        Some("ComponentLabPreview")
    );
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn componentized_workbench_pointer_dispatch_hits_component_drawer_toggle_once() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_toggle");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let point = control_center(&bridge, "WorkbenchCheckboxOff");

    assert!(!control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    )
    .expect("checkbox pointer press should request paint-only feedback")
    .unwrap();
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    )
    .expect("checkbox pointer release should dispatch")
    .unwrap();

    assert!(control_bool(&bridge, "WorkbenchCheckboxOff", "checked"));
    assert_eq!(
        harness
            .runtime
            .journal()
            .records()
            .last()
            .unwrap()
            .operation_group
            .as_deref(),
        Some("ComponentLabPreview")
    );
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn componentized_workbench_window_template_bridge_updates_activity_rail_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchRailScene", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRailImage", "selected"));

    let binding = bridge
        .dispatch_control_state("WorkbenchRailImage", UiEventKind::Click)
        .unwrap()
        .expect("asset rail button should have a binding");

    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::DockCommand(_)
    ));
    assert!(!control_bool(&bridge, "WorkbenchRailScene", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRailImage", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRailImage", "checked"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchRailImage").as_deref(),
        Some("#12383d")
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_scene_tree_selection_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchScenePropsItem", "selected"));

    let binding = bridge
        .dispatch_control_state("WorkbenchScenePlayerItem", UiEventKind::Click)
        .unwrap()
        .expect("scene row should have a binding");

    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::SelectionCommand(_)
    ));
    assert!(!control_bool(&bridge, "WorkbenchSceneRootItem", "selected"));
    assert!(!control_bool(
        &bridge,
        "WorkbenchScenePropsItem",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchScenePlayerItem",
        "selected"
    ));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchScenePlayerItem").as_deref(),
        Some("#12383d")
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_panel_tab_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(&bridge, "WorkbenchSceneTabScene", "selected"));
    assert!(!control_bool(
        &bridge,
        "WorkbenchSceneTabLayers",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInspectorTabInspector",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchInspectorTabHistory",
        "selected"
    ));

    let scene_binding = bridge
        .dispatch_control_state("WorkbenchSceneTabLayers", UiEventKind::Click)
        .unwrap()
        .expect("scene layers tab should expose a preview binding");
    assert!(matches!(
        scene_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "scene_tree.layers_tab.select"
    ));
    assert!(!control_bool(&bridge, "WorkbenchSceneTabScene", "selected"));
    assert!(control_bool(&bridge, "WorkbenchSceneTabLayers", "selected"));
    assert!(control_bool(&bridge, "WorkbenchSceneTabLayers", "checked"));
    assert_eq!(
        render_border_for_control(&bridge, "WorkbenchSceneTabLayers").as_deref(),
        Some("#35c7d0")
    );

    let inspector_binding = bridge
        .dispatch_control_state("WorkbenchInspectorTabHistory", UiEventKind::Click)
        .unwrap()
        .expect("inspector history tab should expose a preview binding");
    assert!(matches!(
        inspector_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "inspector.history_tab.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchInspectorTabInspector",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInspectorTabHistory",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInspectorTabHistory",
        "checked"
    ));
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_tab_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(control_bool(
        &bridge,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchDrawerTabConsole", UiEventKind::Click)
        .unwrap()
        .expect("component drawer console tab should expose a preview binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_drawer.console_tab.select"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Visible)
    );
}

#[test]
fn componentized_workbench_pointer_dispatch_hits_component_drawer_tab_visibility() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_componentized_workbench_pointer_drawer_tab");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let point = control_center(&bridge, "WorkbenchDrawerTabConsole");

    let press_effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(UiPointerButton::Primary),
    )
    .expect("drawer tab pointer press should request paint-only feedback")
    .unwrap();
    assert!(press_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!press_effects.render_dirty);
    assert!(!press_effects.presentation_dirty);

    let effects = dispatch_componentized_workbench_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(UiPointerButton::Primary),
    )
    .expect("drawer tab pointer release should dispatch")
    .unwrap();

    assert!(!control_bool(
        &bridge,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Visible)
    );
    assert!(!effects.render_dirty);
    assert!(!effects.presentation_dirty);
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_input_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert!(!control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchInputSegmented", "value").as_deref(),
        Some("center")
    );

    let dropdown_binding = bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .unwrap()
        .expect("dropdown should expose a preview binding");
    assert!(matches!(
        dropdown_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.input_dropdown.open"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchInputDropdown",
        "popup_open"
    ));
    assert!(control_bool(&bridge, "WorkbenchInputDropdown", "focused"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputDropdown").as_deref(),
        Some("#151b1f")
    );

    let segment_binding = bridge
        .dispatch_control_state("WorkbenchInputSegmented", UiEventKind::Change)
        .unwrap()
        .expect("segmented control should expose a preview binding");
    assert!(matches!(
        segment_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.input_segment.select"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchInputSegmented", "value").as_deref(),
        Some("right")
    );
    assert!(control_bool(&bridge, "WorkbenchInputSegmented", "selected"));
    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchInputSegmented").as_deref(),
        Some("#12383d")
    );
}

#[test]
fn componentized_workbench_window_template_bridge_updates_component_drawer_button_icon_state() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    assert_eq!(
        render_background_for_control(&bridge, "WorkbenchSecondaryButton").as_deref(),
        Some("#1d2328")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchDropdownButton",
        "popup_open"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchIconToggleSegmented", "value").as_deref(),
        Some("grid")
    );

    let dropdown_binding = bridge
        .dispatch_control_state("WorkbenchDropdownButton", UiEventKind::Click)
        .unwrap()
        .expect("button dropdown should expose a preview binding");
    assert!(matches!(
        dropdown_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.button_dropdown.open"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchDropdownButton",
        "popup_open"
    ));
    assert!(control_bool(&bridge, "WorkbenchDropdownButton", "focused"));

    let icon_toggle_binding = bridge
        .dispatch_control_state("WorkbenchIconToggleSegmented", UiEventKind::Change)
        .unwrap()
        .expect("icon toggle should expose a preview binding");
    assert!(matches!(
        icon_toggle_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "component_lab.icon_toggle_segment.select"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchIconToggleSegmented", "value").as_deref(),
        Some("list")
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchIconToggleSegmented",
        "selected"
    ));
}
