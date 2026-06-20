use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::draw::draw_template_nodes;

pub(crate) fn paint_template_nodes_for_test(
    width: u32,
    height: u32,
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(width, height, [0, 0, 0, 255], nodes)
}

pub(crate) fn paint_template_nodes_for_test_with_background(
    width: u32,
    height: u32,
    background: [u8; 4],
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, background);
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    draw_template_nodes(&mut frame, &nodes, &bounds, &bounds, None);
    frame.into_bytes()
}
