use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiImeInputEventKind, UiInputEvent, UiInputMethodRequest,
        UiInputMethodRequestKind, UiInputMethodSurroundingText,
    },
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiEditableTextState, UiRenderCommand, UiResolvedStyle, UiResolvedTextLayout, UiTextRange,
    },
    tree::UiTemplateNodeMetadata,
};

use super::super::super::surface::UiSurface;
use super::super::text_state::clamp_text_boundary;
use crate::ui::text::{
    caret_frame_for_text_layout_with_source_metrics,
    text_range_frames_for_text_layout_with_source_metrics,
};

const DEFAULT_PADDING_X: f32 = 10.0;
const DEFAULT_PADDING_Y: f32 = 4.0;
const DEFAULT_FONT_SIZE: f32 = 11.0;
const CARET_WIDTH: f32 = 1.0;

struct InputMethodTextLayout<'a> {
    layout: &'a UiResolvedTextLayout,
    style: &'a UiResolvedStyle,
}

pub(super) fn input_method_update_for_text_state(
    surface: &mut UiSurface,
    event: &UiInputEvent,
    target: UiNodeId,
    state: &UiEditableTextState,
) -> Option<UiDispatchEffect> {
    if surface.input.input_method_owner != Some(target) || !input_event_refreshes_context(event) {
        return None;
    }

    surface.refresh_render_extract_for_current_tree();
    let text_layout = resolved_text_layout_for_state(surface, target, state);
    Some(UiDispatchEffect::RequestInputMethod {
        request: UiInputMethodRequest {
            kind: UiInputMethodRequestKind::UpdateCursor,
            owner: target,
            cursor_rect: cursor_rect_for_state(surface, target, state, text_layout.as_ref()),
            composition_rects: composition_rects_for_state(
                surface,
                target,
                state,
                text_layout.as_ref(),
            ),
            surrounding_text: surrounding_text_for_state(state),
        },
    })
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
        | UiInputEvent::TypeaheadTimer(_)
        | UiInputEvent::SubmenuHoverTimer(_)
        | UiInputEvent::ToastTimer(_)
        | UiInputEvent::Accessibility(_) => false,
    }
}

fn cursor_rect_for_state(
    surface: &UiSurface,
    target: UiNodeId,
    state: &UiEditableTextState,
    text_layout: Option<&InputMethodTextLayout<'_>>,
) -> Option<UiFrame> {
    if let Some(frame) = text_layout.and_then(|text_layout| {
        caret_frame_for_text_layout_with_source_metrics(
            text_layout.layout,
            &state.caret,
            state.text.as_str(),
            text_layout.style,
        )
    }) {
        return Some(frame);
    }

    let text_frame = text_frame_for_node(surface, target)?;
    let font_metrics = font_metrics_for_node(surface, target);
    let wrap_columns = wrap_columns_for_node(surface, target, text_frame, font_metrics);
    let (line, column) =
        visual_line_column_for_offset(&state.text, state.caret.offset, wrap_columns);
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
    text_layout: Option<&InputMethodTextLayout<'_>>,
) -> Vec<UiFrame> {
    let Some(composition) = state.composition.as_ref() else {
        return Vec::new();
    };
    if let Some(text_layout) = text_layout {
        return text_range_frames_for_text_layout_with_source_metrics(
            text_layout.layout,
            composition.range,
            state.text.as_str(),
            text_layout.style,
        );
    }

    let Some(text_frame) = text_frame_for_node(surface, target) else {
        return Vec::new();
    };
    let font_metrics = font_metrics_for_node(surface, target);
    let wrap_columns = wrap_columns_for_node(surface, target, text_frame, font_metrics);
    range_rects_for_text(
        &state.text,
        composition.range,
        text_frame,
        font_metrics,
        wrap_columns,
    )
}

fn resolved_text_layout_for_state<'a>(
    surface: &'a UiSurface,
    target: UiNodeId,
    state: &UiEditableTextState,
) -> Option<InputMethodTextLayout<'a>> {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .find_map(|command| rendered_text_layout_for_command(command, target, state))
}

fn rendered_text_layout_for_command<'a>(
    command: &'a UiRenderCommand,
    target: UiNodeId,
    state: &UiEditableTextState,
) -> Option<InputMethodTextLayout<'a>> {
    if command.node_id != target || command.text.as_deref() != Some(state.text.as_str()) {
        return None;
    }
    let layout = command.text_layout.as_ref()?;
    if !rendered_text_layout_matches_state(layout, state) {
        return None;
    }
    Some(InputMethodTextLayout {
        layout,
        style: &command.style,
    })
}

fn rendered_text_layout_matches_state(
    layout: &UiResolvedTextLayout,
    state: &UiEditableTextState,
) -> bool {
    match layout.editable.as_ref() {
        Some(editable) => editable == state,
        None => true,
    }
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

fn range_rects_for_text(
    text: &str,
    range: UiTextRange,
    text_frame: UiFrame,
    font_metrics: FontMetrics,
    wrap_columns: Option<usize>,
) -> Vec<UiFrame> {
    let start = clamp_text_boundary(text, range.start);
    let end = clamp_text_boundary(text, range.end).max(start);
    let (mut line, mut column) = visual_line_column_for_offset(text, start, wrap_columns);
    let mut segment_line = line;
    let mut segment_start_column = column;
    let mut rects = Vec::new();

    if start == end {
        rects.push(line_segment_rect(
            text_frame,
            font_metrics,
            line,
            column,
            column,
        ));
        return rects;
    }

    for ch in text[start..end].chars() {
        if ch == '\n' {
            push_line_segment_rect(
                &mut rects,
                text_frame,
                font_metrics,
                segment_line,
                segment_start_column,
                column,
            );
            line += 1;
            column = 0;
            segment_line = line;
            segment_start_column = column;
            continue;
        }
        if wrap_columns.is_some_and(|columns| column >= columns) {
            push_line_segment_rect(
                &mut rects,
                text_frame,
                font_metrics,
                segment_line,
                segment_start_column,
                column,
            );
            line += 1;
            column = 0;
            segment_line = line;
            segment_start_column = column;
        }
        column += 1;
    }

    push_line_segment_rect(
        &mut rects,
        text_frame,
        font_metrics,
        segment_line,
        segment_start_column,
        column,
    );
    if rects.is_empty() {
        rects.push(line_segment_rect(
            text_frame,
            font_metrics,
            segment_line,
            segment_start_column,
            column,
        ));
    }
    rects
}

fn push_line_segment_rect(
    rects: &mut Vec<UiFrame>,
    text_frame: UiFrame,
    font_metrics: FontMetrics,
    line: usize,
    start_column: usize,
    end_column: usize,
) {
    if start_column == end_column {
        return;
    }
    rects.push(line_segment_rect(
        text_frame,
        font_metrics,
        line,
        start_column,
        end_column,
    ));
}

fn line_segment_rect(
    text_frame: UiFrame,
    font_metrics: FontMetrics,
    line: usize,
    start_column: usize,
    end_column: usize,
) -> UiFrame {
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

fn visual_line_column_for_offset(
    text: &str,
    offset: usize,
    wrap_columns: Option<usize>,
) -> (usize, usize) {
    let offset = clamp_text_boundary(text, offset);
    let mut line = 0;
    let mut column = 0;
    for ch in text[..offset].chars() {
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            if wrap_columns.is_some_and(|columns| column >= columns) {
                line += 1;
                column = 0;
            }
            column += 1;
        }
    }
    (line, column)
}

fn wrap_columns_for_node(
    surface: &UiSurface,
    target: UiNodeId,
    text_frame: UiFrame,
    font_metrics: FontMetrics,
) -> Option<usize> {
    let metadata = surface
        .tree
        .nodes
        .get(&target)
        .and_then(|node| node.template_metadata.as_ref());
    let Some(metadata) = metadata else {
        return None;
    };
    let wrap_enabled = metadata
        .attributes
        .get("wrap")
        .and_then(toml::Value::as_str)
        .map(|wrap| !wrap.eq_ignore_ascii_case("none"))
        .or_else(|| {
            metadata
                .attributes
                .get("multiline")
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false);
    wrap_enabled.then(|| {
        (text_frame.width / font_metrics.char_advance)
            .floor()
            .max(1.0) as usize
    })
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
