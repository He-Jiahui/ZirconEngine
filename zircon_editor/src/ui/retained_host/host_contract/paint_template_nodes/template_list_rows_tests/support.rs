use super::super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};

pub(super) fn list_node(selected: bool, disabled: bool) -> TemplatePaneNodeData {
    list_node_with_flags(selected, selected, disabled)
}

pub(super) fn list_node_with_flags(
    selected: bool,
    checked: bool,
    disabled: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: if disabled {
            "WorkbenchListDisabled".into()
        } else if checked {
            "WorkbenchListChecked".into()
        } else {
            "WorkbenchListSelected".into()
        },
        role: "ListRow".into(),
        component_role: "list-row".into(),
        text: if disabled {
            "Disabled item".into()
        } else {
            "Selected item".into()
        },
        selected,
        checked,
        disabled,
        frame: TemplateNodeFrameData {
            x: 4.0,
            y: 4.0,
            width: 148.0,
            height: 32.0,
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
