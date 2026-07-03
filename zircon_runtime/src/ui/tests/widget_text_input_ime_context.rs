use crate::core::framework::input::{ImeCursorArea, ImeHostRequest, ImeSurroundingText};
use crate::ui::{
    dispatch::{UiInputManager, UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    text::{
        caret_frame_for_text_layout, resolve_text_layout, text_range_frames_for_text_layout,
        UiTextLayoutRequest,
    },
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
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiTextCaret, UiTextCaretAffinity, UiTextRange,
        UiTextWrap, UiTextWritingMode,
    },
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
fn text_input_ime_preedit_rects_follow_soft_wrapped_composition_range() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "abcdef",
        5,
        1,
        5,
        [
            ("layout_padding_left", toml::Value::Float(0.0)),
            ("layout_padding_right", toml::Value::Float(136.0)),
            ("layout_padding_top", toml::Value::Float(0.0)),
            ("layout_padding_bottom", toml::Value::Float(0.0)),
            ("font_size", toml::Value::Float(10.0)),
            ("line_height", toml::Value::Float(12.0)),
            ("wrap", toml::Value::String("glyph".to_string())),
        ],
    );
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "WXYZQ", None);

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text_attr(&surface, "content"), "aWXYZQf");
    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    let layout = expected_text_layout_for_node(&surface, "aWXYZQf");
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(
        layout.lines[0].source_range,
        UiTextRange { start: 0, end: 3 }
    );
    assert_eq!(
        layout.lines[1].source_range,
        UiTextRange { start: 3, end: 7 }
    );
    let caret = UiTextCaret {
        offset: 6,
        affinity: UiTextCaretAffinity::Downstream,
    };
    assert_eq!(
        request.cursor_rect,
        caret_frame_for_text_layout(&layout, &caret)
    );
    assert_eq!(
        request.composition_rects,
        text_range_frames_for_text_layout(&layout, UiTextRange { start: 1, end: 6 })
    );
    let first_composition_y = request
        .composition_rects
        .first()
        .expect("composition rect")
        .y;
    assert!(
        request
            .composition_rects
            .iter()
            .any(|rect| rect.y > first_composition_y),
        "composition rects should span the wrapped line break"
    );
}

#[test]
fn text_ime_cursor_area_anchors_at_composition_end() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "abcdef",
        5,
        1,
        5,
        [
            ("layout_padding_left", toml::Value::Float(0.0)),
            ("layout_padding_right", toml::Value::Float(136.0)),
            ("layout_padding_top", toml::Value::Float(0.0)),
            ("layout_padding_bottom", toml::Value::Float(0.0)),
            ("font_size", toml::Value::Float(10.0)),
            ("line_height", toml::Value::Float(12.0)),
            ("wrap", toml::Value::String("glyph".to_string())),
        ],
    );
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let mut manager = UiInputManager::default();

    let result = dispatch_ime_with_manager(
        &mut surface,
        &mut manager,
        UiImeInputEventKind::Preedit,
        "WXYZQ",
        None,
    );

    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    let layout = expected_text_layout_for_node(&surface, "aWXYZQf");
    let composition_end = UiTextCaret {
        offset: 6,
        affinity: UiTextCaretAffinity::Downstream,
    };
    let expected_cursor_rect =
        caret_frame_for_text_layout(&layout, &composition_end).expect("composition end caret rect");
    assert_eq!(request.cursor_rect, Some(expected_cursor_rect));

    let host_requests = manager.drain_ime_host_requests();
    assert_eq!(
        host_requests,
        vec![
            ImeHostRequest::SetCursorArea(ImeCursorArea::new(
                expected_cursor_rect.x,
                expected_cursor_rect.y,
                expected_cursor_rect.width,
                expected_cursor_rect.height,
            )),
            ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("abcdef", 5, 5)),
        ]
    );
    assert!(manager.drain_ime_host_requests().is_empty());
}

#[test]
fn text_input_ime_cursor_rect_uses_resolved_tab_advances() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "a\tb",
        2,
        2,
        2,
        [
            ("layout_padding_left", toml::Value::Float(0.0)),
            ("layout_padding_right", toml::Value::Float(0.0)),
            ("layout_padding_top", toml::Value::Float(0.0)),
            ("layout_padding_bottom", toml::Value::Float(0.0)),
            ("font_size", toml::Value::Float(10.0)),
            ("line_height", toml::Value::Float(12.0)),
            ("tab_size", toml::Value::Float(4.0)),
        ],
    );
    let expected = expected_caret_after_tab_frame(&surface);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "", None);

    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    assert_eq!(request.cursor_rect, Some(expected));
    assert_ne!(
        request.cursor_rect,
        Some(UiFrame::new(20.0, 8.0, 1.0, 12.0)),
        "IME cursor rect should consume resolved tab advance, not fixed character columns"
    );
}

#[test]
fn text_input_ime_cursor_rect_uses_vertical_rl_geometry() {
    let text = "縦書文";
    let caret_offset = "縦書".len();
    let mut surface = text_input_surface_with_selection_and_attributes(
        text,
        caret_offset,
        caret_offset,
        caret_offset,
        [
            ("layout_padding_left", toml::Value::Float(0.0)),
            ("layout_padding_right", toml::Value::Float(0.0)),
            ("layout_padding_top", toml::Value::Float(0.0)),
            ("layout_padding_bottom", toml::Value::Float(0.0)),
            ("font_size", toml::Value::Float(10.0)),
            ("line_height", toml::Value::Float(12.0)),
            ("wrap", toml::Value::String("word".to_string())),
            (
                "writing_mode",
                toml::Value::String("vertical-rl".to_string()),
            ),
        ],
    );
    let layout = actual_text_layout_for_node(&surface, text);
    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    let expected = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: caret_offset,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("vertical caret frame");
    assert_eq!(expected.width, layout.lines[0].frame.width);
    assert_eq!(expected.height, 1.0);
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let result = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "", None);

    let request = assert_input_method_request(&result, UiInputMethodRequestKind::UpdateCursor);
    assert_eq!(request.cursor_rect, Some(expected));
    assert_ne!(
        request.cursor_rect,
        Some(UiFrame::new(
            expected.x,
            expected.y,
            1.0,
            layout.lines[0].frame.height
        )),
        "vertical writing mode should report a horizontal caret bar, not a horizontal-text caret"
    );
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

fn expected_caret_after_tab_frame(surface: &UiSurface) -> UiFrame {
    let layout = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some("a\tb")
        })
        .and_then(|command| command.text_layout.as_ref())
        .expect("text field layout");
    let line = layout.lines.first().expect("text line");
    assert_eq!(line.text, "a\tb");
    assert_eq!(line.glyph_advances.len(), 3);
    let x = line.frame.x + line.glyph_advances.iter().take(2).sum::<f32>();
    UiFrame::new(x, line.frame.y, 1.0, line.frame.height)
}

fn actual_text_layout_for_node(surface: &UiSurface, text: &str) -> UiResolvedTextLayout {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| {
            command.node_id == UiNodeId::new(2) && command.text.as_deref() == Some(text)
        })
        .and_then(|command| command.text_layout.as_ref())
        .cloned()
        .expect("text field layout")
}

fn expected_text_layout_for_node(surface: &UiSurface, text: &str) -> UiResolvedTextLayout {
    let node = surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .expect("text node");
    let metadata = node.template_metadata.as_ref().expect("text metadata");
    let frame = node.layout_cache.frame;
    let left = number_attr(metadata, "layout_padding_left").unwrap_or(10.0);
    let right = number_attr(metadata, "layout_padding_right").unwrap_or(10.0);
    let top = number_attr(metadata, "layout_padding_top").unwrap_or(4.0);
    let bottom = number_attr(metadata, "layout_padding_bottom").unwrap_or(4.0);
    let text_frame = UiFrame::new(
        frame.x + left,
        frame.y + top,
        (frame.width - left - right).max(1.0),
        (frame.height - top - bottom).max(1.0),
    );
    let style = UiResolvedStyle {
        font_size: number_attr(metadata, "font_size").unwrap_or(11.0),
        line_height: number_attr(metadata, "line_height").unwrap_or(13.2),
        wrap: match metadata
            .attributes
            .get("wrap")
            .and_then(toml::Value::as_str)
            .unwrap_or("none")
        {
            wrap if wrap.eq_ignore_ascii_case("word") => UiTextWrap::Word,
            wrap if wrap.eq_ignore_ascii_case("word_smart")
                || wrap.eq_ignore_ascii_case("word-smart") =>
            {
                UiTextWrap::WordSmart
            }
            wrap if wrap.eq_ignore_ascii_case("glyph") => UiTextWrap::Glyph,
            _ => UiTextWrap::None,
        },
        text_writing_mode: match metadata
            .attributes
            .get("writing_mode")
            .and_then(toml::Value::as_str)
            .unwrap_or("horizontal-tb")
        {
            mode if mode.eq_ignore_ascii_case("vertical")
                || mode.eq_ignore_ascii_case("vertical_rl")
                || mode.eq_ignore_ascii_case("vertical-rl") =>
            {
                UiTextWritingMode::VerticalRl
            }
            _ => UiTextWritingMode::HorizontalTb,
        },
        ..UiResolvedStyle::default()
    };
    resolve_text_layout(&UiTextLayoutRequest::new(
        text,
        &style,
        text_frame,
        Some(text_frame),
    ))
    .layout
}

fn number_attr(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .attributes
        .get(key)
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
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
                delete_surrounding: None,
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
