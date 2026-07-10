use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_geometry::{
    frame_from_template, intersect, translated,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;

use super::identity::{activity_content_identity, ActivityContentNodeIdentity};

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
