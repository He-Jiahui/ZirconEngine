use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

const SUMMARY_CARD_INSET_X: f32 = 8.0;
const SUMMARY_CARD_INSET_Y: f32 = 6.0;
const SUMMARY_VISUAL_MAX_EDGE: f32 = 44.0;
const SUMMARY_TEXT_GAP: f32 = 10.0;
const SUMMARY_TEXT_RIGHT_INSET: f32 = 10.0;
const SUMMARY_NAME_OFFSET_Y: f32 = 6.0;
const SUMMARY_NAME_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const SUMMARY_NAME_CONTINUATION_OFFSET_Y: f32 = SUMMARY_NAME_OFFSET_Y + SUMMARY_NAME_HEIGHT;
const SUMMARY_NAME_CONTINUATION_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const SUMMARY_META_ROW_OFFSET_Y: f32 = SUMMARY_NAME_CONTINUATION_OFFSET_Y + 2.0;
const SUMMARY_META_ROW_STACKED_OFFSET_Y: f32 =
    SUMMARY_NAME_CONTINUATION_OFFSET_Y + SUMMARY_NAME_CONTINUATION_HEIGHT + 1.0;
const SUMMARY_META_ROW_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const SUMMARY_TYPE_BADGE_MIN_WIDTH: f32 = 40.0;
const SUMMARY_TYPE_BADGE_MAX_WIDTH: f32 = 76.0;
const SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO: f32 = 0.50;
const SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const SUMMARY_TYPE_BADGE_PADDING_X: f32 = 6.0;
const SUMMARY_TYPE_BADGE_TEXT_INSET_X: f32 = 4.0;
const SUMMARY_META_ROW_GAP: f32 = 6.0;
const SUMMARY_REVISION_MIN_WIDTH: f32 = 34.0;
const SUMMARY_REVISION_MAX_WIDTH: f32 = 62.0;
const SUMMARY_REVISION_MAX_WIDTH_RATIO: f32 = 0.28;
const SUMMARY_REVISION_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const SUMMARY_REVISION_PADDING_X: f32 = 4.0;
const SUMMARY_CONTROL_PREFIX: &str = "AssetBrowserContentPreview";

pub(super) fn apply_compact_content_preview_summary_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    if width <= f32::EPSILON || height <= f32::EPSILON {
        collapse_summary_nodes(nodes, finite_coordinate(x), finite_coordinate(y));
        return;
    }

    let visual_edge = summary_visual_slot_edge(width, height);
    let card_right = x + width;
    let text_x = (x + SUMMARY_CARD_INSET_X + visual_edge + SUMMARY_TEXT_GAP).min(card_right);
    let text_width = finite_non_negative(card_right - text_x - SUMMARY_TEXT_RIGHT_INSET);
    let labels = summary_text_labels(nodes);
    let continuation_height = summary_name_continuation_height(labels.continuation.unwrap_or(""))
        .min(summary_line_height(
            height,
            SUMMARY_NAME_CONTINUATION_OFFSET_Y,
            SUMMARY_NAME_CONTINUATION_HEIGHT,
        ));
    let meta_y = y + summary_meta_row_offset_y(continuation_height);
    let type_badge_width = summary_type_badge_width(labels.kind.unwrap_or(""), text_width);
    let revision_width = summary_revision_width(labels.revision.unwrap_or(""), text_width);
    let revision_x = if revision_width > 0.0 {
        (card_right - SUMMARY_TEXT_RIGHT_INSET - revision_width).max(text_x)
    } else {
        card_right - SUMMARY_TEXT_RIGHT_INSET
    };
    let state_x = text_x + type_badge_width + SUMMARY_META_ROW_GAP;
    let state_width = finite_non_negative(revision_x - state_x - SUMMARY_META_ROW_GAP);

    let frames = summary_frames(
        x,
        y,
        width,
        height,
        visual_edge,
        text_x,
        text_width,
        continuation_height,
        meta_y,
        type_badge_width,
        state_x,
        state_width,
        revision_x,
        revision_width,
    );
    apply_summary_node_frames(nodes, &frames);
}

fn summary_visual_slot_edge(width: f32, height: f32) -> f32 {
    let available_width = finite_non_negative(width - SUMMARY_CARD_INSET_X * 2.0);
    let available_height = finite_non_negative(height - SUMMARY_CARD_INSET_Y * 2.0);
    available_width
        .min(available_height)
        .min(SUMMARY_VISUAL_MAX_EDGE)
}

fn summary_name_continuation_height(text: &str) -> f32 {
    if text.is_empty() {
        0.0
    } else {
        SUMMARY_NAME_CONTINUATION_HEIGHT
    }
}

fn summary_meta_row_offset_y(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        SUMMARY_META_ROW_STACKED_OFFSET_Y
    } else {
        SUMMARY_META_ROW_OFFSET_Y
    }
}

fn summary_type_badge_width(label: &str, text_width: f32) -> f32 {
    let text_width = finite_non_negative(text_width);
    if label.is_empty() || text_width <= f32::EPSILON {
        return 0.0;
    }
    let content_width = measure_runtime_text_width(label, SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE)
        + SUMMARY_TYPE_BADGE_PADDING_X * 2.0;
    let badge_max_width = SUMMARY_TYPE_BADGE_MAX_WIDTH
        .min(text_width * SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO)
        .min(text_width);
    content_width
        .max(SUMMARY_TYPE_BADGE_MIN_WIDTH.min(badge_max_width))
        .min(badge_max_width)
}

fn summary_revision_width(label: &str, text_width: f32) -> f32 {
    let text_width = finite_non_negative(text_width);
    if label.is_empty() || text_width <= f32::EPSILON {
        return 0.0;
    }
    let content_width = measure_runtime_text_width(label, SUMMARY_REVISION_FONT_SIZE)
        + SUMMARY_REVISION_PADDING_X * 2.0;
    let max_width = SUMMARY_REVISION_MAX_WIDTH
        .min(text_width * SUMMARY_REVISION_MAX_WIDTH_RATIO)
        .min(text_width);
    content_width
        .max(SUMMARY_REVISION_MIN_WIDTH.min(max_width))
        .min(max_width)
}

fn summary_line_height(card_height: f32, offset_y: f32, preferred_height: f32) -> f32 {
    finite_non_negative(preferred_height).min(finite_non_negative(
        finite_non_negative(card_height) - finite_non_negative(offset_y),
    ))
}

fn collapse_summary_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        if node.control_id.starts_with("AssetBrowserContentPreview") {
            node.frame = ViewTemplateFrameData {
                x,
                y,
                width: 0.0,
                height: 0.0,
            };
        }
    }
}

#[derive(Clone, Copy, Default)]
struct SummaryTextLabels<'a> {
    continuation: Option<&'a str>,
    kind: Option<&'a str>,
    revision: Option<&'a str>,
}

fn summary_text_labels(nodes: &[ViewTemplateNodeData]) -> SummaryTextLabels<'_> {
    let mut labels = SummaryTextLabels::default();
    for node in nodes {
        let Some(suffix) = node.control_id.strip_prefix(SUMMARY_CONTROL_PREFIX) else {
            continue;
        };
        match suffix {
            "NameContinuation" if labels.continuation.is_none() => {
                labels.continuation = Some(node.text.as_str());
            }
            "Type" if labels.kind.is_none() => labels.kind = Some(node.text.as_str()),
            "Revision" if labels.revision.is_none() => {
                labels.revision = Some(node.text.as_str());
            }
            _ => {}
        }
    }
    labels
}

#[allow(clippy::too_many_arguments)]
fn summary_frames(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    visual_edge: f32,
    text_x: f32,
    text_width: f32,
    continuation_height: f32,
    meta_y: f32,
    type_badge_width: f32,
    state_x: f32,
    state_width: f32,
    revision_x: f32,
    revision_width: f32,
) -> [ViewTemplateFrameData; 9] {
    let meta_height = summary_line_height(height, meta_y - y, SUMMARY_META_ROW_HEIGHT);
    [
        summary_frame(x, y, width, height),
        summary_frame(
            x + SUMMARY_CARD_INSET_X,
            y + SUMMARY_CARD_INSET_Y,
            visual_edge,
            visual_edge,
        ),
        summary_frame(
            text_x,
            y + SUMMARY_NAME_OFFSET_Y,
            text_width,
            summary_line_height(height, SUMMARY_NAME_OFFSET_Y, SUMMARY_NAME_HEIGHT),
        ),
        summary_frame(
            text_x,
            y + SUMMARY_NAME_CONTINUATION_OFFSET_Y,
            text_width,
            continuation_height,
        ),
        summary_frame(text_x, meta_y, 0.0, 0.0),
        summary_frame(text_x, meta_y, type_badge_width, meta_height),
        summary_frame(
            text_x + SUMMARY_TYPE_BADGE_TEXT_INSET_X,
            meta_y,
            finite_non_negative(type_badge_width - SUMMARY_TYPE_BADGE_TEXT_INSET_X * 2.0),
            meta_height,
        ),
        summary_frame(state_x, meta_y, state_width, meta_height),
        summary_frame(revision_x, meta_y, revision_width, meta_height),
    ]
}

fn apply_summary_node_frames(
    nodes: &mut [ViewTemplateNodeData],
    frames: &[ViewTemplateFrameData; 9],
) {
    for node in nodes {
        let Some(frame_index) = summary_frame_index(&node.control_id) else {
            continue;
        };
        node.frame = frames[frame_index].clone();
    }
}

fn summary_frame_index(control_id: &str) -> Option<usize> {
    match control_id.strip_prefix(SUMMARY_CONTROL_PREFIX)? {
        "Card" => Some(0),
        "Visual" => Some(1),
        "Name" => Some(2),
        "NameContinuation" => Some(3),
        "Meta" => Some(4),
        "TypeBadge" => Some(5),
        "Type" => Some(6),
        "State" => Some(7),
        "Revision" => Some(8),
        _ => None,
    }
}

fn summary_frame(x: f32, y: f32, width: f32, height: f32) -> ViewTemplateFrameData {
    ViewTemplateFrameData {
        x: finite_coordinate(x),
        y: finite_coordinate(y),
        width: finite_non_negative(width),
        height: finite_non_negative(height),
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/summary_layout.rs"]
mod tests;
