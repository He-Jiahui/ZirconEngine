use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};

pub(super) fn tooltip_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchTooltipRoot".into(),
        role: "Tooltip".into(),
        component_role: "tooltip".into(),
        surface_variant: "workbench-tooltip".into(),
        text: "Tooltip".into(),
        label_text: "This is a tooltip".into(),
        layout_icon_size: 18.0,
        layout_content_offset_y: 56.0,
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 8.0,
            width: 110.0,
            height: 78.0,
        },
        ..TemplatePaneNodeData::default()
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
    for row in y..(y + height) {
        for column in x..(x + width) {
            if pixel_at(bytes, frame_width, column, row) != [0, 0, 0, 255] {
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
