use std::rc::Rc;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_geometry::{
    frame_from_template, intersect, translated,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    ActivityContentNodeIdentity, AssetContentNodeIdentity, AssetContentPaintMetadata,
    AssetContentRect, AssetContentSurface, BrowserContentNodeIdentity,
};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes)
struct ActivityAssetContentProjector
{
    metadata: Rc<AssetContentPaintMetadata>,
    origin_x: f32,
    origin_y: f32,
    content_clip: FrameRect,
    scroll_px: f32,
    hovered_row_index: i32,
}

impl ActivityAssetContentProjector {
    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn new(
        nodes: &ModelRc<TemplatePaneNodeData>,
        origin: &FrameRect,
        interaction: &HostPaneInteractionStateData,
    ) -> Option<Self> {
        let metadata = nodes.metadata_rc::<AssetContentPaintMetadata>()?;
        if metadata.surface() != AssetContentSurface::Activity {
            return None;
        }
        let content_clip = translated_asset_content_rect(metadata.viewport()?, origin);

        Some(Self {
            metadata,
            origin_x: origin.x,
            origin_y: origin.y,
            content_clip,
            scroll_px: interaction.activity_asset_content_scroll_px.max(0.0),
            hovered_row_index: interaction.activity_asset_content_hovered_index,
        })
    }
}

impl TemplateNodePaintTransform for ActivityAssetContentProjector {
    fn row_visit_indices(&self, _row_count: usize, clip: &FrameRect) -> Option<Vec<usize>> {
        Some(self.metadata.visible_node_rows(
            self.scroll_px,
            self.origin_x,
            self.origin_y,
            asset_content_rect(clip),
        ))
    }

    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        let Some(AssetContentNodeIdentity::Activity(identity)) =
            self.metadata.identity(node.control_id.as_str())
        else {
            return Some((node, clip));
        };
        if identity == ActivityContentNodeIdentity::ContentPanel {
            return Some((node, clip));
        }

        if self.metadata.is_scroll_node(node.control_id.as_str()) {
            node.frame.y -= self.scroll_px;
            if node.has_clip_frame {
                node.clip_frame.y -= self.scroll_px;
            }
        }
        node.hovered = identity.is_row()
            && identity.shared_row_index(self.metadata.folder_row_count())
                == Some(self.hovered_row_index);

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
    metadata: Rc<AssetContentPaintMetadata>,
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
        let metadata = nodes.metadata_rc::<AssetContentPaintMetadata>()?;
        if metadata.surface() != AssetContentSurface::Browser {
            return None;
        }
        let row_clip = translated_asset_content_rect(metadata.viewport()?, origin);
        if row_clip.width <= 0.0 || row_clip.height <= 0.0 {
            return None;
        }
        let mode = if metadata.browser_uses_thumbnails() {
            BrowserAssetContentProjectionMode::Thumbnail
        } else {
            BrowserAssetContentProjectionMode::List
        };

        Some(Self {
            metadata,
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
    fn row_visit_indices(&self, _row_count: usize, clip: &FrameRect) -> Option<Vec<usize>> {
        Some(self.metadata.visible_node_rows(
            self.scroll_px,
            self.origin_x,
            self.origin_y,
            asset_content_rect(clip),
        ))
    }

    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        let Some(AssetContentNodeIdentity::Browser(identity)) =
            self.metadata.identity(node.control_id.as_str())
        else {
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

fn translated_asset_content_rect(rect: AssetContentRect, origin: &FrameRect) -> FrameRect {
    FrameRect {
        x: origin.x + rect.x,
        y: origin.y + rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn asset_content_rect(rect: &FrameRect) -> AssetContentRect {
    AssetContentRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}
