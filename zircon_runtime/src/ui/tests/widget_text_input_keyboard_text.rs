use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
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
fn text_input_keyboard_text_payload_inserts_printable_text() {
    let mut surface = text_input_surface("ab", 1, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_text(&mut surface, "Z", 90, Some("Z"), |metadata| {
        metadata.modifiers.shift = true;
    });

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_payload")
    );
    assert_eq!(text_attr(&surface, "content"), "aZb");
    assert_eq!(int_attr(&surface, "caret_offset"), 2);
    assert_eq!(int_attr(&surface, "selection_anchor"), 2);
    assert_eq!(int_attr(&surface, "selection_focus"), 2);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("aZb".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_text_payload_uses_constraints_and_selection() {
    let mut surface = text_input_surface_with_selection(
        "a9",
        1,
        0,
        1,
        [
            ("input_filter", toml::Value::String("digits".to_string())),
            ("max_chars", toml::Value::Integer(3)),
        ],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_text(&mut surface, "1", 49, Some("A12"), |_| {});

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "129");
    assert_eq!(int_attr(&surface, "caret_offset"), 2);
    assert_eq!(int_attr(&surface, "selection_anchor"), 2);
    assert_eq!(int_attr(&surface, "selection_focus"), 2);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "content".to_string(),
            value: UiValue::String("129".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn text_input_keyboard_text_payload_rejects_stale_disabled_focus_owner() {
    let mut surface = text_input_surface("ab", 1, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();
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

    let result = dispatch_key_with_text(&mut surface, "Z", 90, Some("Z"), |_| {});

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "owner route rejected"));
    assert_eq!(text_attr(&surface, "content"), "ab");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert_eq!(int_attr(&surface, "selection_anchor"), 1);
    assert_eq!(int_attr(&surface, "selection_focus"), 1);
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}

#[test]
fn text_input_keyboard_text_payload_ignores_tab_navigation() {
    let mut surface = text_input_surface("ab", 1, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_text(&mut surface, "Tab", 9, Some("\t"), |_| {});

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(text_attr(&surface, "content"), "ab");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}

#[test]
fn text_input_keyboard_text_payload_newline_does_not_bypass_single_line_enter() {
    let mut surface = text_input_surface("ab", 1, [("multiline", toml::Value::Boolean(false))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_text(&mut surface, "Enter", 13, Some("\n"), |_| {});

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(text_attr(&surface, "content"), "ab");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
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

fn dispatch_key_with_text(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
    configure: impl FnOnce(&mut UiInputEventMetadata),
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(41), UiInputSequence::new(7));
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
                text: text.map(str::to_string),
            }),
        )
        .unwrap()
}

fn text_input_surface<const N: usize>(
    value: &str,
    caret_offset: usize,
    attributes: [(&str, toml::Value); N],
) -> UiSurface {
    text_input_surface_with_selection(value, caret_offset, caret_offset, caret_offset, attributes)
}

fn text_input_surface_with_selection<const N: usize>(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
    attributes: [(&str, toml::Value); N],
) -> UiSurface {
    let mut metadata_attributes = [
        (
            "content".to_string(),
            toml::Value::String(value.to_string()),
        ),
        (
            "caret_offset".to_string(),
            toml::Value::Integer(caret_offset as i64),
        ),
        (
            "selection_anchor".to_string(),
            toml::Value::Integer(selection_anchor as i64),
        ),
        (
            "selection_focus".to_string(),
            toml::Value::Integer(selection_focus as i64),
        ),
    ]
    .into_iter()
    .collect::<std::collections::BTreeMap<_, _>>();
    metadata_attributes.extend(
        attributes
            .into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    );

    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.keyboard_text"));
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
                    attributes: metadata_attributes,
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
