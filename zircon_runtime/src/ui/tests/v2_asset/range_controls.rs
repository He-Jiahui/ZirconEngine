use super::*;

#[test]
fn ui_v2_surface_default_rangefield_click_sets_value_and_rebuilds_render_only() {
    let mut document = v2_document("asset://ui/tests/runtime_rangefield_click.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeField".to_string(),
            control_id: Some("RuntimeRange".to_string()),
            classes: vec!["material-range".to_string()],
            props: BTreeMap::from([
                ("value".to_string(), Value::Float(0.0)),
                ("min".to_string(), Value::Float(0.0)),
                ("max".to_string(), Value::Float(100.0)),
                ("step".to_string(), Value::Float(5.0)),
            ]),
            layout: Some(fixed_size_layout(100.0, 24.0)),
            events: vec![UiBindingRef {
                component_event: Some(UiComponentEventKind::ValueChanged),
                id: "RuntimeRange/ValueChanged".to_string(),
                event: UiEventKind::Change,
                mode: Default::default(),
                route: Some("RuntimeRange.ValueChanged".to_string()),
                action: None,
                targets: Vec::new(),
            }],
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_rangefield_click"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeRange");
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let frame = surface.arranged_tree.get(node_id).unwrap().frame;
    let point = UiPoint::new(frame.x + frame.width * 0.73, frame.y + frame.height * 0.5);
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

    let value = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("value")
        .and_then(Value::as_float)
        .unwrap();
    assert!((value - 75.0).abs() < f64::EPSILON);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    assert!(up.component_events.iter().any(|event| {
        event.node_id == node_id
            && event.binding_id == "RuntimeRange/ValueChanged"
            && event.event_kind == UiEventKind::Change
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value" && value == &UiValue::Float(75.0)
            )
    }));

    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.arranged_rebuilt);
    assert!(!rebuild.hit_grid_rebuilt);
}

#[test]
fn ui_v2_surface_rangefield_drag_captures_pointer_and_updates_value_outside_hit() {
    let mut document = v2_document("asset://ui/tests/runtime_rangefield_drag.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeField".to_string(),
            control_id: Some("RuntimeRange".to_string()),
            classes: vec!["material-range".to_string()],
            props: BTreeMap::from([
                ("value".to_string(), Value::Float(50.0)),
                ("min".to_string(), Value::Float(0.0)),
                ("max".to_string(), Value::Float(100.0)),
                ("step".to_string(), Value::Float(5.0)),
            ]),
            layout: Some(fixed_size_layout(100.0, 24.0)),
            events: vec![
                UiBindingRef {
                    component_event: Some(UiComponentEventKind::ValueChanged),
                    id: "RuntimeRange/ValueChanged".to_string(),
                    event: UiEventKind::Change,
                    mode: Default::default(),
                    route: Some("RuntimeRange.ValueChanged".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    component_event: Some(UiComponentEventKind::BeginDrag),
                    id: "RuntimeRange/DragBegin".to_string(),
                    event: UiEventKind::DragBegin,
                    mode: Default::default(),
                    route: Some("RuntimeRange.BeginDrag".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    component_event: Some(UiComponentEventKind::DragDelta),
                    id: "RuntimeRange/DragUpdate".to_string(),
                    event: UiEventKind::DragUpdate,
                    mode: Default::default(),
                    route: Some("RuntimeRange.DragUpdate".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
                UiBindingRef {
                    component_event: Some(UiComponentEventKind::EndDrag),
                    id: "RuntimeRange/DragEnd".to_string(),
                    event: UiEventKind::DragEnd,
                    mode: Default::default(),
                    route: Some("RuntimeRange.EndDrag".to_string()),
                    action: None,
                    targets: Vec::new(),
                },
            ],
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_rangefield_drag"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeRange");
    let frame = surface.arranged_tree.get(node_id).unwrap().frame;
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let down_point = UiPoint::new(frame.x + frame.width * 0.2, frame.y + frame.height * 0.5);
    let down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, down_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(down.captured_by, Some(node_id));
    assert_eq!(surface.focus.captured, Some(node_id));
    assert!(down.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/DragBegin"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "value"
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let outside_right = UiPoint::new(frame.x + frame.width * 1.25, frame.y + frame.height * 0.5);
    let drag = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, outside_right),
        )
        .unwrap();

    assert_eq!(drag.handled_by, Some(node_id));
    assert_eq!(surface.focus.captured, Some(node_id));
    assert_range_value(&surface, node_id, 100.0);
    assert!(drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/ValueChanged"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value" && value == &UiValue::Float(100.0)
            )
    }));
    assert!(drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/DragUpdate"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::DragDelta { property, delta }
                    if property == "value" && (*delta - 50.0).abs() < f64::EPSILON
            )
    }));
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    let drag_rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(drag_rebuild.render_rebuilt);
    assert!(!drag_rebuild.layout_recomputed);
    assert!(!drag_rebuild.hit_grid_rebuilt);

    let outside_left = UiPoint::new(frame.x - frame.width * 0.25, frame.y + frame.height * 0.5);
    let up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, outside_left)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(up.released_capture, Some(node_id));
    assert_eq!(surface.focus.captured, None);
    assert_range_value(&surface, node_id, 0.0);
    assert!(up.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRange/DragEnd"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::EndDrag { property } if property == "value"
            )
    }));
}

#[test]
fn ui_v2_surface_rangeslider_click_targets_nearest_thumb() {
    let (mut surface, root_size) = runtime_range_slider_surface(
        "asset://ui/tests/runtime_rangeslider_click.v2.ui",
        "runtime.ui.v2.runtime_rangeslider_click",
        None,
    );

    let node_id = node_id_by_control_id(&surface, "RuntimeRangeSlider");
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let frame = surface.arranged_tree.get(node_id).unwrap().frame;
    let lower_point = UiPoint::new(frame.x + frame.width * 0.3, frame.y + frame.height * 0.5);
    let upper_point = UiPoint::new(frame.x + frame.width * 0.9, frame.y + frame.height * 0.5);
    let lower_down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, lower_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(lower_down.captured_by, Some(node_id));
    assert!(lower_down.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragBegin"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "range_min"
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let lower_drag = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, upper_point),
        )
        .unwrap();

    assert_eq!(lower_drag.handled_by, Some(node_id));
    assert_eq!(surface.focus.captured, Some(node_id));
    assert_range_property_value(&surface, node_id, "range_min", 80.0);
    assert_range_value(&surface, node_id, 80.0);
    assert!(lower_drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragDelta"
            && event.reason == UiPointerComponentEventReason::DirectBinding
            && matches!(
                &event.envelope.event,
                UiComponentEvent::DragDelta { property, .. } if property == "range_min"
            )
    }));
    assert!(lower_drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/ValueChanged"
            && event.reason == UiPointerComponentEventReason::DirectBinding
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "range_min" && value == &UiValue::Float(80.0)
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let lower_up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, upper_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(lower_up.released_capture, Some(node_id));
    assert_range_property_value(&surface, node_id, "range_min", 80.0);
    assert_range_value(&surface, node_id, 80.0);
    assert!(lower_up.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragEnd"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::EndDrag { property } if property == "range_min"
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let upper_down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, upper_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(upper_down.captured_by, Some(node_id));
    assert!(upper_down.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragBegin"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "value"
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let upper_up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, upper_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(upper_up.released_capture, Some(node_id));
    assert_range_property_value(&surface, node_id, "range_min", 80.0);
    assert_range_value(&surface, node_id, 90.0);
    assert!(upper_up.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/ValueChanged"
            && event.reason == UiPointerComponentEventReason::DefaultClick
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value" && value == &UiValue::Float(90.0)
            )
    }));
}

#[test]
fn ui_v2_surface_rangeslider_swap_policy_can_switch_active_thumb() {
    let (mut surface, root_size) = runtime_range_slider_surface(
        "asset://ui/tests/runtime_rangeslider_swap.v2.ui",
        "runtime.ui.v2.runtime_rangeslider_swap",
        Some(false),
    );

    let node_id = node_id_by_control_id(&surface, "RuntimeRangeSlider");
    let dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    let frame = surface.arranged_tree.get(node_id).unwrap().frame;
    let lower_point = UiPoint::new(frame.x + frame.width * 0.3, frame.y + frame.height * 0.5);
    let upper_point = UiPoint::new(frame.x + frame.width * 0.9, frame.y + frame.height * 0.5);
    let lower_down = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, lower_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(lower_down.captured_by, Some(node_id));
    assert!(lower_down.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragBegin"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "range_min"
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let lower_drag = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, upper_point),
        )
        .unwrap();

    assert_eq!(lower_drag.handled_by, Some(node_id));
    assert_range_property_value(&surface, node_id, "range_min", 80.0);
    assert_range_value(&surface, node_id, 90.0);
    assert!(lower_drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/ValueChanged"
            && event.reason == UiPointerComponentEventReason::DirectBinding
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "range_min" && value == &UiValue::Float(80.0)
            )
    }));
    assert!(lower_drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/ValueChanged"
            && event.reason == UiPointerComponentEventReason::DirectBinding
            && matches!(
                &event.envelope.event,
                UiComponentEvent::ValueChanged { property, value }
                    if property == "value" && value == &UiValue::Float(90.0)
            )
    }));
    assert!(lower_drag.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragDelta"
            && event.reason == UiPointerComponentEventReason::DirectBinding
            && matches!(
                &event.envelope.event,
                UiComponentEvent::DragDelta { property, delta }
                    if property == "value" && (*delta - 10.0).abs() < f64::EPSILON
            )
    }));
    surface.rebuild_dirty(root_size).unwrap();

    let lower_up = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Up, upper_point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(lower_up.released_capture, Some(node_id));
    assert_range_property_value(&surface, node_id, "range_min", 80.0);
    assert_range_value(&surface, node_id, 90.0);
    assert!(lower_up.component_events.iter().any(|event| {
        event.binding_id == "RuntimeRangeSlider/DragEnd"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::EndDrag { property } if property == "value"
            )
    }));
}

#[test]
fn ui_v2_surface_rangefield_keyboard_navigation_steps_value_render_only() {
    let mut document = v2_document("asset://ui/tests/runtime_rangefield_keyboard.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "RangeField".to_string(),
            control_id: Some("RuntimeRange".to_string()),
            props: BTreeMap::from([
                ("value".to_string(), Value::Float(50.0)),
                ("min".to_string(), Value::Float(0.0)),
                ("max".to_string(), Value::Float(100.0)),
                ("step".to_string(), Value::Float(5.0)),
            ]),
            layout: Some(fixed_size_layout(100.0, 24.0)),
            ..Default::default()
        },
    );

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_rangefield_keyboard"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_size = UiSize::new(160.0, 80.0);
    surface.compute_layout(root_size).unwrap();
    surface.clear_dirty_flags();

    let node_id = node_id_by_control_id(&surface, "RuntimeRange");
    surface.focus_node(node_id).unwrap();
    surface.clear_dirty_flags();

    let right = surface
        .dispatch_navigation_event(
            &crate::ui::dispatch::UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();
    assert_eq!(right.handled_by, Some(node_id));
    assert_eq!(right.focus_changed_to, None);
    assert_range_value(&surface, node_id, 55.0);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
    surface.rebuild_dirty(root_size).unwrap();

    let left = surface
        .dispatch_navigation_event(
            &crate::ui::dispatch::UiNavigationDispatcher::default(),
            UiNavigationEventKind::Left,
        )
        .unwrap();
    assert_eq!(left.handled_by, Some(node_id));
    assert_range_value(&surface, node_id, 50.0);
    let rebuild = surface.rebuild_dirty(root_size).unwrap();
    assert!(rebuild.render_rebuilt);
    assert!(!rebuild.layout_recomputed);
    assert!(!rebuild.hit_grid_rebuilt);
}
