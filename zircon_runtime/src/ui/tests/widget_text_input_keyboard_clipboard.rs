use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{
        UiClipboardInputEvent, UiClipboardRequest, UiClipboardRequestKind, UiClipboardTransferId,
        UiClipboardTransferIntent, UiClipboardTransferOutcome, UiClipboardTransferStatus,
        UiDispatchDisposition, UiDispatchEffect, UiDispatchReply, UiInputEvent,
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
        UiKeyboardInputState, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::{UiActionRef, UiBindingRef},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn text_input_keyboard_read_only_paste_blocks_clipboard_request() {
    let mut surface =
        text_input_surface_with_attributes("alpha", 5, [("read_only", toml::Value::Boolean(true))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "v", 86);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_paste")
    );
    assert!(
        result
            .diagnostics
            .notes
            .iter()
            .any(|note| note == "clipboard paste blocked by read-only text")
    );
    assert_eq!(text_attr(&surface, "content"), "alpha");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 5);
    assert!(result.reply.effects.is_empty());
    assert!(result.applied_effects.is_empty());
    assert!(result.host_requests.is_empty());
    assert!(result.component_events.is_empty());
    assert!(result.binding_reports.is_empty());
}

#[test]
fn secure_text_input_copy_and_cut_preserve_text_without_clipboard_write() {
    for (logical_key, key_code, phase) in [
        ("c", 67, "keyboard.clipboard_copy"),
        ("x", 88, "keyboard.clipboard_cut"),
    ] {
        let mut surface = text_input_surface_with_attributes(
            "alpha",
            5,
            [
                ("input_kind", toml::Value::String("password".to_string())),
                ("selection_anchor", toml::Value::Integer(0)),
                ("selection_focus", toml::Value::Integer(5)),
            ],
        );
        surface.focus_node(UiNodeId::new(2)).unwrap();

        let result = dispatch_key_with_control(&mut surface, logical_key, key_code);

        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(result.diagnostics.handled_phase.as_deref(), Some(phase));
        assert!(
            result
                .diagnostics
                .notes
                .iter()
                .any(|note| note == "clipboard copy and cut disabled for secure text input")
        );
        assert_eq!(text_attr(&surface, "content"), "alpha");
        assert_eq!(int_attr(&surface, "caret_offset"), 5);
        assert_eq!(int_attr(&surface, "selection_anchor"), 0);
        assert_eq!(int_attr(&surface, "selection_focus"), 5);
        assert!(result.reply.effects.is_empty());
        assert!(result.applied_effects.is_empty());
        assert!(result.host_requests.is_empty());
        assert!(result.component_events.is_empty());
        assert!(result.binding_reports.is_empty());
    }
}

#[test]
fn secure_text_change_publishes_only_latest_resolvable_opaque_reference() {
    let mut surface = text_input_surface_with_attributes(
        "alpha",
        5,
        [("input_kind", toml::Value::String("password".to_string()))],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = dispatch_text(&mut surface, "Z", 11);
    assert!(first.diagnostics.secure_text_redacted);
    let first_reference = match &first.component_events.as_slice() {
        [report] => match &report.event {
            UiComponentEvent::SecureValueChanged {
                property,
                reference,
            } => {
                assert_eq!(property, "content");
                reference.clone()
            }
            event => panic!("expected secure value event, got {event:?}"),
        },
        reports => panic!("expected one secure value report, got {reports:?}"),
    };
    assert_eq!(
        surface.resolve_secure_text_value(&first_reference),
        Some("alphaZ")
    );
    let first_json = serde_json::to_string(&first).unwrap();
    assert!(!first_json.contains("alpha"));

    let second = dispatch_text(&mut surface, "Q", 12);
    let second_reference = match &second.component_events[0].event {
        UiComponentEvent::SecureValueChanged { reference, .. } => reference,
        event => panic!("expected secure value event, got {event:?}"),
    };
    assert_eq!(surface.resolve_secure_text_value(&first_reference), None);
    assert_eq!(
        surface.resolve_secure_text_value(second_reference),
        Some("alphaZQ")
    );
    let second_json = serde_json::to_string(&second).unwrap();
    assert!(!second_json.contains("alpha"));

    let other_surface = text_input_surface_with_attributes(
        "alphaZQ",
        7,
        [("input_kind", toml::Value::String("password".to_string()))],
    );
    assert_eq!(
        other_surface.resolve_secure_text_value(second_reference),
        None
    );

    let submit = dispatch_key_with_metadata(&mut surface, "Enter", 13, |_| {});
    let submit_reference = match &submit.component_events[0].event {
        UiComponentEvent::SecureCommit { reference, .. } => reference,
        event => panic!("expected secure commit event, got {event:?}"),
    };
    assert_eq!(surface.resolve_secure_text_value(second_reference), None);
    assert_eq!(
        surface.resolve_secure_text_value(submit_reference),
        Some("alphaZQ")
    );
    assert_eq!(
        surface.clone().resolve_secure_text_value(submit_reference),
        None
    );
    let submit_json = serde_json::to_string(&submit).unwrap();
    assert!(!submit_json.contains("alpha"));
}

#[test]
fn secure_text_change_and_submit_preserve_authored_route_without_secret_payload() {
    let mut surface = text_input_surface_with_attributes(
        "",
        0,
        [("input_kind", toml::Value::String("password".to_string()))],
    );
    let metadata = surface
        .tree
        .node_mut(UiNodeId::new(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap();
    metadata.bindings[0].action = Some(UiActionRef {
        route: Some("woc.shell.auth.set_password".to_string()),
        ..UiActionRef::default()
    });
    metadata.bindings[1].action = Some(UiActionRef {
        route: Some("woc.shell.auth.submit".to_string()),
        ..UiActionRef::default()
    });
    surface.rebuild();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let changed = dispatch_text(&mut surface, "correct horse", 11);
    let changed_action = changed.component_events[0]
        .template_action
        .as_ref()
        .expect("secure change should retain its authored route");
    assert_eq!(changed_action.target_id(), "woc.shell.auth.set_password");
    assert!(changed_action.payload.is_empty());
    let changed_json = serde_json::to_string(&changed).unwrap();
    assert!(!changed_json.contains("correct horse"));

    let submitted = dispatch_key_with_metadata(&mut surface, "Enter", 13, |_| {});
    let submitted_action = submitted.component_events[0]
        .template_action
        .as_ref()
        .expect("secure submit should retain its authored route");
    assert_eq!(submitted_action.target_id(), "woc.shell.auth.submit");
    assert!(submitted_action.payload.is_empty());
    let submitted_json = serde_json::to_string(&submitted).unwrap();
    assert!(!submitted_json.contains("correct horse"));
}

#[test]
fn secure_text_input_paste_still_requests_clipboard_read() {
    let mut surface =
        text_input_surface_with_attributes("alpha", 5, [("secure", toml::Value::Boolean(true))]);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key_with_control(&mut surface, "v", 86);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.clipboard_paste")
    );
    assert_eq!(result.host_requests.len(), 1);
    let zircon_runtime_interface::ui::dispatch::UiDispatchHostRequestKind::Clipboard(request) =
        &result.host_requests[0].request
    else {
        panic!("expected clipboard host request");
    };
    assert_eq!(request.kind, UiClipboardRequestKind::ReadText);
    assert_eq!(request.text, None);

    let request = request.clone();
    let completion = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Clipboard(UiClipboardInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(45),
                    UiInputSequence::new(13),
                ),
                transfer_id: request.transfer_id,
                owner: request.owner,
                outcome: UiClipboardTransferOutcome::ReadText {
                    text: "bravo".to_string(),
                },
            }),
        )
        .unwrap();

    assert_eq!(
        completion
            .diagnostics
            .clipboard_transfer
            .as_ref()
            .map(|receipt| receipt.status),
        Some(UiClipboardTransferStatus::Applied)
    );
    assert!(completion.diagnostics.secure_text_redacted);
    let reference = match &completion.component_events[0].event {
        UiComponentEvent::SecureValueChanged { reference, .. } => reference,
        event => panic!("expected secure value event, got {event:?}"),
    };
    assert_eq!(
        surface.resolve_secure_text_value(reference),
        Some("alphabravo")
    );
    let encoded = serde_json::to_string(&completion).unwrap();
    assert!(!encoded.contains("alpha"));
    assert!(!encoded.contains("bravo"));
}

#[test]
fn secure_text_input_rejects_forged_clipboard_write_effect() {
    let mut surface =
        text_input_surface_with_attributes("alpha", 5, [("secure", toml::Value::Boolean(true))]);
    let reply = UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestClipboard {
        request: UiClipboardRequest {
            transfer_id: UiClipboardTransferId::issue(),
            intent: UiClipboardTransferIntent::Copy,
            expected_edit_revision: 0,
            kind: UiClipboardRequestKind::WriteText,
            owner: UiNodeId::new(2),
            text: Some("alpha".to_string()),
        },
    });

    let result = surface.apply_dispatch_reply(keyboard_event(), reply);

    assert!(result.applied_effects.is_empty());
    assert!(result.host_requests.is_empty());
    assert_eq!(result.rejected_effects.len(), 1);
    assert_eq!(
        result.rejected_effects[0].reason,
        "clipboard write is disabled for secure text input"
    );
    let encoded = serde_json::to_string(&result).unwrap();
    assert!(!encoded.contains("alpha"));
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

fn dispatch_text(
    surface: &mut UiSurface,
    text: &str,
    sequence: u64,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Text(UiTextInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(44 + sequence),
                    UiInputSequence::new(sequence),
                ),
                text: text.to_string(),
            }),
        )
        .unwrap()
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(43),
            UiInputSequence::new(9),
        ),
        state: UiKeyboardInputState::Pressed,
        key_code: 0,
        scan_code: None,
        physical_key: "Unidentified".to_string(),
        logical_key: "Unidentified".to_string(),
        text: None,
    })
}

fn dispatch_key_with_metadata(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
    configure: impl FnOnce(&mut UiInputEventMetadata),
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(42), UiInputSequence::new(8));
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

fn text_input_surface_with_attributes(
    value: &str,
    caret_offset: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.widget.text_input.keyboard_clipboard",
    ));
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
            toml::Value::Integer(caret_offset as i64),
        ),
        (
            "selection_focus".to_string(),
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
        component_event: super::typed_component_event_kind_for_test(id),
        id: id.to_string(),
        event,
        mode: Default::default(),
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
