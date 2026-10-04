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
    let mut card_frames = vec![None; count];

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
        let frames = card_frames[index]
            .get_or_insert_with(|| thumbnail_card_frames(metrics, index, x, y, input));
        let Some(frames) = frames.as_ref() else {
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
#[path = "tests/thumbnail_layout.rs"]
mod tests;
