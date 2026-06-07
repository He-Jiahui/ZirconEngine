use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    dispatch::{
        UiDispatchDisposition, UiDispatchHostRequestKind, UiImeInputEvent, UiImeInputEventKind,
        UiInputEvent, UiInputEventMetadata, UiInputMethodRequestKind, UiInputSequence,
        UiInputTimestamp, UiTextByteRange,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn text_input_ime_preedit_refreshes_context_with_committed_surrounding_text() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(
        &mut surface,
        UiImeInputEventKind::Preedit,
        "XY",
        Some(UiTextByteRange::new(2, 2)),
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "aXYd");
    assert_eq!(text_attr(&surface, "composition_text"), "XY");
    assert_eq!(text_attr(&surface, "composition_restore_text"), "bc");
    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    assert_eq!(request.owner, UiNodeId::new(2));
    assert_eq!(request.surrounding_text.as_ref().unwrap().text, "abcd");
    assert_eq!(request.surrounding_text.as_ref().unwrap().cursor_byte, 3);
    assert_eq!(request.surrounding_text.as_ref().unwrap().anchor_byte, 3);
    assert!(request.cursor_rect.is_some());
    assert_eq!(request.composition_rects.len(), 1);
    assert_eq!(surface.input.input_method_request, Some(request.clone()));
}

#[test]
fn text_input_ime_commit_refreshes_context_after_composition_is_committed() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let preedit = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "XY", None);
    assert_eq!(preedit.reply.disposition, UiDispatchDisposition::Handled);

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Commit, "Z", None);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "aZd");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    assert_eq!(request.surrounding_text.as_ref().unwrap().text, "aZd");
    assert_eq!(request.surrounding_text.as_ref().unwrap().cursor_byte, 2);
    assert_eq!(request.surrounding_text.as_ref().unwrap().anchor_byte, 2);
    assert!(request.composition_rects.is_empty());
}

fn assert_input_method_request(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    kind: UiInputMethodRequestKind,
) -> &zircon_runtime_interface::ui::dispatch::UiInputMethodRequest {
    assert_eq!(result.host_requests.len(), 1);
    let UiDispatchHostRequestKind::InputMethod(request) = &result.host_requests[0].request else {
        panic!("expected input method host request");
    };
    assert_eq!(request.kind, kind);
    request
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
                    UiInputTimestamp::from_micros(43),
                    UiInputSequence::new(9),
                ),
                kind,
                text: text.to_string(),
                cursor_range,
            }),
        )
        .unwrap()
}

fn text_input_surface_with_selection(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.ime_context"));
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
