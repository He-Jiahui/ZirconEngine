use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
    component::UiDragPhase,
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiInputEvent, UiInputEventMetadata,
        UiInputSequence, UiInputTimestamp, UiPointerId, UiPointerInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn text_input_pointer_press_moves_caret_and_captures_pointer() {
    let mut surface = text_input_surface("abcd", 0);

    let result = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(20.6, 10.0),
        Some(UiPointerButton::Primary),
        UiPointerId::new(7),
        |_| {},
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("pointer.text_press")
    );
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.captured_pointer_id, Some(UiPointerId::new(7)));
    assert_eq!(int_attr(&surface, "caret_offset"), 3);
    assert_eq!(int_attr(&surface, "selection_anchor"), 3);
    assert_eq!(int_attr(&surface, "selection_focus"), 3);
    assert!(result
        .reply
        .effects
        .iter()
        .any(|effect| matches!(effect, UiDispatchEffect::CapturePointer { .. })));
    assert_widget_binding_report(&result.binding_reports);
    assert_drag_metrics(
        &result,
        UiDragPhase::Begin,
        UiPoint::new(20.6, 10.0),
        UiPoint::new(20.6, 10.0),
        0.0,
    );
}

#[test]
fn text_input_shift_pointer_press_extends_selection_from_existing_caret() {
    let mut surface = text_input_surface("abcd", 1);

    let result = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(27.0, 10.0),
        Some(UiPointerButton::Primary),
        UiPointerId::new(8),
        |metadata| metadata.modifiers.shift = true,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 1);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_pointer_drag_extends_selection_until_release() {
    let mut surface = text_input_surface("abcd", 0);
    let pointer_id = UiPointerId::new(9);

    let press = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(8.2, 10.0),
        Some(UiPointerButton::Primary),
        pointer_id,
        |_| {},
    );
    assert_drag_metrics(
        &press,
        UiDragPhase::Begin,
        UiPoint::new(8.2, 10.0),
        UiPoint::new(8.2, 10.0),
        0.0,
    );
    let drag = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(27.0, 10.0),
        None,
        pointer_id,
        |_| {},
    );

    assert_eq!(drag.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        drag.diagnostics.handled_phase.as_deref(),
        Some("pointer.text_drag")
    );
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert_drag_metrics(
        &drag,
        UiDragPhase::Update,
        UiPoint::new(8.2, 10.0),
        UiPoint::new(27.0, 10.0),
        18.8,
    );

    let release = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(27.0, 10.0),
        Some(UiPointerButton::Primary),
        pointer_id,
        |_| {},
    );

    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(
        release.diagnostics.handled_phase.as_deref(),
        Some("pointer.text_release")
    );
    assert_drag_metrics(
        &release,
        UiDragPhase::End,
        UiPoint::new(8.2, 10.0),
        UiPoint::new(27.0, 10.0),
        18.8,
    );
}

#[test]
fn text_input_pointer_press_handles_empty_value_layout() {
    let mut surface = text_input_surface("", 0);

    let result = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(8.0, 10.0),
        Some(UiPointerButton::Primary),
        UiPointerId::new(10),
        |_| {},
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
}

#[test]
fn disabled_text_input_pointer_press_does_not_move_caret_or_capture() {
    let mut surface = text_input_surface("abcd", 1);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("disabled".to_string(), toml::Value::Boolean(true));

    let result = dispatch_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(27.0, 10.0),
        Some(UiPointerButton::Primary),
        UiPointerId::new(11),
        |_| {},
    );

    assert_ne!(
        result.diagnostics.handled_phase.as_deref(),
        Some("pointer.text_press")
    );
    assert!(result.binding_reports.is_empty());
    assert!(result.drag.is_none());
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert_eq!(int_attr(&surface, "selection_anchor"), 1);
    assert_eq!(int_attr(&surface, "selection_focus"), 1);
}

fn assert_widget_binding_report(
    reports: &[zircon_runtime_interface::ui::binding::UiBindingUpdateReport],
) {
    assert!(!reports.is_empty());
    assert!(reports.iter().any(|report| {
        report
            .updates
            .first()
            .is_some_and(|update| update.source.kind == UiBindingSourceKind::WidgetBehavior)
    }));
}

fn assert_drag_metrics(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    phase: UiDragPhase,
    start: UiPoint,
    current: UiPoint,
    distance: f32,
) {
    let drag = result.drag.expect("pointer drag metrics");
    assert_eq!(drag.phase, phase);
    assert_point_close(drag.start, start);
    assert_point_close(drag.current, current);
    assert_point_close(
        drag.delta,
        UiPoint::new(current.x - start.x, current.y - start.y),
    );
    assert!((drag.distance - distance).abs() < 0.001);
}

fn assert_point_close(actual: UiPoint, expected: UiPoint) {
    assert!((actual.x - expected.x).abs() < 0.001);
    assert!((actual.y - expected.y).abs() < 0.001);
}

fn dispatch_pointer(
    surface: &mut UiSurface,
    kind: UiPointerEventKind,
    point: UiPoint,
    button: Option<UiPointerButton>,
    pointer_id: UiPointerId,
    configure: impl FnOnce(&mut UiInputEventMetadata),
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(73), UiInputSequence::new(19));
    metadata.pointer_id = Some(pointer_id);
    configure(&mut metadata);
    let mut event = zircon_runtime_interface::ui::dispatch::UiPointerEvent::new(kind, point);
    event.button = button;
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata,
                event,
                precise_scroll: None,
            }),
        )
        .unwrap()
}

fn text_input_surface(value: &str, caret_offset: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.pointer"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/input"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TextField".to_string(),
                    attributes: [
                        ("value".to_string(), toml::Value::String(value.to_string())),
                        (
                            "caret_offset".to_string(),
                            toml::Value::Integer(caret_offset as i64),
                        ),
                        (
                            "selection_anchor".to_string(),
                            toml::Value::Integer(caret_offset as i64),
                        ),
                        (
                            "selection_focus".to_string(),
                            toml::Value::Integer(caret_offset as i64),
                        ),
                        ("font_size".to_string(), toml::Value::Float(10.0)),
                        ("line_height".to_string(), toml::Value::Float(12.0)),
                        ("wrap".to_string(), toml::Value::String("none".to_string())),
                    ]
                    .into_iter()
                    .collect(),
                    bindings: vec![binding("TextField/Change", UiEventKind::Change)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn int_attr(surface: &UiSurface, key: &str) -> i64 {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(toml::Value::as_integer)
        .unwrap_or_default()
}

fn binding(path: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: path.to_string(),
        event,
        route: Some(path.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn focusable_state() -> UiStateFlags {
    UiStateFlags {
        focusable: true,
        enabled: true,
        visible: true,
        ..UiStateFlags::default()
    }
}
