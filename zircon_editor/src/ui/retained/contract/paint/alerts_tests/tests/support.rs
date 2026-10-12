use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};

pub(super) fn alert_node(control_id: &str, text: &str, tone: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Alert".into(),
        component_role: "alert".into(),
        text: text.into(),
        validation_level: tone.into(),
        icon_name: tone.into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 160.0,
            height: 32.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn positioned_alert_node(
    control_id: &str,
    text: &str,
    tone: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..alert_node(control_id, text, tone)
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

pub(super) fn blend_over(source: [u8; 4], destination: [u8; 4]) -> [u8; 4] {
    let alpha = source[3] as u32;
    let inverse = 255 - alpha;
    [
        ((source[0] as u32 * alpha + destination[0] as u32 * inverse) / 255) as u8,
        ((source[1] as u32 * alpha + destination[1] as u32 * inverse) / 255) as u8,
        ((source[2] as u32 * alpha + destination[2] as u32 * inverse) / 255) as u8,
        255,
    ]
}
