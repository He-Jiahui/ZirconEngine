use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::frame_geometry::{union_frame, visible_frame};
use super::frame::frame_from_template_node;
use crate::ui::retained_host::primitives::ModelRc;

pub(in crate::ui::retained_host::host_contract) fn template_popup_bounds(
    native_window_bounds: &FrameRect,
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> FrameRect {
    if visible_frame(native_window_bounds) {
        return native_window_bounds.clone();
    }
    template_nodes_bounds(nodes).unwrap_or_else(|| FrameRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    })
}

pub(in crate::ui::retained_host::host_contract) fn template_nodes_bounds(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> Option<FrameRect> {
    let mut bounds: Option<FrameRect> = None;
    for node in nodes.iter() {
        let frame = frame_from_template_node(node);
        if !visible_frame(&frame) {
            continue;
        }
        bounds = Some(match bounds {
            Some(current) => union_frame(&current, &frame),
            None => frame,
        });
    }
    bounds
}
