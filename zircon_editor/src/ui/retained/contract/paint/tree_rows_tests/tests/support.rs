use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};

pub(super) fn tree_node(
    control_id: &str,
    role: &str,
    component_role: &str,
    text: &str,
    depth: i32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        component_role: component_role.into(),
        text: text.into(),
        tree_depth: depth,
        tree_indent_px: if selected { 40.0 } else { 0.0 },
        selected,
        checked: selected,
        expanded: !text.contains("Player"),
        frame: TemplateNodeFrameData {
            x: 4.0,
            y: 6.0,
            width: 268.0,
            height: 30.0,
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
