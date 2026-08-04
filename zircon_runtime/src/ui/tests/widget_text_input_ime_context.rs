use crate::ui::{
    dispatch::{UiInputManager, UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchReply,
        UiFocusEffectReason, UiImeDeleteSurrounding, UiImeInputEvent, UiImeInputEventKind,
        UiImePreeditClause, UiImePreeditClauseKind, UiInputEvent, UiInputEventMetadata,
        UiInputMethodRequest, UiInputMethodRequestKind, UiInputSequence, UiInputTimestamp,
        UiTextByteRange,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiWidgetBehavior, UiWidgetContract},
};

mod focus_lifecycle;
mod geometry;

#[test]
fn text_input_ime_rejects_invalid_preedit_clauses_without_mutating_text() {
    let mut surface = text_input_surface_with_selection("abc", 1, 1, 1);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(42),
                    UiInputSequence::new(8),
                ),
                kind: UiImeInputEventKind::Preedit,
                text: "x".to_string(),
                cursor_range: None,
                preedit_clauses: vec![UiImePreeditClause::new(
                    UiTextByteRange::new(0, 2),
                    UiImePreeditClauseKind::Input,
                )],
                delete_surrounding: None,
            }),
        )
        .unwrap();

    assert_eq!(text_attr(&surface, "content"), "abc");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    assert!(result.host_requests.is_empty());
}

#[test]
fn text_input_ime_preedit_preserves_validated_clauses_through_render_extract() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let clauses = vec![
        UiImePreeditClause::new(UiTextByteRange::new(0, 1), UiImePreeditClauseKind::Input),
        UiImePreeditClause::new(
            UiTextByteRange::new(1, 2),
            UiImePreeditClauseKind::TargetConverted,
        ),
    ];

    let result = dispatch_ime_preedit_with_clauses(&mut surface, "XY", clauses.clone());

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    let command = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("aXYd")
        })
        .expect("updated text field render command");
    assert_eq!(
        command
            .text_layout
            .as_ref()
            .and_then(|layout| layout.editable.as_ref())
            .and_then(|editable| editable.composition.as_ref())
            .map(|composition| &composition.preedit_clauses),
        Some(&clauses)
    );
    let attribute = surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("composition_clauses"))
        .and_then(toml::Value::as_array)
        .expect("structured composition clauses attribute");
    assert_eq!(attribute.len(), 2);
    assert_eq!(
        attribute[1].get("kind").and_then(toml::Value::as_str),
        Some("target_converted")
    );
}

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
fn text_input_ime_surrounding_text_is_bounded_to_graphemes_around_the_caret() {
    let grapheme = "e\u{301}";
    let value = grapheme.repeat(600);
    let caret_offset = grapheme.len() * 300;
    let mut surface =
        text_input_surface_with_selection(value.as_str(), caret_offset, caret_offset, caret_offset);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "", None);

    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    let surrounding = request
        .surrounding_text
        .as_ref()
        .expect("bounded surrounding text");
    assert_eq!(surrounding.text.graphemes(true).count(), 512);
    assert_eq!(surrounding.text, grapheme.repeat(512));
    assert_eq!(surrounding.cursor_byte as usize, grapheme.len() * 256);
    assert_eq!(surrounding.anchor_byte, surrounding.cursor_byte);
}

#[test]
fn text_input_ime_surrounding_text_trims_a_wide_grapheme_window_to_the_byte_limit() {
    let grapheme = "\u{1f469}\u{200d}\u{1f4bb}";
    let value = grapheme.repeat(600);
    let caret_offset = grapheme.len() * 300;
    let mut surface =
        text_input_surface_with_selection(value.as_str(), caret_offset, caret_offset, caret_offset);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "", None);

    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    let surrounding = request
        .surrounding_text
        .as_ref()
        .expect("byte-limited surrounding text");
    assert!(surrounding.text.len() < 4_000);
    assert!(surrounding.text.graphemes(true).count() < 512);
    assert!(surrounding
        .text
        .is_char_boundary(surrounding.cursor_byte as usize));
    assert_eq!(surrounding.anchor_byte, surrounding.cursor_byte);
}

#[test]
fn text_input_ime_surrounding_text_omits_a_single_grapheme_that_exceeds_the_contract_limit() {
    let oversized_grapheme = format!("a{}", "\u{301}".repeat(4_000));
    let caret_offset = oversized_grapheme.len();
    let value = format!("{oversized_grapheme}z");
    let mut surface =
        text_input_surface_with_selection(value.as_str(), caret_offset, caret_offset, caret_offset);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "", None);

    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    assert!(
        request.surrounding_text.is_none(),
        "an oversized single grapheme must not bypass the surrounding-text byte limit"
    );
}

#[test]
fn text_input_ime_delete_surrounding_expands_partial_byte_ranges_to_complete_graphemes() {
    let emoji = "\u{1f469}\u{200d}\u{1f4bb}";
    let value = format!("a{emoji}b");
    let caret_offset = "a".len() + emoji.len();
    let mut surface =
        text_input_surface_with_selection(value.as_str(), caret_offset, caret_offset, caret_offset);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime_delete_surrounding(&mut surface, 1, 1);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "a");
    assert_eq!(usize_attr(&surface, "caret_offset"), Some(1));
}

#[test]
fn text_input_ime_delete_surrounding_expands_partial_after_bytes_to_complete_graphemes() {
    let mut surface = text_input_surface_with_selection("aé", 1, 1, 1);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime_delete_surrounding(&mut surface, 0, 1);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "a");
    assert_eq!(usize_attr(&surface, "caret_offset"), Some(1));
}

#[test]
fn text_input_ime_delete_surrounding_refreshes_the_host_context_after_mutation() {
    let mut surface = text_input_surface_with_selection("abc", 1, 1, 1);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime_delete_surrounding(&mut surface, 1, 0);

    assert_eq!(text_attr(&surface, "content"), "bc");
    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    let surrounding = request
        .surrounding_text
        .as_ref()
        .expect("updated committed surrounding text");
    assert_eq!(surrounding.text, "bc");
    assert_eq!(surrounding.cursor_byte, 0);
    assert_eq!(surrounding.anchor_byte, 0);
}

#[test]
fn text_input_ime_delete_surrounding_restores_committed_text_before_deleting() {
    let mut surface = text_input_surface_with_selection("abc", 2, 1, 2);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "X", None);

    let result = dispatch_ime_delete_surrounding(&mut surface, 1, 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "ac");
    assert_eq!(text_attr(&surface, "composition_text"), "");
}

#[test]
fn text_input_ime_delete_surrounding_zero_payload_preserves_composition() {
    let mut surface = text_input_surface_with_selection("abc", 2, 1, 2);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "X", None);

    let result = dispatch_ime_delete_surrounding(&mut surface, 0, 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "aXc");
    assert_eq!(text_attr(&surface, "composition_text"), "X");
    assert!(result.host_requests.is_empty());
}

#[test]
fn text_input_ime_delete_surrounding_at_the_committed_start_preserves_composition() {
    let mut surface = text_input_surface_with_selection("abc", 0, 0, 0);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "X", None);

    let result = dispatch_ime_delete_surrounding(&mut surface, 1, 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "Xabc");
    assert_eq!(text_attr(&surface, "composition_text"), "X");
    assert!(result.host_requests.is_empty());
}

#[test]
fn text_input_ime_delete_surrounding_without_payload_preserves_composition() {
    let mut surface = text_input_surface_with_selection("abc", 2, 1, 2);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "X", None);

    let result = dispatch_ime(
        &mut surface,
        UiImeInputEventKind::DeleteSurrounding,
        "",
        None,
    );

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "aXc");
    assert_eq!(text_attr(&surface, "composition_text"), "X");
    assert!(result.host_requests.is_empty());
}

#[test]
fn text_input_ime_delete_surrounding_uses_the_reported_committed_caret_for_preedit() {
    let mut surface = text_input_surface_with_selection("abcd", 3, 1, 3);
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let preedit = dispatch_ime(
        &mut surface,
        UiImeInputEventKind::Preedit,
        "XY",
        Some(UiTextByteRange::new(1, 1)),
    );
    let request = assert_input_method_request(&preedit, UiInputMethodRequestKind::UpdateCursor);
    assert_eq!(
        request
            .surrounding_text
            .as_ref()
            .expect("committed surrounding text")
            .cursor_byte,
        3
    );

    let result = dispatch_ime_delete_surrounding(&mut surface, 1, 0);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "abd");
    assert_eq!(text_attr(&surface, "composition_text"), "");
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
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            }),
        )
        .unwrap()
}

fn dispatch_ime_preedit_with_clauses(
    surface: &mut UiSurface,
    text: &str,
    preedit_clauses: Vec<UiImePreeditClause>,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(45),
                    UiInputSequence::new(11),
                ),
                kind: UiImeInputEventKind::Preedit,
                text: text.to_string(),
                cursor_range: None,
                preedit_clauses,
                delete_surrounding: None,
            }),
        )
        .unwrap()
}

fn dispatch_ime_delete_surrounding(
    surface: &mut UiSurface,
    before_bytes: u32,
    after_bytes: u32,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(44),
                    UiInputSequence::new(10),
                ),
                kind: UiImeInputEventKind::DeleteSurrounding,
                text: String::new(),
                cursor_range: None,
                preedit_clauses: Vec::new(),
                delete_surrounding: Some(UiImeDeleteSurrounding::new(before_bytes, after_bytes)),
            }),
        )
        .unwrap()
}

fn dispatch_ime_with_manager(
    surface: &mut UiSurface,
    manager: &mut UiInputManager,
    kind: UiImeInputEventKind,
    text: &str,
    cursor_range: Option<UiTextByteRange>,
) -> zircon_runtime_interface::ui::dispatch::UiInputDispatchResult {
    surface
        .dispatch_input_event_with_manager(
            manager,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: UiInputEventMetadata::new(
                    UiInputTimestamp::from_micros(43),
                    UiInputSequence::new(9),
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

fn text_input_surface_with_selection_and_attributes<const N: usize>(
    value: &str,
    caret_offset: usize,
    selection_anchor: usize,
    selection_focus: usize,
    attributes: [(&str, toml::Value); N],
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.widget.text_input.ime_context"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 80.0)),
    );
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

fn usize_attr(surface: &UiSurface, key: &str) -> Option<usize> {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(toml::Value::as_integer)
        .and_then(|value| (value >= 0).then_some(value as usize))
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
