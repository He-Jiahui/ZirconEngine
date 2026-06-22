use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn table_node(control_id: &str, selected: bool) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Table".into(),
        options: model_rc(vec![
            SharedString::from("Item_02"),
            SharedString::from("Material"),
            SharedString::from("512 KB"),
            SharedString::from("10m ago"),
        ]),
        selected,
        frame: TemplateNodeFrameData {
            x: 4.0,
            y: 4.0,
            width: 232.0,
            height: 28.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn different_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    reference: [u8; 4],
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            if pixel_at(bytes, frame_width, px, py) != reference {
                changed += 1;
            }
        }
    }
    changed
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
