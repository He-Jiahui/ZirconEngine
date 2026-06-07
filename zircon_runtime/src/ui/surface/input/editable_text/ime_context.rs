use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiImeInputEventKind, UiInputEvent, UiInputMethodRequest,
        UiInputMethodRequestKind, UiInputMethodSurroundingText,
    },
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiEditableTextState, UiTextRange},
    tree::UiTemplateNodeMetadata,
};

use super::super::super::surface::UiSurface;
use super::super::text_state::clamp_text_boundary;

const DEFAULT_PADDING_X: f32 = 10.0;
const DEFAULT_PADDING_Y: f32 = 4.0;
const DEFAULT_FONT_SIZE: f32 = 11.0;
const CARET_WIDTH: f32 = 1.0;

pub(super) fn input_method_update_for_text_state(
    surface: &UiSurface,
    event: &UiInputEvent,
    target: UiNodeId,
    state: &UiEditableTextState,
) -> Option<UiDispatchEffect> {
    (surface.input.input_method_owner == Some(target) && input_event_refreshes_context(event)).then(
        || UiDispatchEffect::RequestInputMethod {
            request: UiInputMethodRequest {
                kind: UiInputMethodRequestKind::UpdateCursor,
                owner: target,
                cursor_rect: cursor_rect_for_state(surface, target, state),
                composition_rects: composition_rects_for_state(surface, target, state),
                surrounding_text: surrounding_text_for_state(state),
            },
        },
    )
}

fn input_event_refreshes_context(event: &UiInputEvent) -> bool {
    match event {
        UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) => true,
        UiInputEvent::Ime(ime) => matches!(
            ime.kind,
            UiImeInputEventKind::Preedit | UiImeInputEventKind::Commit
        ),
        UiInputEvent::Pointer(_)
        | UiInputEvent::Navigation(_)
        | UiInputEvent::Analog(_)
        | UiInputEvent::MouseMotion(_)
        | UiInputEvent::DragDrop(_)
        | UiInputEvent::Popup(_)
        | UiInputEvent::TooltipTimer(_)
        | UiInputEvent::Accessibility(_) => false,
    }
}

fn cursor_rect_for_state(
    surface: &UiSurface,
    target: UiNodeId,
    state: &UiEditableTextState,
) -> Option<UiFrame> {
    let text_frame = text_frame_for_node(surface, target)?;
    let font_metrics = font_metrics_for_node(surface, target);
    let (line, column) = line_column_for_offset(&state.text, state.caret.offset);
    Some(UiFrame::new(
        text_frame.x + column as f32 * font_metrics.char_advance,
        text_frame.y + line as f32 * font_metrics.line_height,
        CARET_WIDTH,
        font_metrics
            .line_height
            .min(text_frame.height)
            .max(CARET_WIDTH),
    ))
}

fn composition_rects_for_state(
    surface: &UiSurface,
    target: UiNodeId,
    state: &UiEditableTextState,
) -> Vec<UiFrame> {
    let Some(composition) = state.composition.as_ref() else {
        return Vec::new();
    };
    let Some(text_frame) = text_frame_for_node(surface, target) else {
        return Vec::new();
    };
    let font_metrics = font_metrics_for_node(surface, target);
    vec![range_rect_for_text(
        &state.text,
        composition.range,
        text_frame,
        font_metrics,
    )]
}

fn surrounding_text_for_state(state: &UiEditableTextState) -> Option<UiInputMethodSurroundingText> {
    let (text, source_range, source_replacement_len) = committed_text_for_input_method(state);
    let cursor_byte = map_visible_offset_to_committed_offset(
        state.caret.offset,
        source_range,
        source_replacement_len,
    );
    let anchor = state
        .selection
        .as_ref()
        .map(|selection| selection.anchor)
        .unwrap_or(state.caret.offset);
    let anchor_byte =
        map_visible_offset_to_committed_offset(anchor, source_range, source_replacement_len);
    UiInputMethodSurroundingText::new(text, cursor_byte as u32, anchor_byte as u32).ok()
}

fn committed_text_for_input_method(state: &UiEditableTextState) -> (String, UiTextRange, usize) {
    let Some(composition) = state.composition.as_ref() else {
        return (state.text.clone(), UiTextRange::default(), 0);
    };
    let start = clamp_text_boundary(&state.text, composition.range.start);
    let end = clamp_text_boundary(&state.text, composition.range.end).max(start);
    let restore_text = composition.restore_text.clone().unwrap_or_default();
    let mut text = String::with_capacity(state.text.len() - (end - start) + restore_text.len());
    text.push_str(&state.text[..start]);
    text.push_str(&restore_text);
    text.push_str(&state.text[end..]);
    (text, UiTextRange { start, end }, restore_text.len())
}

fn map_visible_offset_to_committed_offset(
    offset: usize,
    source_range: UiTextRange,
    replacement_len: usize,
) -> usize {
    if source_range.start == source_range.end {
        return offset;
    }
    if offset <= source_range.start {
        offset
    } else if offset >= source_range.end {
        source_range.start + replacement_len + (offset - source_range.end)
    } else {
        source_range.start + replacement_len
    }
}

#[derive(Clone, Copy)]
struct FontMetrics {
    char_advance: f32,
    line_height: f32,
}

fn font_metrics_for_node(surface: &UiSurface, target: UiNodeId) -> FontMetrics {
    let metadata = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref());
    let font_size = number_attribute(metadata, "font_size").unwrap_or(DEFAULT_FONT_SIZE);
    FontMetrics {
        char_advance: (font_size * 0.6).max(CARET_WIDTH),
        line_height: number_attribute(metadata, "line_height")
            .unwrap_or(font_size * 1.2)
            .max(font_size),
    }
}

fn text_frame_for_node(surface: &UiSurface, target: UiNodeId) -> Option<UiFrame> {
    let node = surface.tree.nodes.get(&target)?;
    let metadata = node.template_metadata.as_ref();
    let frame = node.layout_cache.frame;
    let left = number_attribute(metadata, "layout_padding_left").unwrap_or(DEFAULT_PADDING_X);
    let right = number_attribute(metadata, "layout_padding_right").unwrap_or(DEFAULT_PADDING_X);
    let top = number_attribute(metadata, "layout_padding_top").unwrap_or(DEFAULT_PADDING_Y);
    let bottom = number_attribute(metadata, "layout_padding_bottom").unwrap_or(DEFAULT_PADDING_Y);
    Some(UiFrame::new(
        frame.x + left,
        frame.y + top,
        (frame.width - left - right).max(CARET_WIDTH),
        (frame.height - top - bottom).max(CARET_WIDTH),
    ))
}

fn range_rect_for_text(
    text: &str,
    range: UiTextRange,
    text_frame: UiFrame,
    font_metrics: FontMetrics,
) -> UiFrame {
    let start = clamp_text_boundary(text, range.start);
    let end = clamp_text_boundary(text, range.end).max(start);
    let (line, start_column) = line_column_for_offset(text, start);
    let (_, end_column) = line_column_for_offset(text, end);
    let x = text_frame.x + start_column as f32 * font_metrics.char_advance;
    let width = ((end_column.saturating_sub(start_column)) as f32 * font_metrics.char_advance)
        .max(CARET_WIDTH);
    UiFrame::new(
        x,
        text_frame.y + line as f32 * font_metrics.line_height,
        width,
        font_metrics
            .line_height
            .min(text_frame.height)
            .max(CARET_WIDTH),
    )
}

fn line_column_for_offset(text: &str, offset: usize) -> (usize, usize) {
    let offset = clamp_text_boundary(text, offset);
    let before = &text[..offset];
    let line = before
        .as_bytes()
        .iter()
        .filter(|byte| **byte == b'\n')
        .count();
    let column_text = before
        .rsplit_once('\n')
        .map(|(_, tail)| tail)
        .unwrap_or(before);
    (line, column_text.chars().count())
}

fn number_attribute(metadata: Option<&UiTemplateNodeMetadata>, key: &str) -> Option<f32> {
    metadata
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
}
