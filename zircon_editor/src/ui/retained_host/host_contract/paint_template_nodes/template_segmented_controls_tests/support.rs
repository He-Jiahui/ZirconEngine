use super::super::super::super::data::{FrameRect, TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::{Color, SharedString};

pub(super) fn segmented_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchInputSegmented".into(),
        role: "Mount".into(),
        component_role: "".into(),
        value_text: "center".into(),
        options: model_rc(vec![
            SharedString::from("left"),
            SharedString::from("center"),
            SharedString::from("right"),
        ]),
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 8.0,
            width: 150.0,
            height: 30.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn labeled_segmented_node() -> TemplatePaneNodeData {
    let mut node = segmented_node();
    node.label_text = "Segmented Control".into();
    node.label_color = Color::from_rgb_u8(161, 172, 178);
    node.label_brightness = 0.94;
    node.layout_offset_x = 6.0;
    node.frame = TemplateNodeFrameData {
        x: 12.0,
        y: 4.0,
        width: 150.0,
        height: 48.0,
    };
    node
}

pub(super) fn tab_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchDrawerTabComponents".into(),
        role: "Mount".into(),
        text: "UI Components".into(),
        checked: true,
        selected: true,
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 4.0,
            width: 150.0,
            height: 40.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn frame_rect(frame: &TemplateNodeFrameData) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
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
