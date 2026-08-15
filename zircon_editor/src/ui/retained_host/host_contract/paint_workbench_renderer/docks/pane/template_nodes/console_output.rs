use std::rc::Rc;

use crate::ui::retained_host::console_output::{
    ConsoleOutputPaintMetadata, CONSOLE_OUTPUT_LINE_PREFIX, CONSOLE_OUTPUT_SEVERITY_PREFIX,
};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_geometry::{
    frame_from_template, intersect, translated,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::host_contract::paint_workbench_renderer::native_panes::draw_vertical_scrollbar;
use crate::ui::retained_host::primitives::ModelRc;

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes)
struct ConsoleOutputProjector
{
    metadata: Rc<ConsoleOutputPaintMetadata>,
    origin_x: f32,
    origin_y: f32,
    output_clip: FrameRect,
    scroll_px: f32,
}

impl ConsoleOutputProjector {
    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn new(
        nodes: &ModelRc<TemplatePaneNodeData>,
        origin: &FrameRect,
        interaction: &HostPaneInteractionStateData,
    ) -> Option<Self> {
        let metadata = nodes.metadata_rc::<ConsoleOutputPaintMetadata>()?;
        let viewport = metadata.viewport();
        let output_clip = translated(
            &FrameRect {
                x: viewport.x,
                y: viewport.y,
                width: viewport.width,
                height: viewport.height,
            },
            origin.x,
            origin.y,
        );
        (output_clip.width > 0.0 && output_clip.height > 0.0).then_some(Self {
            metadata,
            origin_x: origin.x,
            origin_y: origin.y,
            output_clip,
            scroll_px: interaction.console_scroll_px.max(0.0),
        })
    }

    pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes) fn draw_scrollbar(
        &self,
        frame: &mut HostRgbaFrame,
        clip: &FrameRect,
    ) -> bool {
        draw_vertical_scrollbar(
            frame,
            &self.output_clip,
            clip,
            self.scroll_px,
            self.metadata.content_extent(),
            false,
        )
    }
}

impl TemplateNodePaintTransform for ConsoleOutputProjector {
    fn row_visit_indices(&self, row_count: usize, _clip: &FrameRect) -> Option<Vec<usize>> {
        Some(self.metadata.visible_node_rows(row_count, self.scroll_px))
    }

    fn transform(
        &self,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        if !node.control_id.starts_with(CONSOLE_OUTPUT_LINE_PREFIX)
            && !node.control_id.starts_with(CONSOLE_OUTPUT_SEVERITY_PREFIX)
        {
            return Some((node, clip));
        }

        node.frame.y -= self.scroll_px;
        let viewport = self.metadata.viewport();
        node.has_clip_frame = true;
        node.clip_frame = TemplateNodeFrameData {
            x: viewport.x,
            y: viewport.y,
            width: viewport.width,
            height: viewport.height,
        };

        let output_clip = intersect(&clip, &self.output_clip)?;
        let node_frame = translated(
            &frame_from_template(&node.frame),
            self.origin_x,
            self.origin_y,
        );
        intersect(&node_frame, &output_clip)?;
        Some((node, output_clip))
    }
}

#[cfg(test)]
#[path = "console_output/tests.rs"]
mod tests;
