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
    let continuation_height = summary_name_continuation_height(nodes).min(summary_line_height(
        height,
        SUMMARY_NAME_CONTINUATION_OFFSET_Y,
        SUMMARY_NAME_CONTINUATION_HEIGHT,
    ));
    let meta_y = y + summary_meta_row_offset_y(continuation_height);
    let type_badge_width = summary_type_badge_width(nodes, text_width);
    let revision_width = summary_revision_width(nodes, text_width);
    let revision_x = if revision_width > 0.0 {
        (card_right - SUMMARY_TEXT_RIGHT_INSET - revision_width).max(text_x)
    } else {
        card_right - SUMMARY_TEXT_RIGHT_INSET
    };
    let state_x = text_x + type_badge_width + SUMMARY_META_ROW_GAP;
    let state_width = finite_non_negative(revision_x - state_x - SUMMARY_META_ROW_GAP);

    set_node_frame(nodes, "AssetBrowserContentPreviewCard", x, y, width, height);
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewVisual",
        x + SUMMARY_CARD_INSET_X,
        y + SUMMARY_CARD_INSET_Y,
        visual_edge,
        visual_edge,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewName",
        text_x,
        y + SUMMARY_NAME_OFFSET_Y,
        text_width,
        summary_line_height(height, SUMMARY_NAME_OFFSET_Y, SUMMARY_NAME_HEIGHT),
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewNameContinuation",
        text_x,
        y + SUMMARY_NAME_CONTINUATION_OFFSET_Y,
        text_width,
        continuation_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewMeta",
        text_x,
        meta_y,
        0.0,
        0.0,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewTypeBadge",
        text_x,
        meta_y,
        type_badge_width,
        summary_line_height(height, meta_y - y, SUMMARY_META_ROW_HEIGHT),
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewType",
        text_x + SUMMARY_TYPE_BADGE_TEXT_INSET_X,
        meta_y,
        finite_non_negative(type_badge_width - SUMMARY_TYPE_BADGE_TEXT_INSET_X * 2.0),
        summary_line_height(height, meta_y - y, SUMMARY_META_ROW_HEIGHT),
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewState",
        state_x,
        meta_y,
        state_width,
        summary_line_height(height, meta_y - y, SUMMARY_META_ROW_HEIGHT),
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewRevision",
        revision_x,
        meta_y,
        revision_width,
        summary_line_height(height, meta_y - y, SUMMARY_META_ROW_HEIGHT),
    );
}

fn summary_visual_slot_edge(width: f32, height: f32) -> f32 {
    let available_width = finite_non_negative(width - SUMMARY_CARD_INSET_X * 2.0);
    let available_height = finite_non_negative(height - SUMMARY_CARD_INSET_Y * 2.0);
    available_width
        .min(available_height)
        .min(SUMMARY_VISUAL_MAX_EDGE)
}

fn summary_name_continuation_height(nodes: &[ViewTemplateNodeData]) -> f32 {
    if node_text(nodes, "AssetBrowserContentPreviewNameContinuation")
        .map(|text| text.is_empty())
        .unwrap_or(true)
    {
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

fn summary_type_badge_width(nodes: &[ViewTemplateNodeData], text_width: f32) -> f32 {
    let label = node_text(nodes, "AssetBrowserContentPreviewType").unwrap_or("");
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

fn summary_revision_width(nodes: &[ViewTemplateNodeData], text_width: f32) -> f32 {
    let label = node_text(nodes, "AssetBrowserContentPreviewRevision").unwrap_or("");
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

fn node_text<'a>(nodes: &'a [ViewTemplateNodeData], control_id: &str) -> Option<&'a str> {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .map(|node| node.text.as_str())
}

fn set_node_frame(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id.as_str() == control_id)
    {
        node.frame = ViewTemplateFrameData {
            x: finite_coordinate(x),
            y: finite_coordinate(y),
            width: finite_non_negative(width),
            height: finite_non_negative(height),
        };
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
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_layout_splits_meta_row_and_collapses_legacy_label() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node(
                "AssetBrowserContentPreviewNameContinuation",
                "Label",
                "authoring.zui",
            ),
            node(
                "AssetBrowserContentPreviewMeta",
                "Label",
                "Mesh | Ready | rev 12",
            ),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "UI Layout"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);

        let card = frame(&nodes, "AssetBrowserContentPreviewCard");
        let visual = frame(&nodes, "AssetBrowserContentPreviewVisual");
        let name = frame(&nodes, "AssetBrowserContentPreviewName");
        let continuation = frame(&nodes, "AssetBrowserContentPreviewNameContinuation");
        let legacy_meta = frame(&nodes, "AssetBrowserContentPreviewMeta");
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");
        let type_label = frame(&nodes, "AssetBrowserContentPreviewType");
        let state = frame(&nodes, "AssetBrowserContentPreviewState");
        let revision = frame(&nodes, "AssetBrowserContentPreviewRevision");

        assert_eq!(card.width, 420.0);
        assert!(visual.x > card.x);
        assert_eq!(visual.width, visual.height);
        assert!(
            visual.width <= SUMMARY_VISUAL_MAX_EDGE,
            "summary icon slot should stay square and compact: {:?}",
            visual
        );
        assert!(name.x > visual.x + visual.width);
        assert!(
            name.height >= 14.0,
            "summary primary title should keep a baseline-safe slot for compact file names: {:?}",
            name
        );
        assert!(
            name.x - (visual.x + visual.width) <= SUMMARY_TEXT_GAP,
            "summary text should sit near the square preview slot: visual={:?}, name={:?}",
            visual,
            name
        );
        assert_eq!(continuation.x, name.x);
        assert!(continuation.y >= name.y + name.height);
        assert!(continuation.height > 0.0);
        assert_eq!(legacy_meta.height, 0.0);
        assert!(type_badge.y >= continuation.y + continuation.height);
        assert_close(
            type_badge.width,
            expected_summary_type_width("UI Layout", name.width),
        );
        assert!(type_badge.width <= SUMMARY_TYPE_BADGE_MAX_WIDTH);
        assert_eq!(type_label.x, type_badge.x + SUMMARY_TYPE_BADGE_TEXT_INSET_X);
        assert!(state.x > type_badge.x + type_badge.width);
        assert!(revision.x > state.x);
        assert!(revision.x + revision.width <= card.x + card.width);
    }

    #[test]
    fn summary_layout_collapses_empty_name_continuation_and_keeps_meta_compact() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node("AssetBrowserContentPreviewNameContinuation", "Label", ""),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "MESH"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);

        let name = frame(&nodes, "AssetBrowserContentPreviewName");
        let visual = frame(&nodes, "AssetBrowserContentPreviewVisual");
        let continuation = frame(&nodes, "AssetBrowserContentPreviewNameContinuation");
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");

        assert_eq!(visual.width, visual.height);
        assert!(
            visual.width <= SUMMARY_VISUAL_MAX_EDGE,
            "single-line summary preview should use a compact square slot: {:?}",
            visual
        );
        assert_eq!(continuation.height, 0.0);
        assert!(type_badge.y > name.y);
        assert!(
            type_badge.y - name.y < 24.0,
            "single-line summary should not reserve second-line spacing: name={:?}, badge={:?}",
            name,
            type_badge
        );
    }

    #[test]
    fn summary_badge_and_revision_widths_use_runtime_text_measurement() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node(
                "AssetBrowserContentPreviewName",
                "Label",
                "MaterialVariant.zmat",
            ),
            node("AssetBrowserContentPreviewNameContinuation", "Label", ""),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", ""),
            node("AssetBrowserContentPreviewType", "Label", "iiiiiiiiii"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev iiiiii"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 80.0, 320.0, 420.0, 50.0);

        let text_width = frame(&nodes, "AssetBrowserContentPreviewName").width;
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");
        let revision = frame(&nodes, "AssetBrowserContentPreviewRevision");
        let expected_type_width = expected_summary_type_width("iiiiiiiiii", text_width);
        let expected_revision_width = expected_summary_revision_width("rev iiiiii", text_width);

        assert_close(type_badge.width, expected_type_width);
        assert_close(revision.width, expected_revision_width);
    }

    #[test]
    fn summary_layout_clips_tiny_cards_without_expanding_their_children() {
        let mut nodes = vec![
            node("AssetBrowserContentPreviewCard", "Panel", ""),
            node("AssetBrowserContentPreviewVisual", "Panel", ""),
            node("AssetBrowserContentPreviewName", "Label", "Hero.mesh"),
            node("AssetBrowserContentPreviewNameContinuation", "Label", ""),
            node("AssetBrowserContentPreviewMeta", "Label", ""),
            node("AssetBrowserContentPreviewTypeBadge", "Panel", "MESH"),
            node("AssetBrowserContentPreviewType", "Label", "MESH"),
            node("AssetBrowserContentPreviewState", "Label", "Ready"),
            node("AssetBrowserContentPreviewRevision", "Label", "rev 12"),
        ];

        apply_compact_content_preview_summary_layout(&mut nodes, 40.0, 80.0, 24.0, 18.0);

        let card = frame(&nodes, "AssetBrowserContentPreviewCard");
        for node in &nodes {
            if node.frame.width <= f32::EPSILON || node.frame.height <= f32::EPSILON {
                continue;
            }
            assert!(node.frame.x >= card.x);
            assert!(node.frame.y >= card.y);
            assert!(node.frame.x + node.frame.width <= card.x + card.width);
            assert!(node.frame.y + node.frame.height <= card.y + card.height);
        }

        apply_compact_content_preview_summary_layout(&mut nodes, 40.0, 80.0, 0.0, 0.0);
        assert!(
            nodes
                .iter()
                .all(|node| node.frame.width == 0.0 && node.frame.height == 0.0)
        );
    }

    fn node(control_id: &str, role: &str, text: &str) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            control_id: control_id.into(),
            role: role.into(),
            text: text.into(),
            frame: ViewTemplateFrameData::default(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> ViewTemplateFrameData {
        nodes
            .iter()
            .find(|node| node.control_id.as_str() == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing {control_id}"))
    }

    fn expected_summary_type_width(label: &str, text_width: f32) -> f32 {
        let content_width = measure_runtime_text_width(label, SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE)
            + SUMMARY_TYPE_BADGE_PADDING_X * 2.0;
        let max_width = SUMMARY_TYPE_BADGE_MAX_WIDTH
            .min(text_width * SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO)
            .min(text_width);
        content_width
            .max(SUMMARY_TYPE_BADGE_MIN_WIDTH.min(max_width))
            .min(max_width)
    }

    fn expected_summary_revision_width(label: &str, text_width: f32) -> f32 {
        let content_width = measure_runtime_text_width(label, SUMMARY_REVISION_FONT_SIZE)
            + SUMMARY_REVISION_PADDING_X * 2.0;
        let max_width = SUMMARY_REVISION_MAX_WIDTH
            .min(text_width * SUMMARY_REVISION_MAX_WIDTH_RATIO)
            .min(text_width);
        content_width
            .max(SUMMARY_REVISION_MIN_WIDTH.min(max_width))
            .min(max_width)
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.01,
            "expected {expected:.3}, got {actual:.3}",
        );
    }
}
