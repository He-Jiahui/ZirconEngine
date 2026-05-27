use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiDragPhase},
    dispatch::{UiPointerDispatchResult, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn range_home_and_end_navigation_use_authored_min_max_aliases() {
    let mut surface = range_surface();
    let node_id = UiNodeId::new(2);
    surface.focus_node(node_id).unwrap();
    surface.clear_dirty_flags();

    let end = surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::End,
        )
        .unwrap();
    assert_eq!(end.handled_by, Some(node_id));
    assert_eq!(end.focus_changed_to, None);
    assert_widget_binding_report(&end.binding_reports);
    assert_range_value(&surface, 100.0);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);

    surface.clear_dirty_flags();
    let home = surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Home,
        )
        .unwrap();
    assert_eq!(home.handled_by, Some(node_id));
    assert_eq!(home.focus_changed_to, None);
    assert_widget_binding_report(&home.binding_reports);
    assert_range_value(&surface, 0.0);
    assert!(surface.dirty_flags().render);
    assert!(!surface.dirty_flags().layout);
}

#[test]
fn range_pointer_drag_reports_phase_and_distance_metrics() {
    let mut surface = range_surface();

    let down = press_primary(&mut surface, 68.0, 16.0);
    assert_eq!(down.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(down.captured_by, Some(UiNodeId::new(2)));
    assert!(down.component_events.iter().any(|event| {
        event.binding_id == "Range/DragBegin"
            && event.drag.as_ref().is_some_and(|drag| {
                drag.phase == UiDragPhase::Begin
                    && drag.start == UiPoint::new(68.0, 16.0)
                    && drag.current == UiPoint::new(68.0, 16.0)
                    && drag.distance == 0.0
            })
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "amount"
            )
    }));

    let drag = move_pointer(&mut surface, 128.0, 16.0);
    assert_eq!(drag.handled_by, Some(UiNodeId::new(2)));
    assert_widget_binding_report(&drag.binding_reports);
    assert!(drag.component_events.iter().any(|event| {
        event.binding_id == "Range/DragUpdate"
            && event.drag.as_ref().is_some_and(|drag| {
                drag.phase == UiDragPhase::Update
                    && drag.start == UiPoint::new(68.0, 16.0)
                    && drag.current == UiPoint::new(128.0, 16.0)
                    && drag.delta == UiPoint::new(60.0, 0.0)
                    && (drag.distance - 60.0).abs() < f32::EPSILON
            })
            && matches!(
                &event.envelope.event,
                UiComponentEvent::DragDelta { property, delta }
                    if property == "amount" && (*delta - 50.0).abs() < f64::EPSILON
            )
    }));
    assert_range_value(&surface, 100.0);

    let up = release_primary(&mut surface, 128.0, 16.0);
    assert_eq!(up.handled_by, Some(UiNodeId::new(2)));
    assert_eq!(up.released_capture, Some(UiNodeId::new(2)));
    assert!(up.component_events.iter().any(|event| {
        event.binding_id == "Range/DragEnd"
            && event.drag.as_ref().is_some_and(|drag| {
                drag.phase == UiDragPhase::End
                    && drag.start == UiPoint::new(68.0, 16.0)
                    && drag.current == UiPoint::new(128.0, 16.0)
                    && drag.delta == UiPoint::new(60.0, 0.0)
                    && (drag.distance - 60.0).abs() < f32::EPSILON
            })
            && matches!(
                &event.envelope.event,
                UiComponentEvent::EndDrag { property } if property == "amount"
            )
    }));
}

fn assert_widget_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].updates.first().map(|update| update.source.kind),
        Some(UiBindingSourceKind::WidgetBehavior)
    );
}

fn range_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.range.navigation"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/range"))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "RuntimeMeter".to_string(),
                    bindings: vec![
                        binding("Range/DragBegin", UiEventKind::DragBegin),
                        binding("Range/DragUpdate", UiEventKind::DragUpdate),
                        binding("Range/DragEnd", UiEventKind::DragEnd),
                    ],
                    attributes: toml::from_str(
                        "amount = 50.0\nlow = 0.0\nhigh = 100.0\nquantum = 5.0",
                    )
                    .unwrap(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Range,
                        value_property: Some("amount".to_string()),
                        min_property: Some("low".to_string()),
                        max_property: Some("high".to_string()),
                        step_property: Some("quantum".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn press_primary(surface: &mut UiSurface, x: f32, y: f32) -> UiPointerDispatchResult {
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(x, y))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap()
}

fn move_pointer(surface: &mut UiSurface, x: f32, y: f32) -> UiPointerDispatchResult {
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(x, y)),
        )
        .unwrap()
}

fn release_primary(surface: &mut UiSurface, x: f32, y: f32) -> UiPointerDispatchResult {
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(x, y))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap()
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn assert_range_value(surface: &UiSurface, expected: f64) {
    let value = surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes["amount"]
        .as_float()
        .unwrap();
    assert!(
        (value - expected).abs() < f64::EPSILON,
        "expected range value {expected}, got {value}"
    );
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        ..UiStateFlags::default()
    }
}
