use crate::ui::surface::UiSurface;
use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::dispatch::{UiPointerComponentEventReason, UiPointerEvent};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentEventKind, UiValue},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiHitTestQuery, UiPointerButton, UiPointerEventKind, UiVirtualPointerPosition},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn primary_release_inside_pressed_target_marks_click_target_and_clears_press_state() {
    let mut surface = button_surface();

    let down = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(down.target, Some(UiNodeId::new(2)));
    assert_eq!(down.pressed, Some(UiNodeId::new(2)));
    assert_eq!(down.click_target, None);
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));

    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.pressed, Some(UiNodeId::new(2)));
    assert_eq!(up.click_target, Some(UiNodeId::new(2)));
    assert!(up.release_inside_pressed);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn primary_release_outside_pressed_target_does_not_mark_click_target() {
    let mut surface = button_surface();

    surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();
    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(140.0, 80.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.pressed, Some(UiNodeId::new(2)));
    assert_eq!(up.click_target, None);
    assert!(!up.release_inside_pressed);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn captured_release_uses_hit_path_not_capture_target_for_click_target() {
    let mut surface = button_surface();
    surface.focus.pressed = Some(UiNodeId::new(2));
    surface.focus.captured = Some(UiNodeId::new(2));

    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(140.0, 80.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.target, Some(UiNodeId::new(2)));
    assert_eq!(up.hit_path.target, None);
    assert_eq!(up.click_target, None);
    assert!(!up.release_inside_pressed);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn pointer_route_can_use_virtual_pointer_hit_from_custom_surface_mapper() {
    let mut surface = button_surface();
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(20.0, 20.0), UiPoint::new(18.0, 18.0));
    let route = surface
        .route_pointer_event_with_query_and_button(
            UiPointerEventKind::Down,
            UiHitTestQuery::new(UiPoint::new(140.0, 80.0)).with_virtual_pointer(virtual_pointer),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(route.point, UiPoint::new(20.0, 20.0));
    assert_eq!(route.target, Some(UiNodeId::new(2)));
    assert_eq!(route.hit_path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
}

#[test]
fn pointer_dispatch_uses_virtual_pointer_query_for_component_events() {
    let mut surface =
        bound_button_surface(vec![binding("Showcase/ButtonPress", UiEventKind::Press)]);
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(20.0, 20.0), UiPoint::new(18.0, 18.0));

    let result = surface
        .dispatch_pointer_event_with_query(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(140.0, 80.0))
                .with_button(UiPointerButton::Primary),
            UiHitTestQuery::new(UiPoint::new(140.0, 80.0)).with_virtual_pointer(virtual_pointer),
        )
        .unwrap();

    assert_eq!(result.route.target, Some(UiNodeId::new(2)));
    assert_eq!(result.route.hit_path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].binding_id,
        "Showcase/ButtonPress"
    );
}

#[test]
fn pointer_dispatch_result_reports_same_target_hover_as_idle_diagnostic() {
    let mut surface = button_surface();

    let first = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert!(!first.diagnostics.ignored_same_target_hover);
    assert_eq!(first.diagnostics.hover_entered, 1);

    let second = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(25.0, 25.0)),
        )
        .unwrap();

    assert!(second.diagnostics.pointer_routed);
    assert!(second.diagnostics.ignored_same_target_hover);
    assert_eq!(second.diagnostics.hover_entered, 0);
    assert_eq!(second.diagnostics.hover_left, 0);
}

#[test]
fn click_component_events_preserve_every_matching_binding_on_target() {
    let mut surface = bound_button_surface(vec![
        binding("Showcase/ButtonPrimary", UiEventKind::Click),
        binding("Showcase/ButtonAudit", UiEventKind::Click),
    ]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let up = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(up.component_events.len(), 2);
    assert_eq!(up.component_events[0].binding_id, "Showcase/ButtonPrimary");
    assert_eq!(up.component_events[1].binding_id, "Showcase/ButtonAudit");
    for event in &up.component_events {
        assert_eq!(event.node_id, UiNodeId::new(2));
        assert_eq!(event.event_kind, UiEventKind::Click);
        assert_eq!(event.reason, UiPointerComponentEventReason::DefaultClick);
        assert_eq!(event.envelope.document_id, "runtime.ui.events");
        assert_eq!(event.envelope.control_id, "MaterialButton");
        assert_eq!(event.envelope.event_kind, UiComponentEventKind::Commit);
        assert_eq!(
            event.envelope.event,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            }
        );
    }
}

fn button_surface() -> UiSurface {
    button_surface_with_metadata(None)
}

fn bound_button_surface(bindings: Vec<UiBindingRef>) -> UiSurface {
    button_surface_with_metadata(Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings,
        ..Default::default()
    }))
}

fn button_surface_with_metadata(template_metadata: Option<UiTemplateNodeMetadata>) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    let mut button = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
        .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(template_metadata) = template_metadata {
        button = button.with_template_metadata(template_metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), button).unwrap();
    surface.rebuild();
    surface
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

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
