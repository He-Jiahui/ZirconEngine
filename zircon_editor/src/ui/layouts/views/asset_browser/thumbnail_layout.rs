use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::asset_content_layout::{
    AssetThumbnailGridMetrics, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

use super::thumbnail_nodes::{compact_thumbnail_file_name_to_width, thumbnail_control_id};

const THUMBNAIL_VISUAL_MIN_HEIGHT: f32 = 72.0;
const THUMBNAIL_VISUAL_MAX_HEIGHT: f32 = 88.0;
const THUMBNAIL_CARD_INSET: f32 = 8.0;
const THUMBNAIL_INFO_BAND_SINGLE_LINE_HEIGHT: f32 = 42.0;
const THUMBNAIL_INFO_BAND_STACKED_HEIGHT: f32 = 54.0;
const THUMBNAIL_INFO_TEXT_INSET_X: f32 = 5.0;
const THUMBNAIL_SELECTION_MARKER_WIDTH: f32 = 0.0;
const THUMBNAIL_NAME_PRIMARY_OFFSET_Y: f32 = 5.0;
const THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const THUMBNAIL_NAME_CONTINUATION_OFFSET_Y: f32 =
    THUMBNAIL_NAME_PRIMARY_OFFSET_Y + THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT;
const THUMBNAIL_NAME_CONTINUATION_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const THUMBNAIL_META_ROW_SINGLE_OFFSET_Y: f32 = 25.0;
const THUMBNAIL_META_ROW_STACKED_OFFSET_Y: f32 = 36.0;
const THUMBNAIL_TYPE_BADGE_MIN_WIDTH: f32 = 42.0;
const THUMBNAIL_TYPE_BADGE_MAX_WIDTH: f32 = 48.0;
const THUMBNAIL_TYPE_BADGE_HEIGHT: f32 = 13.0;
const THUMBNAIL_TYPE_BADGE_TEXT_INSET_X: f32 = 5.0;
const THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const THUMBNAIL_TYPE_BADGE_PADDING_X: f32 = 6.0;
const THUMBNAIL_TYPE_BADGE_MAX_WIDTH_RATIO: f32 = 0.55;
const THUMBNAIL_META_ROW_GAP: f32 = 5.0;
const THUMBNAIL_CARD_LAYOUT_PARTS: [&str; 9] = [
    "Card",
    "Visual",
    "InfoBand",
    "SelectionMarker",
    "Name",
    "NameContinuation",
    "TypeBadge",
    "Type",
    "Meta",
];

pub(super) fn has_thumbnail_grid(nodes: &[ViewTemplateNodeData]) -> bool {
    node_frame(nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID).is_some()
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
        set_node_frame(
            nodes,
            BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
            x,
            y,
            width,
            0.0,
        );
        return;
    }

    let metrics = AssetThumbnailGridMetrics::new(width, count);
    set_node_frame(
        nodes,
        BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
        x,
        y,
        width,
        height,
    );
    set_node_value_number(
        nodes,
        BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
        metrics.content_extent(),
    );

    for index in 0..count {
        let Some(frame) = metrics.item_frame(index) else {
            collapse_thumbnail_card(nodes, index);
            continue;
        };
        layout_thumbnail_card(
            nodes,
            index,
            x + frame.x,
            y + frame.y,
            frame.width,
            frame.height,
        );
    }
}

fn collapse_thumbnail_card(nodes: &mut [ViewTemplateNodeData], index: usize) {
    for part in THUMBNAIL_CARD_LAYOUT_PARTS {
        set_node_frame(
            nodes,
            &thumbnail_control_id(part, index),
            0.0,
            0.0,
            0.0,
            0.0,
        );
    }
}

fn thumbnail_card_count(nodes: &[ViewTemplateNodeData]) -> usize {
    (0..nodes.len())
        .take_while(|index| node_frame(nodes, &thumbnail_control_id("Card", *index)).is_some())
        .count()
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
    let continuation_height = thumbnail_name_continuation_height(nodes, index);
    let band_height = thumbnail_info_band_height(continuation_height)
        .min((height - THUMBNAIL_CARD_INSET * 2.0).max(0.0));
    let band_y = y + height - THUMBNAIL_CARD_INSET - band_height;
    let visual_y = y + THUMBNAIL_CARD_INSET;
    let visual_height =
        (band_y - visual_y).clamp(THUMBNAIL_VISUAL_MIN_HEIGHT, THUMBNAIL_VISUAL_MAX_HEIGHT);
    let text_x = inner_x + THUMBNAIL_INFO_TEXT_INSET_X;
    let text_width = (inner_width - THUMBNAIL_INFO_TEXT_INSET_X * 2.0).max(16.0);
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
        THUMBNAIL_SELECTION_MARKER_WIDTH,
        band_height,
    );
    set_node_frame(
        nodes,
        &thumbnail_control_id("Name", index),
        text_x,
        band_y + THUMBNAIL_NAME_PRIMARY_OFFSET_Y,
        text_width,
        THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT,
    );
    compact_thumbnail_name_to_frame(nodes, index, text_width);
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

fn compact_thumbnail_name_to_frame(
    nodes: &mut [ViewTemplateNodeData],
    index: usize,
    max_width: f32,
) {
    let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == thumbnail_control_id("Name", index))
    else {
        return;
    };
    if node.value_text.is_empty() {
        return;
    }

    node.text = compact_thumbnail_file_name_to_width(node.value_text.as_str(), max_width).into();
}

fn thumbnail_type_badge_width(
    nodes: &[ViewTemplateNodeData],
    index: usize,
    text_width: f32,
) -> f32 {
    let label = node_text(nodes, &thumbnail_control_id("Type", index)).unwrap_or("");
    let content_width = measure_runtime_text_width(label, THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_type_badge_width_uses_runtime_text_measurement() {
        let nodes = vec![node(&thumbnail_control_id("Type", 0), "Label", "iiiiiiii")];
        let text_width = 96.0;
        let width = thumbnail_type_badge_width(&nodes, 0, text_width);
        let content_width =
            measure_runtime_text_width("iiiiiiii", THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE)
                + THUMBNAIL_TYPE_BADGE_PADDING_X * 2.0;
        let max_width = THUMBNAIL_TYPE_BADGE_MAX_WIDTH
            .min(text_width * THUMBNAIL_TYPE_BADGE_MAX_WIDTH_RATIO)
            .max(THUMBNAIL_TYPE_BADGE_MIN_WIDTH);
        let expected = content_width
            .clamp(THUMBNAIL_TYPE_BADGE_MIN_WIDTH, max_width)
            .max(0.0);

        assert!(
            (width - expected).abs() <= 0.01,
            "expected {expected:.3}, got {width:.3}",
        );
    }

    #[test]
    fn thumbnail_badge_measurement_uses_the_workbench_caption_size() {
        assert_eq!(
            THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE,
            zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
        );
    }

    #[test]
    fn thumbnail_text_line_geometry_follows_workbench_typography_without_overlap() {
        assert_eq!(
            THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT,
            EditorTypographyTokens::WORKBENCH_BODY_SIZE
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO
        );
        assert_eq!(
            THUMBNAIL_NAME_CONTINUATION_LINE_HEIGHT,
            EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO
        );
        assert!(
            THUMBNAIL_NAME_CONTINUATION_OFFSET_Y
                >= THUMBNAIL_NAME_PRIMARY_OFFSET_Y + THUMBNAIL_NAME_PRIMARY_LINE_HEIGHT
        );
    }

    #[test]
    fn thumbnail_file_name_compacts_to_its_actual_card_text_frame() {
        let source_name = "workbench_extension_accessibility_workspace.zui";
        let mut name = node(&thumbnail_control_id("Name", 0), "Label", source_name);
        name.value_text = source_name.into();
        let mut nodes = vec![
            node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""),
            node(&thumbnail_control_id("Card", 0), "Panel", ""),
            name,
        ];

        apply_compact_thumbnail_grid_layout(&mut nodes, 0.0, 0.0, 120.0, 160.0);

        let name_frame = node_frame(&nodes, &thumbnail_control_id("Name", 0))
            .expect("thumbnail name should receive a frame");
        let compact_name = node_text(&nodes, &thumbnail_control_id("Name", 0))
            .expect("thumbnail name should retain text");
        assert!(compact_name.ends_with(".zui"));
        assert!(
            measure_runtime_text_width(compact_name, EditorTypographyTokens::WORKBENCH_BODY_SIZE)
                <= name_frame.width + 0.01,
            "thumbnail title must fit its real frame: text={compact_name}, frame={name_frame:?}"
        );
    }

    #[test]
    fn collapsed_grid_clears_each_thumbnail_card_descendant() {
        let mut nodes = [
            node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""),
            node(&thumbnail_control_id("Card", 0), "Panel", ""),
            node(&thumbnail_control_id("Visual", 0), "Panel", ""),
            node(&thumbnail_control_id("InfoBand", 0), "Panel", ""),
            node(&thumbnail_control_id("SelectionMarker", 0), "Panel", ""),
            node(&thumbnail_control_id("Name", 0), "Label", "asset"),
            node(
                &thumbnail_control_id("NameContinuation", 0),
                "Label",
                "asset",
            ),
            node(&thumbnail_control_id("TypeBadge", 0), "Panel", ""),
            node(&thumbnail_control_id("Type", 0), "Label", "UI"),
            node(&thumbnail_control_id("Meta", 0), "Label", "Ready"),
        ]
        .to_vec();

        apply_compact_thumbnail_grid_layout(&mut nodes, 10.0, 20.0, 0.0, 120.0);

        for part in THUMBNAIL_CARD_LAYOUT_PARTS {
            let frame = node_frame(&nodes, &thumbnail_control_id(part, 0))
                .expect("thumbnail part should remain in the node model");
            assert_eq!(frame.width, 0.0, "{part} width should collapse");
            assert_eq!(frame.height, 0.0, "{part} height should collapse");
        }
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
}

fn thumbnail_meta_row_offset_y(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        THUMBNAIL_META_ROW_STACKED_OFFSET_Y
    } else {
        THUMBNAIL_META_ROW_SINGLE_OFFSET_Y
    }
}

fn thumbnail_info_band_height(continuation_height: f32) -> f32 {
    if continuation_height > 0.0 {
        THUMBNAIL_INFO_BAND_STACKED_HEIGHT
    } else {
        THUMBNAIL_INFO_BAND_SINGLE_LINE_HEIGHT
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

fn set_node_value_number(nodes: &mut [ViewTemplateNodeData], control_id: &str, value: f32) {
    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == control_id)
    {
        node.value_number = value;
    }
}
