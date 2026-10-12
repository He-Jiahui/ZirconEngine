use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::super::template_nodes::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
};
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

pub(super) fn paint_nodes(width: u32, height: u32, nodes: Vec<TemplatePaneNodeData>) -> Vec<u8> {
    paint_template_nodes_for_test(width, height, model_rc(nodes))
}

pub(super) fn paint_nodes_with_background(
    width: u32,
    height: u32,
    background: [u8; 4],
    nodes: Vec<TemplatePaneNodeData>,
) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(width, height, background, model_rc(nodes))
}

pub(super) fn styled_node(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    background: [u8; 4],
) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Pane".into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    };
    node.button_style.element.background_color = Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
        background[0],
        background[1],
        background[2],
        background[3],
    )));
    node
}

pub(super) fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

pub(super) fn luma(pixel: [u8; 4]) -> u16 {
    pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16
}
