use crate::ui::{
    dispatch::{UiInputManager, UI_INPUT_ROUTE_ORDER},
    surface::UiSurface,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiDispatchPhase, UiInputEvent, UiInputEventMetadata,
        UiInputRoutePolicy, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
        UiKeyboardInputState, UiPointerDispatchEffect, UiPointerEvent, UiPointerId,
        UiPointerInputEvent, UiPointerSource, UiPopupInputEvent, UiPopupInputEventKind, UiWindowId,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint, UiSize},
    surface::{UiPointerButton, UiPointerEventKind},
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
    window::{
        UiWindowEvent, UiWindowEventMetadata, UiWindowInputPumpBatch, UiWindowInputPumpEvent,
        UiWindowMetrics, UiWindowPixelSize, UiWindowRedrawReason,
    },
};

mod route_matrix;
mod route_order;
mod touch_pointer;
mod window_timer;

fn route_matrix_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.route_matrix"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 70.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(100.0, 10.0, 70.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn double_click_manager_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.double_click"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 40.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MaterialButton".to_string(),
                    control_id: Some("PrimaryButton".to_string()),
                    bindings: vec![
                        binding("MaterialButton/Click", UiEventKind::Click),
                        binding("MaterialButton/DoubleClick", UiEventKind::DoubleClick),
                    ],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Button,
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn popup_matrix_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.popup_matrix"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            popup_node(
                UiNodeId::new(2),
                "root/popup",
                "MenuPopup",
                UiFrame::new(8.0, 8.0, 140.0, 74.0),
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            popup_node(
                UiNodeId::new(4),
                "root/popup/nested",
                "NestedMenuPopup",
                UiFrame::new(72.0, 40.0, 76.0, 40.0),
            ),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn popup_node(node_id: UiNodeId, path: &str, component: &str, frame: UiFrame) -> UiTreeNode {
    let binding_id = format!("{component}/ClosePopup");
    UiTreeNode::new(node_id, UiNodePath::new(path))
        .with_frame(frame)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(container_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: component.to_string(),
            attributes: [("popup_open".to_string(), toml::Value::Boolean(true))]
                .into_iter()
                .collect(),
            bindings: vec![binding(binding_id.as_str(), UiEventKind::Click)],
            widget: UiWidgetContract {
                behavior: UiWidgetBehavior::Popup,
                open_property: Some("popup_open".to_string()),
                ..UiWidgetContract::default()
            },
            ..UiTemplateNodeMetadata::default()
        })
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

fn container_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn pointer_event(kind: UiPointerEventKind, point: UiPoint) -> UiInputEvent {
    pointer_event_at(kind, point, 10)
}

fn pointer_event_at(
    kind: UiPointerEventKind,
    point: UiPoint,
    timestamp_micros: u64,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.timestamp = UiInputTimestamp::from_micros(timestamp_micros);
    metadata.sequence = UiInputSequence::new(timestamp_micros);
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn touch_pointer_event_at(
    pointer_id: UiPointerId,
    kind: UiPointerEventKind,
    point: UiPoint,
    timestamp_micros: u64,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.pointer_id = Some(pointer_id);
    metadata.pointer_source = UiPointerSource::Touch;
    metadata.timestamp = UiInputTimestamp::from_micros(timestamp_micros);
    metadata.sequence = UiInputSequence::new(timestamp_micros);
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(kind, point).with_button(UiPointerButton::Primary),
        precise_scroll: None,
    })
}

fn assert_pointer_button(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    expected: Option<UiPointerButton>,
) {
    match &result.event {
        UiInputEvent::Pointer(pointer) => assert_eq!(pointer.event.button, expected),
        other => panic!("expected pointer input event, got {other:?}"),
    }
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

fn popup_event(
    kind: UiPopupInputEventKind,
    popup_id: &str,
    owner: Option<UiNodeId>,
) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner,
        anchor: Some(UiPoint::new(8.0, 12.0)),
    })
}

fn assert_popup_node_open(surface: &UiSurface, node_id: UiNodeId, expected: bool) {
    let metadata = surface
        .tree
        .node(node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["popup_open"].as_bool(), Some(expected));
}

fn assert_popup_stack(surface: &UiSurface, expected: &[&str]) {
    assert_eq!(popup_stack_ids(surface), expected);
}

fn popup_stack_ids(surface: &UiSurface) -> Vec<&str> {
    surface
        .input
        .popup_stack
        .iter()
        .map(|popup| popup.popup_id.as_str())
        .collect()
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

fn window_metadata(sequence: u64) -> UiWindowEventMetadata {
    UiWindowEventMetadata::for_window(
        UiWindowId::new("main-window"),
        UiInputTimestamp::from_micros(100 + sequence),
        UiInputSequence::new(sequence),
    )
}
