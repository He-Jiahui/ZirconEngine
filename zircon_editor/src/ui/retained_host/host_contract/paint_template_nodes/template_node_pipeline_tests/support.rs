use super::super::super::super::data::{
    FrameRect, TemplateNodeFrameData, TemplatePaneMenuItemData, TemplatePaneNodeData,
    TemplatePaneOptionData,
};

pub(super) fn panel_node(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn dropdown_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "Dropdown".into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 12.0,
            width: 112.0,
            height: 28.0,
        },
        structured_options: crate::ui::layouts::common::model_rc(vec![
            option("dropdown", true, false, false, false),
            option("option_a", false, true, true, false),
            option("option_b", false, false, false, true),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn dropdown_near_bottom_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "Dropdown".into(),
        role: "Dropdown".into(),
        component_role: "dropdown".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 20.0,
            y: 120.0,
            width: 100.0,
            height: 28.0,
        },
        structured_options: crate::ui::layouts::common::model_rc(vec![
            option("dropdown", true, false, false, false),
            option("option_a", false, true, true, false),
            option("option_b", false, false, false, false),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn popup_menu_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: "PopupMenu".into(),
        role: "Menu".into(),
        component_role: "menu".into(),
        popup_open: true,
        frame: TemplateNodeFrameData {
            x: 16.0,
            y: 16.0,
            width: 128.0,
            height: 96.0,
        },
        structured_menu_items: crate::ui::layouts::common::model_rc(vec![
            menu_item("New", false, false, false),
            menu_item("Open", false, false, false),
            menu_item("Save", true, false, false),
            menu_item("", false, true, false),
            menu_item("Delete", false, false, true),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

fn option(
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

fn menu_item(
    action_id: &str,
    checked: bool,
    separator: bool,
    hovered: bool,
) -> TemplatePaneMenuItemData {
    TemplatePaneMenuItemData {
        action_id: action_id.into(),
        label: action_id.into(),
        checked,
        separator,
        disabled: separator,
        hovered,
        ..TemplatePaneMenuItemData::default()
    }
}

pub(super) fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
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
