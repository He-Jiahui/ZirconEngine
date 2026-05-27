use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
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
fn text_input_keyboard_super_arrow_left_moves_to_hard_line_start() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 10);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_super(&mut surface, "ArrowLeft", 37);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 4);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_super_shift_arrow_right_extends_to_hard_line_end() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 8);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_super_shift(&mut surface, "ArrowRight", 39);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 13);
    assert_eq!(int_attr(&surface, "selection_anchor"), 8);
    assert_eq!(int_attr(&surface, "selection_focus"), 13);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_alt_home_extends_to_hard_line_start() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 8);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_alt_shift(&mut surface, "Home", 36);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 8);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_alt_end_moves_to_hard_line_end() {
    let value = "one\ntwo three\nfour";
    let mut surface = text_input_surface(value, 8);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_alt(&mut surface, "End", 35);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), value);
    assert_eq!(int_attr(&surface, "caret_offset"), 13);
    assert_eq!(int_attr(&surface, "selection_anchor"), 13);
    assert_eq!(int_attr(&surface, "selection_focus"), 13);
    assert!(result.component_events.is_empty());
    assert_widget_binding_report(&result.binding_reports);
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

fn dispatch_key_with_super(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.super_key = true;
    })
}

fn dispatch_key_with_super_shift(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.super_key = true;
        metadata.modifiers.shift = true;
    })
}

fn dispatch_key_with_alt(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.alt = true;
    })
}

fn dispatch_key_with_alt_shift(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.alt = true;
        metadata.modifiers.shift = true;
    })
}

fn dispatch_key_with_metadata(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
    configure: impl FnOnce(&mut UiInputEventMetadata),
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(40), UiInputSequence::new(6));
    configure(&mut metadata);
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata,
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
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.hard_line"));
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
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_string()
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
