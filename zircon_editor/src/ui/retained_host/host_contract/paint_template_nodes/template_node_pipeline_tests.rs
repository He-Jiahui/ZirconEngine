use super::super::super::data::{
    TemplateNodeFrameData, TemplatePaneMenuItemData, TemplatePaneOptionData,
};
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn template_nodes_skip_when_active_paint_clip_misses_template_clip() {
    let mut frame = HostRgbaFrame::filled(32, 32, [1, 2, 3, 255]);
    let before = frame.as_bytes().to_vec();
    frame.replace_paint_clip(Some(rect(24.0, 24.0, 4.0, 4.0)));

    let bounds = rect(0.0, 0.0, 16.0, 16.0);
    let painted = draw_template_nodes(
        &mut frame,
        &model_rc(vec![panel_node("outside", 0.0, 0.0, 8.0, 8.0)]),
        &bounds,
        &bounds,
        None,
    );

    assert!(!painted);
    assert_eq!(frame.as_bytes(), before.as_slice());
}

#[test]
fn template_nodes_only_paint_nodes_intersecting_active_damage_clip() {
    let mut frame = HostRgbaFrame::filled(40, 20, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(rect(20.0, 0.0, 10.0, 10.0)));

    let bounds = rect(0.0, 0.0, 40.0, 20.0);
    let painted = draw_template_nodes(
        &mut frame,
        &model_rc(vec![
            panel_node("left", 0.0, 0.0, 10.0, 10.0),
            panel_node("damage", 20.0, 0.0, 10.0, 10.0),
        ]),
        &bounds,
        &bounds,
        None,
    );

    assert!(painted);
    assert_eq!(changed_pixel_count(frame.as_bytes(), 40, 0, 0, 10, 10), 0);
    assert!(changed_pixel_count(frame.as_bytes(), 40, 20, 0, 10, 10) > 0);
}

#[test]
fn template_nodes_paint_open_dropdown_option_rows_below_control() {
    let bytes = paint_template_nodes_for_test(128, 128, model_rc(vec![dropdown_node()]));

    assert!(changed_pixel_count(&bytes, 128, 12, 48, 112, 66) > 0);
}

#[test]
fn template_nodes_anchor_workbench_dropdown_popup_to_declared_layout_offset() {
    let mut node = dropdown_node();
    node.control_id = "WorkbenchInputDropdown".into();
    node.layout_offset_x = 10.0;
    node.layout_offset_y = 6.0;
    let bytes = paint_template_nodes_for_test(160, 160, model_rc(vec![node]));

    assert!(changed_pixel_count(&bytes, 160, 22, 54, 112, 66) > 0);
    assert_eq!(changed_pixel_count(&bytes, 160, 12, 44, 8, 84), 0);
}

#[test]
fn template_nodes_paint_open_dropdown_option_rows_above_control_when_below_clipped() {
    let bytes =
        paint_template_nodes_for_test(160, 160, model_rc(vec![dropdown_near_bottom_node()]));

    assert!(changed_pixel_count(&bytes, 160, 20, 32, 100, 84) > 0);
    assert_eq!(changed_pixel_count(&bytes, 160, 20, 152, 100, 8), 0);
}

#[test]
fn template_nodes_paint_open_popup_menu_rows_inside_menu_frame() {
    let bytes = paint_template_nodes_for_test(160, 128, model_rc(vec![popup_menu_node()]));

    assert!(changed_pixel_count(&bytes, 160, 16, 16, 128, 96) > 0);
}

fn panel_node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> TemplatePaneNodeData {
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

fn dropdown_node() -> TemplatePaneNodeData {
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
        structured_options: model_rc(vec![
            option("dropdown", true, false, false, false),
            option("option_a", false, true, true, false),
            option("option_b", false, false, false, true),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

fn dropdown_near_bottom_node() -> TemplatePaneNodeData {
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
        structured_options: model_rc(vec![
            option("dropdown", true, false, false, false),
            option("option_a", false, true, true, false),
            option("option_b", false, false, false, false),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

fn popup_menu_node() -> TemplatePaneNodeData {
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
        structured_menu_items: model_rc(vec![
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

fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn changed_pixel_count(
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
