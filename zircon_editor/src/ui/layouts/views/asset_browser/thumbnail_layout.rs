use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

use super::thumbnail_nodes::thumbnail_control_id;

const THUMBNAIL_GRID_PADDING: f32 = 8.0;
const THUMBNAIL_GRID_GAP: f32 = 8.0;
const THUMBNAIL_CARD_MIN_WIDTH: f32 = 104.0;
const THUMBNAIL_CARD_MAX_WIDTH: f32 = 132.0;
const THUMBNAIL_CARD_HEIGHT_RATIO: f32 = 1.14;
const THUMBNAIL_CARD_MIN_HEIGHT: f32 = 146.0;
const THUMBNAIL_CARD_MAX_HEIGHT: f32 = 150.0;
const THUMBNAIL_VISUAL_MIN_HEIGHT: f32 = 72.0;
const THUMBNAIL_VISUAL_MAX_HEIGHT: f32 = 88.0;
const THUMBNAIL_CARD_INSET: f32 = 8.0;
const THUMBNAIL_INFO_BAND_HEIGHT: f32 = 52.0;
const THUMBNAIL_INFO_TEXT_INSET_X: f32 = 4.0;
const THUMBNAIL_SELECTION_MARKER_HEIGHT: f32 = 2.0;
const THUMBNAIL_NAME_PRIMARY_OFFSET_Y: f32 = 5.0;
const THUMBNAIL_NAME_CONTINUATION_OFFSET_Y: f32 = 17.0;
const THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT: f32 = 12.0;
const THUMBNAIL_NAME_CONTINUATION_LINE_HEIGHT: f32 = 10.0;
const THUMBNAIL_META_ROW_SINGLE_OFFSET_Y: f32 = 22.0;
const THUMBNAIL_META_ROW_STACKED_OFFSET_Y: f32 = 32.0;
const THUMBNAIL_TYPE_BADGE_MIN_WIDTH: f32 = 28.0;
const THUMBNAIL_TYPE_BADGE_MAX_WIDTH: f32 = 46.0;
const THUMBNAIL_TYPE_BADGE_HEIGHT: f32 = 12.0;
const THUMBNAIL_TYPE_BADGE_TEXT_INSET_X: f32 = 4.0;
const THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE: f32 = 8.0;
const THUMBNAIL_TYPE_BADGE_TEXT_WIDTH_RATIO: f32 = 0.56;
const THUMBNAIL_TYPE_BADGE_PADDING_X: f32 = 6.0;
const THUMBNAIL_TYPE_BADGE_MAX_WIDTH_RATIO: f32 = 0.55;
const THUMBNAIL_META_ROW_GAP: f32 = 5.0;
const THUMBNAIL_MAX_VISIBLE_ITEMS: usize = 8;
const THUMBNAIL_MAX_COLUMNS: usize = 6;

pub(super) fn has_thumbnail_grid(nodes: &[ViewTemplateNodeData]) -> bool {
    node_frame(nodes, "AssetBrowserThumbGridPanel").is_some()
}

pub(super) fn apply_compact_thumbnail_grid_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let count = thumbnail_card_count(nodes);
    if count == 0 {
        set_node_frame(nodes, "AssetBrowserThumbGridPanel", x, y, width, 0.0);
        return;
    }

    set_node_frame(nodes, "AssetBrowserThumbGridPanel", x, y, width, height);
    let columns = thumbnail_grid_columns(width, count);
    let rows = count.div_ceil(columns);
    let inner_width = (width - THUMBNAIL_GRID_PADDING * 2.0).max(0.0);
    let inner_height = (height - THUMBNAIL_GRID_PADDING * 2.0).max(0.0);
    let card_width = ((inner_width - THUMBNAIL_GRID_GAP * (columns - 1) as f32) / columns as f32)
        .clamp(THUMBNAIL_CARD_MIN_WIDTH, THUMBNAIL_CARD_MAX_WIDTH);
    let row_height = ((inner_height - THUMBNAIL_GRID_GAP * (rows - 1) as f32) / rows as f32)
        .clamp(THUMBNAIL_CARD_MIN_HEIGHT, THUMBNAIL_CARD_MAX_HEIGHT);
    let card_height = thumbnail_card_height_for_width(card_width).min(row_height);

    for index in 0..count {
        let column = index % columns;
        let row = index / columns;
        let card_x = x + THUMBNAIL_GRID_PADDING + column as f32 * (card_width + THUMBNAIL_GRID_GAP);
        let card_y = y + THUMBNAIL_GRID_PADDING + row as f32 * (card_height + THUMBNAIL_GRID_GAP);
        layout_thumbnail_card(nodes, index, card_x, card_y, card_width, card_height);
    }
}

fn thumbnail_card_count(nodes: &[ViewTemplateNodeData]) -> usize {
    (0..THUMBNAIL_MAX_VISIBLE_ITEMS)
        .take_while(|index| node_frame(nodes, &thumbnail_control_id("Card", *index)).is_some())
        .count()
}

fn thumbnail_grid_columns(width: f32, count: usize) -> usize {
    let inner_width = (width - THUMBNAIL_GRID_PADDING * 2.0).max(0.0);
    let columns = ((inner_width + THUMBNAIL_GRID_GAP)
        / (THUMBNAIL_CARD_MIN_WIDTH + THUMBNAIL_GRID_GAP))
        .floor()
        .max(1.0) as usize;
    columns.min(count).min(THUMBNAIL_MAX_COLUMNS).max(1)
}

fn thumbnail_card_height_for_width(width: f32) -> f32 {
    (width * THUMBNAIL_CARD_HEIGHT_RATIO)
        .clamp(THUMBNAIL_CARD_MIN_HEIGHT, THUMBNAIL_CARD_MAX_HEIGHT)
}

fn layout_thumbnail_card(
    nodes: &mut [ViewTemplateNodeData],
    index: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let inner_x = x + THUMBNAIL_CARD_INSET;
    let inner_width = (width - THUMBNAIL_CARD_INSET * 2.0).max(24.0);
    let band_height =
        THUMBNAIL_INFO_BAND_HEIGHT.min((height - THUMBNAIL_CARD_INSET * 2.0).max(0.0));
    let band_y = y + height - THUMBNAIL_CARD_INSET - band_height;
    let visual_y = y + THUMBNAIL_CARD_INSET;
    let visual_height = (band_y - visual_y - THUMBNAIL_SELECTION_MARKER_HEIGHT)
        .clamp(THUMBNAIL_VISUAL_MIN_HEIGHT, THUMBNAIL_VISUAL_MAX_HEIGHT);
    let text_x = inner_x + THUMBNAIL_INFO_TEXT_INSET_X;
    let text_width = (inner_width - THUMBNAIL_INFO_TEXT_INSET_X * 2.0).max(16.0);
    let continuation_height = thumbnail_name_continuation_height(nodes, index);
    let meta_row_y = band_y + thumbnail_meta_row_offset_y(continuation_height);
    let type_badge_width = thumbnail_type_badge_width(nodes, index, text_width);
    let type_text_x = text_x + THUMBNAIL_TYPE_BADGE_TEXT_INSET_X;
    let type_text_width = (type_badge_width - THUMBNAIL_TYPE_BADGE_TEXT_INSET_X * 2.0).max(0.0);
    let meta_x = text_x + type_badge_width + THUMBNAIL_META_ROW_GAP;
    let meta_width = (text_x + text_width - meta_x).max(0.0);

    set_node_frame(
        nodes,
        &thumbnail_control_id("Card", index),
        x,
        y,
        width,
        height,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("Visual", index),
        inner_x,
        visual_y,
        inner_width,
        visual_height,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("InfoBand", index),
        inner_x,
        band_y,
        inner_width,
        band_height,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("SelectionMarker", index),
        inner_x,
        band_y,
        inner_width,
        THUMBNAIL_SELECTION_MARKER_HEIGHT,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("Name", index),
        text_x,
        band_y + THUMBNAIL_NAME_PRIMARY_OFFSET_Y,
        text_width,
        THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("NameContinuation", index),
        text_x,
        band_y + THUMBNAIL_NAME_CONTINUATION_OFFSET_Y,
        text_width,
        continuation_height,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("TypeBadge", index),
        text_x,
        meta_row_y,
        type_badge_width,
        THUMBNAIL_TYPE_BADGE_HEIGHT,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("Type", index),
        type_text_x,
        meta_row_y,
        type_text_width,
        THUMBNAIL_TYPE_BADGE_HEIGHT,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("Meta", index),
        meta_x,
        meta_row_y,
        meta_width,
        THUMBNAIL_TYPE_BADGE_HEIGHT,
    );
}

fn thumbnail_type_badge_width(
    nodes: &[ViewTemplateNodeData],
    index: usize,
    text_width: f32,
) -> f32 {
    let label_chars = node_text(nodes, &thumbnail_control_id("Type", index))
        .map(|text| text.chars().count())
        .unwrap_or(0) as f32;
    let content_width =
        label_chars * THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE * THUMBNAIL_TYPE_BADGE_TEXT_WIDTH_RATIO
            + THUMBNAIL_TYPE_BADGE_PADDING_X * 2.0;
    let badge_max_width = THUMBNAIL_TYPE_BADGE_MAX_WIDTH
        .min(text_width * THUMBNAIL_TYPE_BADGE_MAX_WIDTH_RATIO)
        .max(THUMBNAIL_TYPE_BADGE_MIN_WIDTH);
    content_width
        .clamp(THUMBNAIL_TYPE_BADGE_MIN_WIDTH, badge_max_width)
        .max(0.0)
}

fn thumbnail_name_continuation_height(nodes: &[ViewTemplateNodeData], index: usize) -> f32 {
    if node_text(nodes, &thumbnail_control_id("NameContinuation", index))
        .map(|text| text.is_empty())
        .unwrap_or(true)
    {
        0.0
    } else {
        THUMBNAIL_NAME_CONTINUATION_LINE_HEIGHT
    }
}

fn thumbnail_meta_row_offset_y(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        THUMBNAIL_META_ROW_STACKED_OFFSET_Y
    } else {
        THUMBNAIL_META_ROW_SINGLE_OFFSET_Y
    }
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .map(|node| node.frame.clone())
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
    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == control_id)
    {
        node.frame.x = x;
        node.frame.y = y;
        node.frame.width = width;
        node.frame.height = height;
    }
}
