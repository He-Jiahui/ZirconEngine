use super::super::super::super::data::{
    TemplateNodeFrameData, TemplatePaneMenuItemData, TemplatePaneNodeData, TemplatePaneOptionData,
};
use crate::ui::layouts::common::model_rc;

pub(super) fn popup_menu_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchPopupMenu".into(),
        role: "Menu".into(),
        component_role: "menu".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 10.0,
            width: 130.0,
            height: 150.0,
        },
        structured_menu_items: model_rc(vec![
            menu_item("New|icon=plus", false, false, false),
            menu_item("Open|icon=folder", false, false, false),
            menu_item("Save|icon=save", false, false, false),
            menu_item("Delete|danger,hovered,icon=trash", false, false, true),
            menu_item("More Tools|submenu", false, false, false),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn dropdown_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchInputDropdown".into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 12.0,
            width: 112.0,
            height: 28.0,
        },
        structured_options: model_rc(vec![
            option("selected", true, false, false, false),
            option("disabled", false, false, false, true),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn dropdown_popup_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "WorkbenchDropdownPopup".into(),
        role: "DropdownPopup".into(),
        component_role: "dropdown-popup".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 20.0,
            y: 16.0,
            width: 120.0,
            height: 96.0,
        },
        structured_options: model_rc(vec![
            option("selected", true, false, false, false),
            TemplatePaneOptionData {
                focused: true,
                ..option("focused", false, false, false, false)
            },
            option("disabled", false, false, false, true),
            option("loading", false, false, false, false),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn menu_item(
    raw: &str,
    checked: bool,
    separator: bool,
    hovered: bool,
) -> TemplatePaneMenuItemData {
    let label = raw.split('|').next().unwrap_or_default();
    TemplatePaneMenuItemData {
        raw: raw.into(),
        action_id: label.into(),
        label: label.into(),
        checked,
        separator,
        disabled: separator,
        hovered,
        ..TemplatePaneMenuItemData::default()
    }
}

pub(super) fn option(
    id: &str,
    selected: bool,
    hovered: bool,
    special: bool,
    disabled: bool,
) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: id.into(),
        selected,
        hovered,
        special,
        disabled,
        ..TemplatePaneOptionData::default()
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

pub(super) fn matching_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    color: [u8; 4],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut matched = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] == color {
                matched += 1;
            }
        }
    }
    matched
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
