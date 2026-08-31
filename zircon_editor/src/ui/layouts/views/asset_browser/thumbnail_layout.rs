use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::asset_content_layout::{
    asset_thumbnail_card_geometry, AssetContentRect, AssetThumbnailGridMetrics,
    BrowserThumbnailNodeRole, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

use super::thumbnail_nodes::{
    compact_thumbnail_file_name_to_width, thumbnail_node_identity, ThumbnailNodeKind,
};

const THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
#[cfg(test)]
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
    nodes
        .iter()
        .any(|node| node.control_id == BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID)
}

pub(super) fn apply_thumbnail_grid_logical_extent(
    nodes: &mut [ViewTemplateNodeData],
    logical_item_count: usize,
) {
    let Some(grid) = nodes
        .iter_mut()
        .find(|node| node.control_id == BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID)
    else {
        return;
    };
    grid.value_number =
        AssetThumbnailGridMetrics::new(grid.frame.width, logical_item_count).content_extent();
}

#[derive(Clone, Copy, Default)]
struct ThumbnailLayoutInput {
    has_name_continuation: bool,
    type_label_width: f32,
}

#[derive(Clone)]
struct ThumbnailCardFrames {
    card: ViewTemplateFrameData,
    visual: ViewTemplateFrameData,
    info_band: ViewTemplateFrameData,
    selection_marker: ViewTemplateFrameData,
    name: ViewTemplateFrameData,
    name_continuation: ViewTemplateFrameData,
    type_badge: ViewTemplateFrameData,
    type_label: ViewTemplateFrameData,
    meta: ViewTemplateFrameData,
}

impl ThumbnailCardFrames {
    fn for_kind(&self, kind: ThumbnailNodeKind) -> ViewTemplateFrameData {
        match kind {
            ThumbnailNodeKind::Card => &self.card,
            ThumbnailNodeKind::Visual => &self.visual,
            ThumbnailNodeKind::InfoBand => &self.info_band,
            ThumbnailNodeKind::SelectionMarker => &self.selection_marker,
            ThumbnailNodeKind::Name => &self.name,
            ThumbnailNodeKind::NameContinuation => &self.name_continuation,
            ThumbnailNodeKind::TypeBadge => &self.type_badge,
            ThumbnailNodeKind::Type => &self.type_label,
            ThumbnailNodeKind::Meta => &self.meta,
        }
        .clone()
    }
}

pub(super) fn apply_compact_thumbnail_grid_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let count = thumbnail_card_count(nodes);
    let layout_inputs = thumbnail_layout_inputs(nodes, count);
    let metrics = AssetThumbnailGridMetrics::new(width, count);
    let grid_height = if count == 0 { 0.0 } else { height };
    let content_extent = metrics.content_extent();

    for node in nodes.iter_mut() {
        if node.control_id == BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID {
            node.frame = ViewTemplateFrameData {
                x,
                y,
                width,
                height: grid_height,
            };
            node.value_number = content_extent;
            continue;
        }
        let Some((kind, index)) = thumbnail_node_identity(node.control_id.as_str()) else {
            continue;
        };
        let Some(input) = layout_inputs.get(index).copied() else {
            node.frame = ViewTemplateFrameData::default();
            continue;
        };
        let Some(frames) = thumbnail_card_frames(metrics, index, x, y, input) else {
            node.frame = ViewTemplateFrameData::default();
            continue;
        };
        node.frame = frames.for_kind(kind);
        if kind == ThumbnailNodeKind::Name && !node.value_text.is_empty() {
            node.text =
                compact_thumbnail_file_name_to_width(node.value_text.as_str(), frames.name.width)
                    .into();
        }
    }
}

fn thumbnail_card_count(nodes: &[ViewTemplateNodeData]) -> usize {
    nodes
        .iter()
        .filter_map(|node| thumbnail_node_identity(node.control_id.as_str()))
        .filter_map(|(kind, index)| (kind == ThumbnailNodeKind::Card).then_some(index + 1))
        .max()
        .unwrap_or(0)
}

fn thumbnail_layout_inputs(
    nodes: &[ViewTemplateNodeData],
    count: usize,
) -> Vec<ThumbnailLayoutInput> {
    let mut inputs = vec![ThumbnailLayoutInput::default(); count];
    for node in nodes {
        let Some((kind, index)) = thumbnail_node_identity(node.control_id.as_str()) else {
            continue;
        };
        let Some(input) = inputs.get_mut(index) else {
            continue;
        };
        match kind {
            ThumbnailNodeKind::NameContinuation => {
                input.has_name_continuation = !node.text.is_empty();
            }
            ThumbnailNodeKind::Type => {
                input.type_label_width = measure_runtime_text_width(
                    node.text.as_str(),
                    THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE,
                );
            }
            _ => {}
        }
    }
    inputs
}

fn thumbnail_card_frames(
    metrics: AssetThumbnailGridMetrics,
    index: usize,
    origin_x: f32,
    origin_y: f32,
    input: ThumbnailLayoutInput,
) -> Option<ThumbnailCardFrames> {
    let item = metrics.item_frame(index)?;
    let geometry = asset_thumbnail_card_geometry(
        AssetContentRect {
            x: origin_x + item.x,
            y: origin_y + item.y,
            width: item.width,
            height: item.height,
        },
        input.has_name_continuation,
        input.type_label_width,
    );

    Some(ThumbnailCardFrames {
        card: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::Card)),
        visual: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::Visual)),
        info_band: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::InfoBand)),
        selection_marker: thumbnail_frame(
            geometry.for_role(BrowserThumbnailNodeRole::SelectionMarker),
        ),
        name: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::Name)),
        name_continuation: thumbnail_frame(
            geometry.for_role(BrowserThumbnailNodeRole::NameContinuation),
        ),
        type_badge: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::TypeBadge)),
        type_label: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::Type)),
        meta: thumbnail_frame(geometry.for_role(BrowserThumbnailNodeRole::Meta)),
    })
}

fn thumbnail_frame(rect: AssetContentRect) -> ViewTemplateFrameData {
    ViewTemplateFrameData {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

#[cfg(test)]
mod tests {
    use super::super::thumbnail_nodes::thumbnail_control_id;
    use super::*;

    #[test]
    fn thumbnail_type_badge_width_uses_runtime_text_measurement() {
        let nodes = vec![node(&thumbnail_control_id("Type", 0), "Label", "iiiiiiii")];
        let measured =
            measure_runtime_text_width(nodes[0].text.as_str(), THUMBNAIL_TYPE_BADGE_TEXT_FONT_SIZE);
        let card = AssetContentRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 160.0,
        };
        let narrow = asset_thumbnail_card_geometry(card, false, 0.0)
            .for_role(BrowserThumbnailNodeRole::TypeBadge);
        let measured = asset_thumbnail_card_geometry(card, false, measured)
            .for_role(BrowserThumbnailNodeRole::TypeBadge);

        assert!(measured.width >= narrow.width);
        assert!(measured.width <= 48.0);
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
        let geometry = asset_thumbnail_card_geometry(
            AssetContentRect {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 160.0,
            },
            true,
            0.0,
        );
        let name = geometry.for_role(BrowserThumbnailNodeRole::Name);
        let continuation = geometry.for_role(BrowserThumbnailNodeRole::NameContinuation);

        assert!(continuation.height > 0.0);
        assert!(continuation.y >= name.y + name.height);
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

    #[test]
    fn ten_thousand_thumbnail_cards_use_the_same_indexed_layout_path() {
        let mut nodes = Vec::with_capacity(10_001);
        nodes.push(node(BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID, "Panel", ""));
        nodes.extend(
            (0..10_000).map(|index| node(&thumbnail_control_id("Card", index), "Panel", "")),
        );

        apply_compact_thumbnail_grid_layout(&mut nodes, 0.0, 0.0, 900.0, 620.0);

        let grid = node_frame(&nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID)
            .expect("thumbnail grid should remain materialized");
        let first = node_frame(&nodes, &thumbnail_control_id("Card", 0))
            .expect("first thumbnail card should receive a frame");
        let last = node_frame(&nodes, &thumbnail_control_id("Card", 9_999))
            .expect("last thumbnail card should receive a frame");
        assert!(grid.height > 0.0);
        assert!(last.y > first.y);
        assert!(nodes[0].value_number > last.y + last.height);
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

    fn node_frame(
        nodes: &[ViewTemplateNodeData],
        control_id: &str,
    ) -> Option<ViewTemplateFrameData> {
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
}
