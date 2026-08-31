use super::*;

#[test]
fn ui_v2_surface_default_toggle_click_mutates_checked_and_restyles_runtime_pseudo_state() {
    let mut document = v2_document("asset://ui/tests/runtime_toggle_click.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Toggle".to_string(),
            control_id: Some("RuntimeToggle".to_string()),
            classes: vec!["material-toggle".to_string()],
            layout: Some(fixed_size_layout(120.0, 32.0)),
            events: vec![UiBindingRef {
                component_event: Some(UiComponentEventKind::ValueChanged),
                id: "RuntimeToggle/Changed".to_string(),
                event: UiEventKind::Change,
                mode: Default::default(),
                route: Some("RuntimeToggle.Change".to_string()),
                action: None,
                targets: Vec::new(),
            }],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_toggle_material".to_string(),
        rules: vec![
            style_rule("Toggle.material-toggle", [("background", "#101010")]),
            style_rule(
                "Toggle.material-toggle:checked",
                [("background", "#225533")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_toggle_click"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(200.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeToggle");
    assert!(
        !surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    assert!(up.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.event_kind == UiEventKind::Change
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "checked" && value == &UiValue::Bool(true)
            )
    }));

    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
    assert_eq!(
        render_command_background(&surface, node_id).as_deref(),
        Some("#225533")
    );
}

#[test]
fn ui_v2_surface_authored_checked_state_can_toggle_off_without_stale_prop_style() {
    let mut document = v2_document("asset://ui/tests/runtime_toggle_checked_seed.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Toggle".to_string(),
            control_id: Some("RuntimeToggle".to_string()),
            classes: vec!["material-toggle".to_string()],
            props: BTreeMap::from([("checked".to_string(), Value::Boolean(true))]),
            layout: Some(fixed_size_layout(120.0, 32.0)),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_toggle_checked_seed_material".to_string(),
        rules: vec![
            style_rule("Toggle.material-toggle", [("background", "#101010")]),
            style_rule(
                "Toggle.material-toggle:checked",
                [("background", "#225533")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_toggle_checked_seed"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(200.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeToggle");
    assert!(
        surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(
        !surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .state_flags
            .checked
    );
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
}

#[test]
fn ui_v2_surface_default_foldout_click_toggles_expanded_and_restyles_runtime_pseudo_state() {
    let mut document = v2_document("asset://ui/tests/runtime_foldout_toggle.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Foldout".to_string(),
            control_id: Some("RuntimeFoldout".to_string()),
            classes: vec!["material-foldout".to_string()],
            props: BTreeMap::from([("expanded".to_string(), Value::Boolean(true))]),
            layout: Some(fixed_size_layout(160.0, 32.0)),
            events: vec![UiBindingRef {
                component_event: Some(UiComponentEventKind::ToggleExpanded),
                id: "RuntimeFoldout/Toggled".to_string(),
                event: UiEventKind::Toggle,
                mode: Default::default(),
                route: Some("RuntimeFoldout.Toggle".to_string()),
                action: None,
                targets: Vec::new(),
            }],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_foldout_material".to_string(),
        rules: vec![
            style_rule("Foldout.material-foldout", [("background", "#101010")]),
            style_rule(
                "Foldout.material-foldout:expanded",
                [("background", "#225533")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_foldout_toggle"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(220.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeFoldout");
    assert!(surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.expanded));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(!surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.expanded));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    assert!(up.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.event_kind == UiEventKind::Toggle
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ToggleExpanded { expanded } if !expanded
            )
    }));

    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
}

#[test]
fn ui_v2_surface_default_combobox_click_toggles_popup_open_and_routes_typed_events() {
    let mut document = v2_document("asset://ui/tests/runtime_combobox_popup.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "ComboBox".to_string(),
            control_id: Some("RuntimeComboBox".to_string()),
            classes: vec!["material-combo".to_string()],
            props: BTreeMap::from([
                ("value".to_string(), Value::String("scene".to_string())),
                ("popup_open".to_string(), Value::Boolean(false)),
            ]),
            layout: Some(fixed_size_layout(180.0, 32.0)),
            events: vec![
                UiBindingRef {
                    component_event: Some(UiComponentEventKind::OpenPopup),
                    id: "RuntimeComboBox/OpenPopup".to_string(),
                    event: UiEventKind::Click,
                    mode: Default::default(),
                    route: Some("RuntimeComboBox.OpenPopup".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    component_event: Some(UiComponentEventKind::ClosePopup),
                    id: "RuntimeComboBox/ClosePopup".to_string(),
                    event: UiEventKind::Click,
                    mode: Default::default(),
                    route: Some("RuntimeComboBox.ClosePopup".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
            ],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_combobox_popup_material".to_string(),
        rules: vec![
            style_rule("ComboBox.material-combo", [("background", "#101010")]),
            style_rule("ComboBox.material-combo:open", [("background", "#225533")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_combobox_popup"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(240.0, 100.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeComboBox");
    assert!(!surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.popup_open));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let point = UiPoint::new(12.0, 12.0);
    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let open = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.popup_open));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );
    assert!(open.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.binding_id == "RuntimeComboBox/OpenPopup"
            && event.event_kind == UiEventKind::Click
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(&event.envelope.event, UiComponentEvent::OpenPopup)
    }));
    assert!(!open
        .component_events
        .iter()
        .any(|event| event.binding_id == "RuntimeComboBox/ClosePopup"));

    let rebuild_open = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild_open.render_rebuilt);
    assert!(!rebuild_open.layout_recomputed);
    assert!(!rebuild_open.arranged_rebuilt);
    assert!(!rebuild_open.hit_grid_rebuilt);
    assert_eq!(
        render_command_background(&surface, node_id).as_deref(),
        Some("#225533")
    );

    surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    surface.rebuild_dirty(root_size).unwrap();

    let close = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert!(!surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.popup_open));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert!(close.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.binding_id == "RuntimeComboBox/ClosePopup"
            && event.event_kind == UiEventKind::Click
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(&event.envelope.event, UiComponentEvent::ClosePopup)
    }));
    assert!(!close
        .component_events
        .iter()
        .any(|event| event.binding_id == "RuntimeComboBox/OpenPopup"));
}
