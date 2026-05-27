use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdateReport, UiBindingUpdateStatus, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    focus::UiFocusChangeReason,
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn mui_input_base_component_name_is_editable_text_owner() {
    let mut surface = mui_text_input_surface("InputBase", "a", 1, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_text(&mut surface, "b");

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("text.edit")
    );
    assert_eq!(value_attr(&surface), "ab");
    assert_eq!(int_attr(&surface, "caret_offset"), 2);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("ab".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn mui_input_base_read_only_alias_blocks_text_mutation_but_allows_navigation() {
    let mut surface = mui_text_input_surface(
        "InputBase",
        "hello",
        5,
        [("readOnly", toml::Value::Boolean(true))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let text = dispatch_text(&mut surface, "!");
    assert_eq!(text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "hello");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert!(text.component_events.is_empty());

    let arrow = dispatch_key(&mut surface, "ArrowLeft", 37);
    assert_eq!(arrow.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "hello");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
}

#[test]
fn mui_text_field_auto_focus_alias_resolves_initial_focus() {
    let mut surface = mui_text_input_surface(
        "TextField",
        "hello",
        0,
        [("autoFocus", toml::Value::Boolean(true))],
    );

    let event = surface
        .resolve_autofocus()
        .unwrap()
        .expect("MUI autoFocus alias should resolve");

    assert_eq!(event.reason, UiFocusChangeReason::Autofocus);
    assert_eq!(event.current, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
}

fn dispatch_text(
    surface: &mut UiSurface,
    text: &str,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(41),
                    UiInputSequence::new(14),
                ),
                text: text.to_string(),
            }),
        )
        .unwrap()
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
                    UiInputTimestamp::from_micros(42),
                    UiInputSequence::new(15),
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

fn mui_text_input_surface(
    component: &str,
    value: &str,
    caret_offset: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.mui"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    let attributes = [
        ("value".to_string(), toml::Value::String(value.to_string())),
        (
            "caret_offset".to_string(),
            toml::Value::Integer(caret_offset as i64),
        ),
    ]
    .into_iter()
    .chain(
        extra_attributes
            .into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    )
    .collect();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/mui_text"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes,
                    bindings: vec![binding("MuiTextField/Change", UiEventKind::Change)],
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn value_attr(surface: &UiSurface) -> String {
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("value")
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

fn assert_widget_binding_report(reports: &[UiBindingUpdateReport]) {
    let value_updates: Vec<_> = reports
        .iter()
        .flat_map(|report| report.updates.iter())
        .filter(|update| {
            update.source.kind == UiBindingSourceKind::WidgetBehavior
                && update.source.node_id == Some(UiNodeId::new(2))
                && update.source.property.as_deref() == Some("value")
        })
        .collect();
    assert_eq!(value_updates.len(), 1, "{reports:#?}");
    let update = value_updates[0];
    assert_eq!(update.source.kind, UiBindingSourceKind::WidgetBehavior);
    assert_eq!(update.source.node_id, Some(UiNodeId::new(2)));
    assert_eq!(update.source.property.as_deref(), Some("value"));
    assert_eq!(update.target.node_id, Some(UiNodeId::new(2)));
    assert_eq!(update.target.property.as_deref(), Some("value"));
    assert_eq!(update.status, UiBindingUpdateStatus::Applied);
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
