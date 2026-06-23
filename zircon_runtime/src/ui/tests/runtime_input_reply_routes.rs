use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityActionRequest},
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentKeyboardAction, UiDragPhase, UiValue},
    dispatch::{
        UiAccessibilityInputEvent, UiAnalogInputEvent, UiDispatchDisposition, UiDispatchEffect,
        UiDispatchHostRequestKind, UiDispatchPhase, UiDispatchReply, UiDispatchReplyStep,
        UiDragDropInputEvent, UiDragDropInputEventKind, UiDragSessionId, UiFocusEffectReason,
        UiImeInputEvent, UiImeInputEventKind, UiInputDispatchResult, UiInputEvent,
        UiInputEventMetadata, UiInputRoutePolicy, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiMouseMotionInputEvent,
        UiNavigationInputEvent, UiPointerCaptureReason, UiPointerDispatchEffect, UiPointerEvent,
        UiPointerId, UiPointerInputEvent, UiPointerLockPolicy, UiPointerSource, UiPopupEffectKind,
        UiPopupInputEvent, UiPopupInputEventKind, UiPreciseScrollDelta,
        UiSubmenuHoverTimerInputEvent, UiTextInputEvent, UiToastTimerInputEvent,
        UiTooltipEffectKind, UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
        UiTransientDismissalReason, UiTransientDismissalTarget,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    focus::UiFocusedInputKind,
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
    },
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

mod analog_navigation_routes;
mod drag_drop_routes;
mod focus_text_accessibility_routes;
mod gamepad_navigation_routes;
mod keyboard_activation_routes;
mod keyboard_navigation_routes;
mod keyboard_popup_routes;
mod pointer_bubble_routes;
mod pointer_capture_routes;
mod pointer_hover_routes;
mod pointer_popup_routes;
mod popup_routes;
mod route_trace_routes;
mod table_pointer_routes;
mod tooltip_timer_routes;
mod touch_pointer_routes;
mod tree_view_pointer_routes;

fn assert_two_node_bubble_handled_at_target(result: &UiInputDispatchResult) {
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[0].target,
        Some(UiNodeId::new(1))
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].handler,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 0);
    assert!(result.diagnostics.route_steps[2].stopped);
}

fn capture_pointer_for_test(surface: &mut UiSurface, pointer_id: UiPointerId, owner: UiNodeId) {
    surface.focus.captured = Some(owner);
    surface.input.set_pointer_capture_for_id(pointer_id, owner);
}

fn assert_pointer_capture(surface: &UiSurface, pointer_id: UiPointerId, owner: UiNodeId) {
    assert_eq!(surface.input.pointer_capture_owner(pointer_id), Some(owner));
}

fn assert_no_pointer_capture(surface: &UiSurface) {
    assert_eq!(surface.input.active_pointer_capture(), None);
}

fn route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn press_release_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings: vec![
            binding("MaterialButton/Press", UiEventKind::Press),
            binding("MaterialButton/Release", UiEventKind::Release),
            binding("MaterialButton/Click", UiEventKind::Click),
        ],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn double_click_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings: vec![binding(
            "MaterialButton/DoubleClick",
            UiEventKind::DoubleClick,
        )],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn scroll_route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.scroll"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 0,
                    }),
                }))
                .with_scroll_state(UiScrollState::default())
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
                .with_state_flags(input_state()),
            )
            .unwrap();
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();
    surface
}

fn editable_route_surface(value: &str, caret_offset: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.text"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/editable"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    control_id: Some("EditableText".to_string()),
                    bindings: vec![
                        binding("EditableText/Change", UiEventKind::Change),
                        binding("EditableText/Submit", UiEventKind::Submit),
                    ],
                    attributes: toml::from_str(&format!(
                        r#"
value = "{}"
caret_offset = {}
editable_text = true
"#,
                        value, caret_offset
                    ))
                    .unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn input_state() -> UiStateFlags {
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

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: input_metadata(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "KeyA".to_string(),
        text: None,
    })
}

fn text_event(text: &str) -> UiInputEvent {
    UiInputEvent::Text(UiTextInputEvent {
        metadata: input_metadata(),
        text: text.to_string(),
    })
}

fn ime_event(kind: UiImeInputEventKind, text: &str) -> UiInputEvent {
    UiInputEvent::Ime(UiImeInputEvent {
        metadata: input_metadata(),
        kind,
        text: text.to_string(),
        cursor_range: None,
        delete_surrounding: None,
    })
}

fn navigation_event(kind: UiNavigationEventKind) -> UiInputEvent {
    UiInputEvent::Navigation(UiNavigationInputEvent {
        metadata: input_metadata(),
        kind,
    })
}

fn raw_mouse_motion_event(delta_x: f32, delta_y: f32) -> UiInputEvent {
    UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
        metadata: input_metadata(),
        delta_x,
        delta_y,
    })
}

fn pointer_event(kind: UiPointerEventKind, point: UiPoint) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn pointer_event_with_click_count(
    kind: UiPointerEventKind,
    point: UiPoint,
    click_count: u8,
) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(kind, point)
            .with_button(UiPointerButton::Primary)
            .with_click_count(click_count),
        precise_scroll: None,
    })
}

fn scroll_event(point: UiPoint, scroll_delta: f32) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(UiPointerEventKind::Scroll, point)
            .with_scroll_delta(scroll_delta),
        precise_scroll: Some(UiPreciseScrollDelta::pixels(0.0, scroll_delta)),
    })
}

fn touch_pointer_event_with_id(
    pointer_id: UiPointerId,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.pointer_id = Some(pointer_id);
    metadata.pointer_source = UiPointerSource::Touch;
    touch_pointer_event_from_metadata(metadata, kind, point)
}

fn touch_pointer_event_from_metadata(
    metadata: UiInputEventMetadata,
    kind: UiPointerEventKind,
    point: UiPoint,
) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn drag_drop_event(
    kind: UiDragDropInputEventKind,
    session_id: Option<UiDragSessionId>,
    point: UiPoint,
) -> UiInputEvent {
    UiInputEvent::DragDrop(UiDragDropInputEvent {
        metadata: input_metadata(),
        kind,
        session_id,
        point,
        payload: None,
    })
}

fn popup_event(kind: UiPopupInputEventKind, popup_id: &str) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner: Some(UiNodeId::new(2)),
        anchor: Some(UiPoint::new(8.0, 12.0)),
    })
}

fn popup_event_without_owner(kind: UiPopupInputEventKind, popup_id: &str) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner: None,
        anchor: None,
    })
}

fn tooltip_event(
    kind: UiTooltipTimerInputEventKind,
    tooltip_id: &str,
    owner: Option<UiNodeId>,
) -> UiInputEvent {
    UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
        metadata: input_metadata(),
        kind,
        tooltip_id: tooltip_id.to_string(),
        owner,
    })
}

fn accessibility_event(target: UiNodeId, action: UiAccessibilityAction) -> UiInputEvent {
    UiInputEvent::Accessibility(UiAccessibilityInputEvent {
        metadata: input_metadata(),
        request: UiAccessibilityActionRequest {
            target,
            action,
            ..UiAccessibilityActionRequest::default()
        },
    })
}

fn editable_attr_string(surface: &UiSurface, key: &str) -> String {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
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

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
