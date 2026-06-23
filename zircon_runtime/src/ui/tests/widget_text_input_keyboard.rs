use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiClipboardRequestKind, UiDispatchDisposition, UiDispatchHostRequestKind, UiImeInputEvent,
        UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiTextByteRange, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

mod basic_editing;
mod clipboard_newline;
mod selection_navigation;
mod text_ime;
mod word_shortcuts;

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

fn assert_clipboard_request(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    kind: UiClipboardRequestKind,
    text: Option<&str>,
) {
    assert_eq!(result.reply.effects.len(), 1);
    assert_eq!(result.applied_effects.len(), 1);
    assert_eq!(result.host_requests.len(), 1);
    let UiDispatchHostRequestKind::Clipboard(request) = &result.host_requests[0].request else {
        panic!("expected clipboard host request");
    };
    assert_eq!(request.kind, kind);
    assert_eq!(request.owner, UiNodeId::new(2));
    assert_eq!(request.text.as_deref(), text);
}

fn dispatch_key(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(
        surface,
        logical_key,
        key_code,
        |_: &mut UiInputEventMetadata| {},
    )
}

fn dispatch_key_with_shift(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.shift = true;
    })
}

fn dispatch_key_with_control(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.control = true;
    })
}

fn dispatch_key_with_control_shift(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_metadata(surface, logical_key, key_code, |metadata| {
        metadata.modifiers.control = true;
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
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(30), UiInputSequence::new(3));
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
                    UiInputTimestamp::from_micros(31),
                    UiInputSequence::new(4),
                ),
                text: text.to_string(),
            }),
        )
        .unwrap()
}

fn dispatch_ime(
    surface: &mut UiSurface,
    kind: UiImeInputEventKind,
    text: &str,
    cursor_range: Option<UiTextByteRange>,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(32),
                    UiInputSequence::new(5),
                ),
                kind,
                text: text.to_string(),
                cursor_range,
                delete_surrounding: None,
            }),
        )
        .unwrap()
}

fn text_input_surface(value: &str, caret_offset: usize) -> UiSurface {
    text_input_surface_with_attributes(value, caret_offset, [])
}

fn text_input_surface_with_selection(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
) -> UiSurface {
    text_input_surface_with_selection_and_attributes(
        value,
        caret_offset,
        selection_anchor,
        selection_focus,
        [],
    )
}

fn text_input_surface_with_selection_and_attributes(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    text_input_surface_with_attributes(
        value,
        caret_offset,
        [
            (
                "selection_anchor",
                toml::Value::Integer(selection_anchor as i64),
            ),
            (
                "selection_focus",
                toml::Value::Integer(selection_focus as i64),
            ),
        ]
        .into_iter()
        .chain(extra_attributes),
    )
}

fn text_input_surface_with_attributes(
    value: &str,
    caret_offset: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.keyboard"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    let attributes = [
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/search"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "SearchBox".to_string(),
                    attributes,
                    bindings: vec![
                        binding("SearchBox/Change", UiEventKind::Change),
                        binding("SearchBox/Submit", UiEventKind::Submit),
                    ],
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

fn combining_acute_cluster() -> &'static str {
    "e\u{301}"
}
