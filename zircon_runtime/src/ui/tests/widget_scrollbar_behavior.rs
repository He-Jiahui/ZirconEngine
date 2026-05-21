use crate::ui::{dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityContract,
    },
    binding::{UiBindingDirtyDomain, UiBindingSourceKind, UiBindingTargetKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiDispatchDisposition, UiInputEvent, UiInputEventMetadata,
        UiPointerDispatchResult, UiPointerEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiAxis, UiContainerKind, UiFrame, UiPoint, UiScrollState, UiScrollableBoxConfig},
    surface::{UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: false,
        ..UiStateFlags::default()
    }
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: true,
        ..UiStateFlags::default()
    }
}

fn scrollbar_metadata(scroll_target: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Scrollbar".to_string(),
        widget: UiWidgetContract {
            behavior: UiWidgetBehavior::Scrollbar,
            scroll_target: Some(scroll_target.to_string()),
            scroll_axis: Some(UiAxis::Vertical),
            min_thumb_extent: Some(16.0),
            ..UiWidgetContract::default()
        },
        ..UiTemplateNodeMetadata::default()
    }
}

fn thumb_metadata() -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "ScrollbarThumb".to_string(),
        bindings: vec![
            binding("ScrollbarThumb/DragBegin", UiEventKind::DragBegin),
            binding("ScrollbarThumb/DragUpdate", UiEventKind::DragUpdate),
            binding("ScrollbarThumb/DragEnd", UiEventKind::DragEnd),
        ],
        widget: UiWidgetContract {
            behavior: UiWidgetBehavior::ScrollbarThumb,
            ..UiWidgetContract::default()
        },
        ..UiTemplateNodeMetadata::default()
    }
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

fn scrollable_node() -> UiTreeNode {
    UiTreeNode::new(id(2), UiNodePath::new("root/scroll"))
        .with_frame(UiFrame::new(0.0, 0.0, 100.0, 100.0))
        .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: UiAxis::Vertical,
            ..UiScrollableBoxConfig::default()
        }))
        .with_scroll_state(UiScrollState {
            offset: 0.0,
            viewport_extent: 100.0,
            content_extent: 300.0,
        })
        .with_state_flags(focusable_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "ScrollableBox".to_string(),
            a11y: UiAccessibilityContract {
                name: Some("Results".to_string()),
                ..UiAccessibilityContract::default()
            },
            ..UiTemplateNodeMetadata::default()
        })
}

fn scrollbar_surface(include_thumb: bool) -> UiSurface {
    scrollbar_surface_with_thumb_frame(include_thumb, UiFrame::new(120.0, 60.0, 12.0, 20.0))
}

fn scrollbar_surface_with_thumb_frame(include_thumb: bool, thumb_frame: UiFrame) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.scrollbar"));
    surface.tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 120.0)),
    );
    surface.tree.insert_child(id(1), scrollable_node()).unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(3), UiNodePath::new("root/scrollbar"))
                .with_frame(UiFrame::new(120.0, 0.0, 12.0, 100.0))
                .with_state_flags(pointer_state())
                .with_template_metadata(scrollbar_metadata("#2")),
        )
        .unwrap();
    if include_thumb {
        surface
            .tree
            .insert_child(
                id(3),
                UiTreeNode::new(id(4), UiNodePath::new("root/scrollbar/thumb"))
                    .with_frame(thumb_frame)
                    .with_state_flags(pointer_state())
                    .with_template_metadata(thumb_metadata()),
            )
            .unwrap();
    }
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

fn click_primary(surface: &mut UiSurface, x: f32, y: f32) {
    press_primary(surface, x, y);
}

#[test]
fn scrollbar_track_click_pages_target_scroll_container() {
    let mut surface = scrollbar_surface(false);

    click_primary(&mut surface, 126.0, 80.0);
    let result = surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(
                UiPointerEventKind::Up,
                zircon_runtime_interface::ui::layout::UiPoint::new(126.0, 80.0),
            )
            .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(id(3)));
    assert!(result.diagnostics.scroll_defaulted);
    assert!(result.component_events.is_empty());
    assert_scrollbar_binding_report(&result.binding_reports);
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        100.0
    );
}

#[test]
fn scrollbar_thumb_click_does_not_page_scroll_container() {
    let mut surface = scrollbar_surface(true);

    click_primary(&mut surface, 126.0, 70.0);
    let result = release_primary(&mut surface, 126.0, 70.0);

    assert_ne!(result.handled_by, Some(id(3)));
    assert_eq!(result.handled_by, Some(id(4)));
    assert_eq!(result.released_capture, Some(id(4)));
    assert!(result.binding_reports.is_empty());
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );
}

#[test]
fn scrollbar_thumb_drag_updates_target_scroll_container_offset() {
    let mut surface =
        scrollbar_surface_with_thumb_frame(true, UiFrame::new(120.0, 0.0, 12.0, 20.0));

    let down = press_primary(&mut surface, 126.0, 10.0);
    assert_eq!(down.handled_by, Some(id(4)));
    assert_eq!(down.captured_by, Some(id(4)));
    assert_eq!(surface.focus.captured, Some(id(4)));
    assert!(down.diagnostics.capture_started);
    assert!(down.binding_reports.is_empty());
    assert!(down.component_events.iter().any(|event| {
        event.node_id == id(4)
            && event.binding_id == "ScrollbarThumb/DragBegin"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::BeginDrag { property } if property == "scroll_offset"
            )
    }));

    let drag = move_pointer(&mut surface, 126.0, 70.0);
    assert_eq!(drag.handled_by, Some(id(4)));
    assert_eq!(surface.focus.captured, Some(id(4)));
    assert_thumb_drag_binding_report(&drag.binding_reports);
    assert!(drag.component_events.iter().any(|event| {
        event.node_id == id(4)
            && event.binding_id == "ScrollbarThumb/DragUpdate"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::DragDelta { property, delta }
                    if property == "scroll_offset" && (*delta - 150.0).abs() < f64::EPSILON
            )
    }));
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        150.0
    );

    let up = release_primary(&mut surface, 126.0, 70.0);
    assert_eq!(up.handled_by, Some(id(4)));
    assert_eq!(up.released_capture, Some(id(4)));
    assert_eq!(surface.focus.captured, None);
    assert!(up.diagnostics.capture_released);
    assert!(up.binding_reports.is_empty());
    assert!(up.component_events.iter().any(|event| {
        event.node_id == id(4)
            && event.binding_id == "ScrollbarThumb/DragEnd"
            && matches!(
                &event.envelope.event,
                UiComponentEvent::EndDrag { property } if property == "scroll_offset"
            )
    }));
}

#[test]
fn disabled_scrollbar_thumb_does_not_capture_or_scroll() {
    let mut surface =
        scrollbar_surface_with_thumb_frame(true, UiFrame::new(120.0, 0.0, 12.0, 20.0));
    surface.tree.node_mut(id(4)).unwrap().state_flags.enabled = false;

    let down = press_primary(&mut surface, 126.0, 10.0);
    assert_eq!(down.captured_by, None);
    assert_eq!(surface.focus.captured, None);
    assert!(down.binding_reports.is_empty());

    let drag = move_pointer(&mut surface, 126.0, 70.0);
    assert_eq!(drag.handled_by, None);
    assert!(drag.binding_reports.is_empty());
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );
}

#[test]
fn scrollbar_is_headless_in_accessibility_unless_authored_explicitly() {
    let mut surface = scrollbar_surface(false);

    let snapshot = surface.accessibility_snapshot();
    assert!(snapshot.node(id(3)).is_none());

    let metadata = surface
        .tree
        .node_mut(id(3))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata.a11y = UiAccessibilityContract {
        role: UiA11yRole::Scrollbar,
        name: Some("Results scroll bar".to_string()),
        ..UiAccessibilityContract::default()
    };

    let snapshot = surface.accessibility_snapshot();
    let node = snapshot
        .node(id(3))
        .expect("explicitly-authored scrollbar a11y node is retained");

    assert_eq!(node.role, UiA11yRole::Scrollbar);
    assert!(node.actions.is_empty());
}

fn assert_scrollbar_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].applied_count, 1);
    let update = reports[0]
        .updates
        .first()
        .expect("scrollbar binding update");
    assert_eq!(update.source.kind, UiBindingSourceKind::WidgetBehavior);
    assert_eq!(update.source.node_id, Some(id(3)));
    assert_eq!(update.source.property.as_deref(), Some("scroll_target"));
    assert_eq!(update.target.kind, UiBindingTargetKind::RuntimeState);
    assert_eq!(update.target.node_id, Some(id(2)));
    assert_eq!(update.target.property.as_deref(), Some("scroll_offset"));
    assert_eq!(update.previous, Some(UiValue::Float(0.0)));
    assert_eq!(update.value, UiValue::Float(100.0));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Layout));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Render));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Input));
}

fn assert_thumb_drag_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].applied_count, 1);
    let update = reports[0]
        .updates
        .first()
        .expect("scrollbar thumb drag binding update");
    assert_eq!(update.source.kind, UiBindingSourceKind::WidgetBehavior);
    assert_eq!(update.source.node_id, Some(id(4)));
    assert_eq!(update.source.property.as_deref(), Some("scroll_thumb"));
    assert_eq!(update.target.kind, UiBindingTargetKind::RuntimeState);
    assert_eq!(update.target.node_id, Some(id(2)));
    assert_eq!(update.target.property.as_deref(), Some("scroll_offset"));
    assert_eq!(update.previous, Some(UiValue::Float(0.0)));
    assert_eq!(update.value, UiValue::Float(150.0));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Layout));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Render));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Input));
}

fn assert_accessibility_scroll_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].applied_count, 1);
    let update = reports[0]
        .updates
        .first()
        .expect("accessibility scroll binding update");
    assert_eq!(update.source.kind, UiBindingSourceKind::AccessibilityAction);
    assert_eq!(update.source.node_id, Some(id(2)));
    assert_eq!(update.source.property.as_deref(), Some("scroll_to"));
    assert_eq!(update.target.kind, UiBindingTargetKind::RuntimeState);
    assert_eq!(update.target.node_id, Some(id(2)));
    assert_eq!(update.target.property.as_deref(), Some("scroll_offset"));
    assert_eq!(update.previous, Some(UiValue::Float(0.0)));
    assert_eq!(update.value, UiValue::Float(64.0));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Layout));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Render));
    assert!(update.dirty.contains(&UiBindingDirtyDomain::Input));
}

#[test]
fn accessibility_scroll_to_mutates_scrollable_container_offset() {
    let mut surface = scrollbar_surface(false);

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &crate::ui::dispatch::UiNavigationDispatcher::default(),
            UiInputEvent::Accessibility(UiAccessibilityInputEvent {
                metadata: UiInputEventMetadata::default(),
                request: UiAccessibilityActionRequest {
                    target: id(2),
                    action: UiAccessibilityAction::ScrollTo,
                    numeric_value: Some(64.0),
                    ..UiAccessibilityActionRequest::default()
                },
            }),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("accessibility.scroll_to")
    );
    assert_accessibility_scroll_binding_report(&result.binding_reports);
    assert_eq!(
        surface
            .tree
            .node(id(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        64.0
    );
}
