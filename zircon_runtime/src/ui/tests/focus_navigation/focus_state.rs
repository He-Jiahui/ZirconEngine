use super::*;

#[test]
fn autofocus_records_initial_focus_change_and_visible_reason() {
    let mut surface = focus_surface();

    let event = surface.resolve_autofocus().unwrap().expect("autofocus");

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert_eq!(surface.focus.pending_autofocus, None);
    assert_eq!(surface.focus.previous, None);
    assert_eq!(event.current, Some(id(2)));
    assert_eq!(event.reason, UiFocusChangeReason::Autofocus);
    assert!(!event.visible.visible);
    assert_eq!(event.visible.reason, UiFocusVisibleReason::Programmatic);
    assert_eq!(surface.focus.changes, vec![event]);
    assert!(surface.component_state(id(2)).unwrap().flags.focused);
    assert!(!surface.component_state(id(2)).unwrap().flags.focus_visible);
}

#[test]
fn pointer_and_navigation_focus_sources_update_visible_reason() {
    let mut surface = focus_surface();

    surface.focus_node(id(2)).unwrap();
    assert!(surface.component_state(id(2)).unwrap().flags.focused);
    assert!(!surface.component_state(id(2)).unwrap().flags.focus_visible);
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(3)));
    assert!(!surface.component_state(id(2)).unwrap().flags.focused);
    assert!(!surface.component_state(id(2)).unwrap().flags.focus_visible);
    assert!(surface.component_state(id(3)).unwrap().flags.focused);
    assert!(surface.component_state(id(3)).unwrap().flags.focus_visible);
    assert!(surface.focus.focus_visible.visible);
    assert_eq!(
        surface.focus.focus_visible.reason,
        UiFocusVisibleReason::KeyboardNavigation
    );

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: input_metadata(),
                state: UiKeyboardInputState::Pressed,
                key_code: 65,
                scan_code: Some(30),
                physical_key: "KeyA".to_string(),
                logical_key: "A".to_string(),
                text: Some("a".to_string()),
            }),
        )
        .unwrap();

    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, id(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(surface.focus.focused_inputs[0].route, vec![id(3), id(1)]);
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn focus_component_state_changes_mark_render_only_dirty() {
    let mut surface = focus_surface();
    surface.clear_dirty_flags();

    surface.focus_node(id(2)).unwrap();
    assert_render_only_dirty(surface.dirty_flags());
    surface.clear_dirty_flags();

    surface.focus_node(id(3)).unwrap();
    assert_render_only_dirty(surface.dirty_flags());
    assert_eq!(
        surface
            .tree
            .nodes
            .values()
            .filter(|node| node.dirty.render)
            .count(),
        2
    );
    surface.clear_dirty_flags();

    surface.clear_focus();
    assert_render_only_dirty(surface.dirty_flags());
}

#[test]
fn text_and_ime_inputs_record_focused_input_routes() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();
    surface.input.input_method_owner = Some(id(2));

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Text(zircon_runtime_interface::ui::dispatch::UiTextInputEvent {
                metadata: input_metadata(),
                text: "x".to_string(),
            }),
        )
        .unwrap();
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(zircon_runtime_interface::ui::dispatch::UiImeInputEvent {
                metadata: input_metadata(),
                kind: zircon_runtime_interface::ui::dispatch::UiImeInputEventKind::Cancel,
                text: String::new(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap();

    assert_eq!(surface.focus.focused_inputs.len(), 2);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Text
    );
    assert_eq!(
        surface.focus.focused_inputs[1].kind,
        UiFocusedInputKind::Ime
    );
    assert_eq!(surface.focus.focused_inputs[0].route, vec![id(2), id(1)]);
    assert_eq!(surface.focus.focused_inputs[1].route, vec![id(2), id(1)]);
    assert!(surface.focus.focused_inputs[0].accepted);
    assert!(surface.focus.focused_inputs[1].accepted);
}
