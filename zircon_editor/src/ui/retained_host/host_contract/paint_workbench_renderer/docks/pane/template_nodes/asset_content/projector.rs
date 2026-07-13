use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_geometry::{
    frame_from_template, intersect, translated,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::BROWSER_CONTENT_PREVIEW_CONTROL_ID;

use super::identity::{
    activity_content_identity, browser_content_identity, ActivityContentNodeIdentity,
    BrowserContentNodeIdentity,
};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes)
struct ActivityAssetContentProjector
{
    origin_x: f32,
    origin_y: f32,
    content_clip: FrameRect,
    folder_row_count: usize,
    scroll_px: f32,
    hovered_row_index: i32,
}

impl ActivityAssetContentProjector {
    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn new(
        nodes: &ModelRc<TemplatePaneNodeData>,
        origin: &FrameRect,
        interaction: &HostPaneInteractionStateData,
    ) -> Option<Self> {
        let content_panel = (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .find(|node| {
                activity_content_identity(node.control_id.as_str())
                    == Some(ActivityContentNodeIdentity::ContentPanel)
            })?;
        let folder_row_count = (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .filter_map(|node| activity_content_identity(node.control_id.as_str()))
            .filter(|identity| {
                matches!(identity, ActivityContentNodeIdentity::Folder { .. }) && identity.is_row()
            })
            .count();

        Some(Self {
            origin_x: origin.x,
            origin_y: origin.y,
            content_clip: translated(
                &frame_from_template(&content_panel.frame),
                origin.x,
                origin.y,
            ),
            folder_row_count,
            scroll_px: interaction.activity_asset_content_scroll_px.max(0.0),
            hovered_row_index: interaction.activity_asset_content_hovered_index,
        })
    }
}

impl TemplateNodePaintTransform for ActivityAssetContentProjector {
    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        let Some(identity) = activity_content_identity(node.control_id.as_str()) else {
            return Some((node, clip));
        };
        if identity == ActivityContentNodeIdentity::ContentPanel {
            return Some((node, clip));
        }

        if identity != ActivityContentNodeIdentity::Empty {
            node.frame.y -= self.scroll_px;
            if node.has_clip_frame {
                node.clip_frame.y -= self.scroll_px;
            }
        }
        node.hovered = identity.is_row()
            && identity.shared_row_index(self.folder_row_count) == Some(self.hovered_row_index);

        let content_clip = intersect(&clip, &self.content_clip)?;
        let node_frame = translated(
            &frame_from_template(&node.frame),
            self.origin_x,
            self.origin_y,
        );
        intersect(&node_frame, &content_clip)?;
        Some((node, content_clip))
    }
}

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes)
struct BrowserAssetContentProjector
{
    origin_x: f32,
    origin_y: f32,
    row_clip: FrameRect,
    scroll_px: f32,
    hovered_row_index: i32,
    mode: BrowserAssetContentProjectionMode,
}

#[derive(Clone, Copy)]
enum BrowserAssetContentProjectionMode {
    List,
    Thumbnail,
}

impl BrowserAssetContentProjector {
    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn new(
        nodes: &ModelRc<TemplatePaneNodeData>,
        origin: &FrameRect,
        interaction: &HostPaneInteractionStateData,
    ) -> Option<Self> {
        let (row_clip, mode) = if let Some(grid) =
            find_browser_node(nodes, BrowserContentNodeIdentity::ThumbnailGrid)
        {
            (
                translated(&frame_from_template(&grid.frame), origin.x, origin.y),
                BrowserAssetContentProjectionMode::Thumbnail,
            )
        } else {
            let table = find_browser_node(nodes, BrowserContentNodeIdentity::TablePanel)?;
            let header = find_browser_node(nodes, BrowserContentNodeIdentity::Header)?;
            let table_frame = translated(&frame_from_template(&table.frame), origin.x, origin.y);
            let header_bottom = origin.y + header.frame.y + header.frame.height;
            let rows_bottom = find_browser_control_node(nodes, BROWSER_CONTENT_PREVIEW_CONTROL_ID)
                .map(|preview| origin.y + preview.frame.y)
                .unwrap_or(table_frame.y + table_frame.height)
                .min(table_frame.y + table_frame.height);
            (
                FrameRect {
                    x: table_frame.x,
                    y: header_bottom,
                    width: table_frame.width,
                    height: (rows_bottom - header_bottom).max(0.0),
                },
                BrowserAssetContentProjectionMode::List,
            )
        };
        if row_clip.width <= 0.0 || row_clip.height <= 0.0 {
            return None;
        }

        Some(Self {
            origin_x: origin.x,
            origin_y: origin.y,
            row_clip,
            scroll_px: interaction.browser_asset_content_scroll_px.max(0.0),
            hovered_row_index: interaction.browser_asset_content_hovered_index,
            mode,
        })
    }
}

impl TemplateNodePaintTransform for BrowserAssetContentProjector {
    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        let Some(identity) = browser_content_identity(node.control_id.as_str()) else {
            return Some((node, clip));
        };
        let (index, paints_hover) = match (self.mode, identity) {
            (
                BrowserAssetContentProjectionMode::List,
                BrowserContentNodeIdentity::Row { index },
            ) => (index, true),
            (
                BrowserAssetContentProjectionMode::Thumbnail,
                BrowserContentNodeIdentity::Thumbnail { index, role },
            ) => (index, role.paints_hover()),
            _ => return Some((node, clip)),
        };

        node.frame.y -= self.scroll_px;
        if node.has_clip_frame {
            node.clip_frame.y -= self.scroll_px;
        }
        node.hovered = paints_hover && i32::try_from(index).ok() == Some(self.hovered_row_index);

        let row_clip = intersect(&clip, &self.row_clip)?;
        let node_frame = translated(
            &frame_from_template(&node.frame),
            self.origin_x,
            self.origin_y,
        );
        intersect(&node_frame, &row_clip)?;
        Some((node, row_clip))
    }
}

fn find_browser_node(
    nodes: &ModelRc<TemplatePaneNodeData>,
    identity: BrowserContentNodeIdentity,
) -> Option<TemplatePaneNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| browser_content_identity(node.control_id.as_str()) == Some(identity))
}

fn find_browser_control_node(
    nodes: &ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> Option<TemplatePaneNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.rsplit('/').next() == Some(control_id))
}
