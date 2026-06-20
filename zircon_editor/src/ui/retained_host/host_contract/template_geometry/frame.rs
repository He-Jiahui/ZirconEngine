use super::super::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract) fn frame_from_template_node(
    node: &TemplatePaneNodeData,
) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}
