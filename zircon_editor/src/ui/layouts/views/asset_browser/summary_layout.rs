use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

const SUMMARY_CARD_INSET_X: f32 = 8.0;
const SUMMARY_CARD_INSET_Y: f32 = 6.0;
const SUMMARY_VISUAL_MIN_WIDTH: f32 = 48.0;
const SUMMARY_VISUAL_MAX_WIDTH: f32 = 64.0;
const SUMMARY_VISUAL_WIDTH_RATIO: f32 = 0.18;
const SUMMARY_TEXT_GAP: f32 = 14.0;
const SUMMARY_TEXT_RIGHT_INSET: f32 = 10.0;
const SUMMARY_TEXT_MIN_WIDTH: f32 = 32.0;
const SUMMARY_NAME_OFFSET_Y: f32 = 7.0;
const SUMMARY_NAME_HEIGHT: f32 = 12.0;
const SUMMARY_NAME_CONTINUATION_OFFSET_Y: f32 = 19.0;
const SUMMARY_NAME_CONTINUATION_HEIGHT: f32 = 12.0;
const SUMMARY_META_ROW_OFFSET_Y: f32 = 27.0;
const SUMMARY_META_ROW_STACKED_OFFSET_Y: f32 = 36.0;
const SUMMARY_META_ROW_HEIGHT: f32 = 12.0;
const SUMMARY_TYPE_BADGE_MIN_WIDTH: f32 = 28.0;
const SUMMARY_TYPE_BADGE_MAX_WIDTH: f32 = 48.0;
const SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO: f32 = 0.42;
const SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE: f32 = 8.0;
const SUMMARY_TYPE_BADGE_TEXT_WIDTH_RATIO: f32 = 0.56;
const SUMMARY_TYPE_BADGE_PADDING_X: f32 = 6.0;
const SUMMARY_TYPE_BADGE_TEXT_INSET_X: f32 = 4.0;
const SUMMARY_META_ROW_GAP: f32 = 6.0;
const SUMMARY_REVISION_MIN_WIDTH: f32 = 34.0;
const SUMMARY_REVISION_MAX_WIDTH: f32 = 62.0;
const SUMMARY_REVISION_MAX_WIDTH_RATIO: f32 = 0.28;
const SUMMARY_REVISION_FONT_SIZE: f32 = 9.0;
const SUMMARY_REVISION_TEXT_WIDTH_RATIO: f32 = 0.54;
const SUMMARY_REVISION_PADDING_X: f32 = 4.0;

pub(super) fn apply_compact_content_preview_summary_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let visual_width = (width * SUMMARY_VISUAL_WIDTH_RATIO)
        .clamp(SUMMARY_VISUAL_MIN_WIDTH, SUMMARY_VISUAL_MAX_WIDTH)
        .min((width - SUMMARY_CARD_INSET_X * 2.0).max(0.0));
    let text_x = x + SUMMARY_CARD_INSET_X + visual_width + SUMMARY_TEXT_GAP;
    let text_width = (x + width - text_x - SUMMARY_TEXT_RIGHT_INSET).max(SUMMARY_TEXT_MIN_WIDTH);
    let continuation_height = summary_name_continuation_height(nodes);
    let meta_y = y + summary_meta_row_offset_y(continuation_height);
    let type_badge_width = summary_type_badge_width(nodes, text_width);
    let revision_width = summary_revision_width(nodes, text_width);
    let revision_x = if revision_width > 0.0 {
        (x + width - SUMMARY_TEXT_RIGHT_INSET - revision_width).max(text_x)
    } else {
        x + width - SUMMARY_TEXT_RIGHT_INSET
    };
    let state_x = text_x + type_badge_width + SUMMARY_META_ROW_GAP;
    let state_width = (revision_x - state_x - SUMMARY_META_ROW_GAP).max(0.0);

    set_node_frame(nodes, "AssetBrowserContentPreviewCard", x, y, width, height);
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewVisual",
        x + SUMMARY_CARD_INSET_X,
        y + SUMMARY_CARD_INSET_Y,
        visual_width,
        (height - SUMMARY_CARD_INSET_Y * 2.0).max(28.0),
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewName",
        text_x,
        y + SUMMARY_NAME_OFFSET_Y,
        text_width,
        SUMMARY_NAME_HEIGHT,
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
        SUMMARY_META_ROW_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewType",
        text_x + SUMMARY_TYPE_BADGE_TEXT_INSET_X,
        meta_y,
        (type_badge_width - SUMMARY_TYPE_BADGE_TEXT_INSET_X * 2.0).max(0.0),
        SUMMARY_META_ROW_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewState",
        state_x,
        meta_y,
        state_width,
        SUMMARY_META_ROW_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentPreviewRevision",
        revision_x,
        meta_y,
        revision_width,
        SUMMARY_META_ROW_HEIGHT,
    );
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
    let label_chars = node_text(nodes, "AssetBrowserContentPreviewType")
        .map(|text| text.chars().count())
        .unwrap_or(0) as f32;
    if label_chars == 0.0 {
        return 0.0;
    }
    let content_width =
        label_chars * SUMMARY_TYPE_BADGE_TEXT_FONT_SIZE * SUMMARY_TYPE_BADGE_TEXT_WIDTH_RATIO
            + SUMMARY_TYPE_BADGE_PADDING_X * 2.0;
    let badge_max_width = SUMMARY_TYPE_BADGE_MAX_WIDTH
        .min(text_width * SUMMARY_TYPE_BADGE_MAX_WIDTH_RATIO)
        .max(SUMMARY_TYPE_BADGE_MIN_WIDTH);
    content_width.clamp(SUMMARY_TYPE_BADGE_MIN_WIDTH, badge_max_width)
}

fn summary_revision_width(nodes: &[ViewTemplateNodeData], text_width: f32) -> f32 {
    let label_chars = node_text(nodes, "AssetBrowserContentPreviewRevision")
        .map(|text| text.chars().count())
        .unwrap_or(0) as f32;
    if label_chars == 0.0 {
        return 0.0;
    }
    let content_width =
        label_chars * SUMMARY_REVISION_FONT_SIZE * SUMMARY_REVISION_TEXT_WIDTH_RATIO
            + SUMMARY_REVISION_PADDING_X * 2.0;
    let max_width = SUMMARY_REVISION_MAX_WIDTH
        .min(text_width * SUMMARY_REVISION_MAX_WIDTH_RATIO)
        .max(SUMMARY_REVISION_MIN_WIDTH);
    content_width.clamp(SUMMARY_REVISION_MIN_WIDTH, max_width)
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
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        };
    }
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
            node("AssetBrowserContentPreviewType", "Label", "MESH"),
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
        assert!(name.x > visual.x + visual.width);
        assert_eq!(continuation.x, name.x);
        assert!(continuation.y > name.y);
        assert!(continuation.height > 0.0);
        assert_eq!(legacy_meta.height, 0.0);
        assert!(type_badge.y >= continuation.y + continuation.height);
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
        let continuation = frame(&nodes, "AssetBrowserContentPreviewNameContinuation");
        let type_badge = frame(&nodes, "AssetBrowserContentPreviewTypeBadge");

        assert_eq!(continuation.height, 0.0);
        assert!(type_badge.y > name.y);
        assert!(
            type_badge.y - name.y < 24.0,
            "single-line summary should not reserve second-line spacing: name={:?}, badge={:?}",
            name,
            type_badge
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
}
