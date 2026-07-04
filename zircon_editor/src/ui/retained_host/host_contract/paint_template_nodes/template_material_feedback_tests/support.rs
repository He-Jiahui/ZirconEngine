use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};

pub(super) fn positioned_progress_node(
    control_id: &str,
    percent: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Progress".into(),
        component_role: "progress-bar".into(),
        value_percent: percent,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn positioned_backdrop_node(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Backdrop".into(),
        component_role: "backdrop".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((py_index(y, frame_width)) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

fn py_index(y: u32, frame_width: u32) -> usize {
    y as usize * frame_width as usize
}
