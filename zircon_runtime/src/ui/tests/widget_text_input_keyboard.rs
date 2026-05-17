use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn text_input_keyboard_backspace_uses_widget_value_property() {
    let mut surface = text_input_surface("hello", 5);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(text_attr(&surface, "content"), "hell");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("hell".to_string()),
        }
    );
}

#[test]
fn text_input_keyboard_arrow_left_moves_caret_without_value_event() {
    let mut surface = text_input_surface("he", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "he");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(result.component_events.is_empty());
}

fn dispatch_key(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(30),
                    UiInputSequence::new(3),
                ),
                state: UiKeyboardInputState::Pressed,
                key_code,
                scan_code: None,
                physical_key: logical_key.to_string(),
                logical_key: logical_key.to_string(),
                text: None,
            }),
        )
        .unwrap()
}

fn text_input_surface(value: &str, caret_offset: usize) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.keyboard"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/search"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "SearchBox".to_string(),
                    attributes: [
                        (
                            "content".to_string(),
                            toml::Value::String(value.to_string()),
                        ),
                        (
                            "caret_offset".to_string(),
                            toml::Value::Integer(caret_offset as i64),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    bindings: vec![binding("SearchBox/Change", UiEventKind::Change)],
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::TextInput,
                        value_property: Some("content".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn text_attr(surface: &UiSurface, key: &str) -> String {
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get(key)
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn int_attr(surface: &UiSurface, key: &str) -> i64 {
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get(key)
        .and_then(toml::Value::as_integer)
        .unwrap_or_default()
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
