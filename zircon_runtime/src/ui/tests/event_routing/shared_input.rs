use super::*;

#[test]
fn shared_input_dispatch_routes_keyboard_text_ime_and_preserves_scroll_diagnostics() {
    let mut surface = button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    let keyboard = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            keyboard_event(),
        )
        .unwrap();
    assert!(keyboard.diagnostics.routed);
    assert_eq!(keyboard.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        keyboard.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(keyboard.diagnostics.notes, vec!["focused_route_len=2"]);

    let text = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "commit".to_string(),
            }),
        )
        .unwrap();
    assert_eq!(text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(text.diagnostics.route_policy, UiInputRoutePolicy::FocusPath);

    let ime = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Cancel,
                text: String::new(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();
    assert_eq!(ime.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, None);
    assert!(ime
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "ime owner cleared"));

    let scroll = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata: input_metadata(),
                event: UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                    .with_scroll_delta(-3.5),
                precise_scroll: Some(UiPreciseScrollDelta::pixels(2.25, -3.5)),
            }),
        )
        .unwrap();
    assert!(scroll
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "scroll_delta=-3.5"));
    assert_eq!(scroll.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    let UiInputEvent::Pointer(pointer) = scroll.event else {
        panic!("scroll dispatch changed event family");
    };
    assert_eq!(
        pointer.precise_scroll,
        Some(UiPreciseScrollDelta::pixels(2.25, -3.5))
    );

    let mut touch_metadata = input_metadata();
    touch_metadata.pointer_id = Some(UiPointerId::new(44));
    touch_metadata.pointer_source = UiPointerSource::Touch;
    let touch_move = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata: touch_metadata.clone(),
                event: UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(24.0, 24.0)),
                precise_scroll: None,
            }),
        )
        .unwrap();
    assert_eq!(
        touch_move.diagnostics.route_policy,
        UiInputRoutePolicy::Direct
    );
    assert!(touch_move
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "touch_like_pointer"));

    if let Some(pointer_id) = touch_metadata.pointer_id {
        capture_pointer_for_test(&mut surface, pointer_id, UiNodeId::new(2));
    }
    let captured_touch_move = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata: touch_metadata,
                event: UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(26.0, 26.0)),
                precise_scroll: None,
            }),
        )
        .unwrap();
    assert_eq!(
        captured_touch_move.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(99));
    let stale_text = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "ignored".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(stale_text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(stale_text.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, None);
}

#[test]
fn shared_text_input_mutates_focused_editable_value_and_marks_text_dirty() {
    let mut surface = editable_text_surface("Hi", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    let result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "!".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(editable_attr_string(&surface, "value"), "Hi!");
    assert_eq!(editable_attr_usize(&surface, "caret_offset"), 3);
    let node = surface.tree.nodes.get(&UiNodeId::new(2)).unwrap();
    assert!(node.dirty.layout);
    assert!(node.dirty.render);
    assert!(node.dirty.text);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("text_property_changed:value:")));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Hi!".to_string()),
        }
    );
}

#[test]
fn shared_ime_preedit_commit_and_cancel_mutate_editable_composition() {
    let mut surface = editable_text_surface("", 0);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();
    let preedit = "拼";

    let preedit_result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Preedit,
                text: preedit.to_string(),
                cursor_range: Some(
                    zircon_runtime_interface::ui::dispatch::UiTextByteRange::new(
                        preedit.len() as u32,
                        preedit.len() as u32,
                    ),
                ),
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();

    assert_eq!(
        preedit_result.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(editable_attr_string(&surface, "value"), preedit);
    assert_eq!(editable_attr_usize(&surface, "composition_start"), 0);
    assert_eq!(
        editable_attr_usize(&surface, "composition_end"),
        preedit.len()
    );
    assert_eq!(editable_attr_string(&surface, "composition_text"), preedit);
    assert_eq!(
        editable_attr_string(&surface, "composition_restore_text"),
        ""
    );
    assert_eq!(editable_attr_usize(&surface, "caret_offset"), preedit.len());

    let cancel_result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Cancel,
                text: String::new(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();

    assert_eq!(
        cancel_result.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(editable_attr_string(&surface, "value"), "");
    assert_eq!(editable_attr_string(&surface, "composition_text"), "");
    assert_eq!(surface.input.input_method_owner, None);

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Preedit,
                text: preedit.to_string(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();
    let commit_result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Commit,
                text: preedit.to_string(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();

    assert_eq!(
        commit_result.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(editable_attr_string(&surface, "value"), preedit);
    assert_eq!(editable_attr_string(&surface, "composition_text"), "");
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(commit_result.component_events.iter().any(|event| {
        event.event
            == UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String(preedit.to_string()),
            }
    }));

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let delete_surrounding = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::DeleteSurrounding,
                text: String::new(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: Some(UiImeDeleteSurrounding::new(1, 1)),
            }),
        )
        .unwrap();
    assert_eq!(
        delete_surrounding.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(
        delete_surrounding.diagnostics.route_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(editable_attr_string(&surface, "value"), "draf");
    assert!(!delete_surrounding
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "ime delete-surrounding is not applied by editable text yet"));
}

#[test]
fn shared_input_dispatch_rejects_invalid_owners_and_hidden_ancestors() {
    let mut surface = two_button_surface(None, None);
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    surface.input.input_method_owner = Some(UiNodeId::new(99));
    let missing_ime_owner = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Preedit,
                text: "draft".to_string(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();

    assert_eq!(
        missing_ime_owner.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(missing_ime_owner.diagnostics.route_target, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert!(missing_ime_owner
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "owner route rejected"));

    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .visibility = UiVisibility::Collapsed;
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let hidden_ancestor_text = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "ignored".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(
        hidden_ancestor_text.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(hidden_ancestor_text.diagnostics.route_target, None);
    assert_eq!(surface.input.input_method_owner, None);
}
