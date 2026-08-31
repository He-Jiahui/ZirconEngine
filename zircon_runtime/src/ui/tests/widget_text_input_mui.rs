use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingSourceKind, UiBindingUpdateReport, UiBindingUpdateStatus, UiEventKind},
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiDispatchDisposition, UiImeInputEvent, UiImeInputEventKind, UiInputEvent,
        UiInputEventMetadata, UiInputModifiers, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiTextByteRange, UiTextInputEvent,
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
fn editor_text_component_aliases_use_their_canonical_surface_value_property() {
    for (component, property) in [("FieldEditor", "value_text"), ("SourceEditor", "text")] {
        let mut surface = mui_text_input_surface(component, "a", 1, []);
        surface.focus_node(UiNodeId::new(2)).unwrap();

        let result = dispatch_text(&mut surface, "b");

        assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
        assert_eq!(
            result.diagnostics.handled_phase.as_deref(),
            Some("text.edit")
        );
        assert_eq!(text_attr(&surface, property), "ab");
        assert_eq!(int_attr(&surface, "caret_offset"), 2);
        assert_eq!(result.component_events.len(), 1);
        assert_eq!(
            result.component_events[0].event,
            UiComponentEvent::ValueChanged {
                property: property.to_string(),
                value: UiValue::String("ab".to_string()),
            }
        );
        assert_widget_binding_report_for_property(&result.binding_reports, property);
    }
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
fn mui_text_field_keyboard_delete_and_caret_keys_update_retained_edit_state() {
    let mut surface = mui_text_input_surface("TextField", "abcd", 2, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let arrow = dispatch_key(&mut surface, "ArrowLeft", 37);

    assert_eq!(arrow.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        arrow.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(value_attr(&surface), "abcd");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert!(arrow.component_events.is_empty());

    let delete = dispatch_key(&mut surface, "Delete", 46);

    assert_eq!(delete.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        delete.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(value_attr(&surface), "acd");
    assert_eq!(int_attr(&surface, "caret_offset"), 1);
    assert_eq!(delete.component_events.len(), 1);
    assert_eq!(
        delete.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("acd".to_string()),
        }
    );
    assert_widget_binding_report(&delete.binding_reports);

    let end = dispatch_key(&mut surface, "End", 35);

    assert_eq!(end.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        end.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(value_attr(&surface), "acd");
    assert_eq!(int_attr(&surface, "caret_offset"), 3);
    assert!(end.component_events.is_empty());
}

#[test]
fn mui_text_field_keyboard_backspace_removes_active_selection_and_collapses_caret() {
    let mut surface = mui_text_input_surface(
        "TextField",
        "alpha beta",
        10,
        [
            ("selection_anchor", toml::Value::Integer(6)),
            ("selection_focus", toml::Value::Integer(10)),
        ],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(value_attr(&surface), "alpha ");
    assert_eq!(int_attr(&surface, "caret_offset"), 6);
    assert_eq!(int_attr(&surface, "selection_anchor"), 6);
    assert_eq!(int_attr(&surface, "selection_focus"), 6);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("alpha ".to_string()),
        }
    );
    assert_widget_binding_report(&result.binding_reports);
}

#[test]
fn mui_text_field_keyboard_word_delete_and_select_all_replace_use_retained_state() {
    let mut surface = mui_text_input_surface("TextField", "alpha beta gamma", 16, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let delete_word = dispatch_key_with_modifiers(
        &mut surface,
        "Backspace",
        8,
        UiInputModifiers {
            control: true,
            ..UiInputModifiers::default()
        },
    );

    assert_eq!(
        delete_word.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(value_attr(&surface), "alpha beta ");
    assert_eq!(int_attr(&surface, "caret_offset"), 11);
    assert_eq!(int_attr(&surface, "selection_anchor"), 11);
    assert_eq!(int_attr(&surface, "selection_focus"), 11);
    assert_eq!(delete_word.component_events.len(), 1);

    let select_all = dispatch_key_with_modifiers(
        &mut surface,
        "a",
        65,
        UiInputModifiers {
            control: true,
            ..UiInputModifiers::default()
        },
    );

    assert_eq!(select_all.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "alpha beta ");
    assert_eq!(int_attr(&surface, "caret_offset"), 11);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 11);

    let replace = dispatch_text(&mut surface, "Ω");

    assert_eq!(replace.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "Ω");
    assert_eq!(int_attr(&surface, "caret_offset"), "Ω".len() as i64);
    assert_eq!(int_attr(&surface, "selection_anchor"), "Ω".len() as i64);
    assert_eq!(int_attr(&surface, "selection_focus"), "Ω".len() as i64);
    assert_eq!(replace.component_events.len(), 1);
}

#[test]
fn mui_text_field_keyboard_shift_extends_selection_and_delete_collapses_it() {
    let mut surface = mui_text_input_surface("TextField", "alpha beta", 5, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let shift_left = dispatch_key_with_modifiers(
        &mut surface,
        "ArrowLeft",
        37,
        UiInputModifiers {
            shift: true,
            ..UiInputModifiers::default()
        },
    );

    assert_eq!(shift_left.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "alpha beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 4);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 4);

    let ctrl_shift_left = dispatch_key_with_modifiers(
        &mut surface,
        "ArrowLeft",
        37,
        UiInputModifiers {
            shift: true,
            control: true,
            ..UiInputModifiers::default()
        },
    );

    assert_eq!(
        ctrl_shift_left.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 5);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);

    let delete = dispatch_key(&mut surface, "Delete", 46);

    assert_eq!(delete.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), " beta");
    assert_eq!(int_attr(&surface, "caret_offset"), 0);
    assert_eq!(int_attr(&surface, "selection_anchor"), 0);
    assert_eq!(int_attr(&surface, "selection_focus"), 0);
    assert_eq!(delete.component_events.len(), 1);
}

#[test]
fn mui_textarea_keyboard_multiline_navigation_and_enter_update_retained_state() {
    let mut surface = mui_text_input_surface("TextareaAutosize", "one\nalphabet\nxy", 8, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let down = dispatch_key(&mut surface, "ArrowDown", 40);

    assert_eq!(down.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "one\nalphabet\nxy");
    assert_eq!(int_attr(&surface, "caret_offset"), 15);

    let up = dispatch_key(&mut surface, "ArrowUp", 38);

    assert_eq!(up.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(int_attr(&surface, "caret_offset"), 6);

    let home = dispatch_key(&mut surface, "Home", 36);

    assert_eq!(home.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(int_attr(&surface, "caret_offset"), 4);

    let end = dispatch_key(&mut surface, "End", 35);

    assert_eq!(end.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(int_attr(&surface, "caret_offset"), 12);

    let enter = dispatch_key(&mut surface, "Enter", 13);

    assert_eq!(enter.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "one\nalphabet\n\nxy");
    assert_eq!(int_attr(&surface, "caret_offset"), 13);
    assert_eq!(enter.component_events.len(), 1);
}

#[test]
fn mui_text_field_ime_preedit_commit_and_cancel_use_retained_composition_state() {
    let mut surface = mui_text_input_surface(
        "TextField",
        "abcd",
        3,
        [
            ("selection_anchor", toml::Value::Integer(1)),
            ("selection_focus", toml::Value::Integer(3)),
        ],
    );
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let preedit_text = "拼";

    let preedit = dispatch_ime(
        &mut surface,
        UiImeInputEventKind::Preedit,
        preedit_text,
        Some(UiTextByteRange::new(
            preedit_text.len() as u32,
            preedit_text.len() as u32,
        )),
    );

    assert_eq!(preedit.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        preedit.diagnostics.handled_phase.as_deref(),
        Some("ime.edit")
    );
    assert_eq!(value_attr(&surface), "a拼d");
    assert_eq!(int_attr(&surface, "composition_start"), 1);
    assert_eq!(
        int_attr(&surface, "composition_end"),
        (1 + preedit_text.len()) as i64
    );
    assert_eq!(text_attr(&surface, "composition_text"), preedit_text);
    assert_eq!(text_attr(&surface, "composition_restore_text"), "bc");
    assert_eq!(
        int_attr(&surface, "caret_offset"),
        (1 + preedit_text.len()) as i64
    );

    let commit = dispatch_ime(&mut surface, UiImeInputEventKind::Commit, "你", None);

    assert_eq!(commit.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "a你d");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    assert_eq!(text_attr(&surface, "composition_restore_text"), "");
    assert_eq!(int_attr(&surface, "caret_offset"), (1 + "你".len()) as i64);
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "临", None);
    let cancel = dispatch_ime(&mut surface, UiImeInputEventKind::Cancel, "", None);

    assert_eq!(cancel.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(value_attr(&surface), "a你d");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    assert_eq!(text_attr(&surface, "composition_restore_text"), "");
    assert_eq!(surface.input.input_method_owner, None);
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

#[test]
fn mui_search_field_keyboard_backspace_uses_query_property_without_widget_override() {
    let mut surface = mui_search_field_surface("needle", 6, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(text_attr(&surface, "query"), "needl");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "query".to_string(),
            value: UiValue::String("needl".to_string()),
        }
    );
    assert_widget_binding_report_for_property(&result.binding_reports, "query");
}

#[test]
fn mui_autocomplete_keyboard_backspace_uses_query_without_overwriting_value() {
    let mut surface = mui_autocomplete_surface("needle", "asset://selected", 6, []);
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_key(&mut surface, "Backspace", 8);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.text_edit")
    );
    assert_eq!(text_attr(&surface, "query"), "needl");
    assert_eq!(text_attr(&surface, "value"), "asset://selected");
    assert_eq!(int_attr(&surface, "caret_offset"), 5);
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "query".to_string(),
            value: UiValue::String("needl".to_string()),
        }
    );
    assert_widget_binding_report_for_property(&result.binding_reports, "query");
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
                    UiInputSequence::new(16),
                ),
                kind,
                text: text.to_string(),
                cursor_range,
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap()
}

fn dispatch_key(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    dispatch_key_with_modifiers(surface, logical_key, key_code, UiInputModifiers::default())
}

fn dispatch_key_with_modifiers(
    surface: &mut UiSurface,
    logical_key: &str,
    key_code: u32,
    modifiers: UiInputModifiers,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(42), UiInputSequence::new(15));
    metadata.modifiers = modifiers;
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
    let value_property = match component {
        "FieldEditor" => "value_text",
        "SourceEditor" => "text",
        _ => "value",
    };
    let attributes = [
        (
            value_property.to_string(),
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
    text_attr(surface, "value")
}

fn mui_search_field_surface(
    query: &str,
    caret_offset: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.mui.search"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    let attributes = [
        ("query".to_string(), toml::Value::String(query.to_string())),
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/mui_search"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "SearchField".to_string(),
                    attributes,
                    bindings: vec![binding("MuiSearchField/Change", UiEventKind::Change)],
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn mui_autocomplete_surface(
    query: &str,
    value: &str,
    caret_offset: usize,
    extra_attributes: impl IntoIterator<Item = (&'static str, toml::Value)>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.widget.text_input.mui.autocomplete",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
    let attributes = [
        ("query".to_string(), toml::Value::String(query.to_string())),
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
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/mui_autocomplete"))
                .with_frame(UiFrame::new(8.0, 8.0, 160.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Autocomplete".to_string(),
                    attributes,
                    bindings: vec![binding("MuiAutocomplete/Change", UiEventKind::Change)],
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

fn assert_widget_binding_report(reports: &[UiBindingUpdateReport]) {
    assert_widget_binding_report_for_property(reports, "value");
}

fn assert_widget_binding_report_for_property(reports: &[UiBindingUpdateReport], property: &str) {
    let value_updates: Vec<_> = reports
        .iter()
        .flat_map(|report| report.updates.iter())
        .filter(|update| {
            update.source.kind == UiBindingSourceKind::WidgetBehavior
                && update.source.node_id == Some(UiNodeId::new(2))
                && update.source.property.as_deref() == Some(property)
        })
        .collect();
    assert_eq!(value_updates.len(), 1, "{reports:#?}");
    let update = value_updates[0];
    assert_eq!(update.source.kind, UiBindingSourceKind::WidgetBehavior);
    assert_eq!(update.source.node_id, Some(UiNodeId::new(2)));
    assert_eq!(update.source.property.as_deref(), Some(property));
    assert_eq!(update.target.node_id, Some(UiNodeId::new(2)));
    assert_eq!(update.target.property.as_deref(), Some(property));
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
