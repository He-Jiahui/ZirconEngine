use super::super::super::data::{
    TemplateNodeFrameData, TemplatePaneMenuItemData, TemplatePaneNodeData, TemplatePaneOptionData,
};
use super::super::style_selector::WORKBENCH_POPUP_ROW_DANGER_TEXT as POPUP_ROW_DANGER_TEXT;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::super::template_popup_row_adornments::menu_item_flag_value;
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn menu_item_adornment_kind_reads_icon_danger_and_submenu_flags() {
    let delete = menu_item("Delete|danger,icon=trash", false, false, false);
    let more = menu_item("More Tools|submenu", false, false, false);
    let save = menu_item("Save", false, false, false);

    assert!(menu_item_has_flag(&delete, "danger"));
    assert_eq!(
        menu_item_flag_value(&delete, "icon").as_deref(),
        Some("trash")
    );
    assert_eq!(
        menu_row_adornment_kind(&delete),
        Some(PopupRowAdornmentKind::Trash)
    );
    assert_eq!(
        menu_row_adornment_kind(&more),
        Some(PopupRowAdornmentKind::Chevron)
    );
    assert_eq!(
        menu_row_adornment_kind(&save),
        Some(PopupRowAdornmentKind::Save)
    );
    assert_eq!(popup_menu_row_style(&delete).text, POPUP_ROW_DANGER_TEXT);
    assert_eq!(
        popup_menu_row_style(&delete).adornment,
        POPUP_ROW_DANGER_TEXT
    );
}

#[test]
fn popup_row_style_selector_resolves_state_priority_for_options_and_menu_items() {
    let disabled_pressed = TemplatePaneOptionData {
        pressed: true,
        ..option("disabled", false, false, false, true)
    };
    let focused_selected = option("selected", true, false, false, false);
    let checked_pressed = TemplatePaneMenuItemData {
        pressed: true,
        ..menu_item("Checked", true, false, false)
    };

    let disabled = popup_option_row_style(&disabled_pressed);
    assert_eq!(
        disabled.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
    );
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.selection_mark, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    let focused = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        ..focused_selected
    });
    assert_eq!(
        focused.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Focused
    );
    assert_eq!(focused.background, Some(PALETTE.surface_selected));
    assert_eq!(focused.selection_mark, Some(PALETTE.focus_ring));
    assert_eq!(focused.text, PALETTE.focus_ring);

    let checked = popup_menu_row_style(&checked_pressed);
    assert_eq!(
        checked.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(checked.background, Some(PALETTE.surface_selected));
    assert_eq!(checked.selection_mark, Some(PALETTE.focus_ring));
    assert_eq!(checked.adornment, PALETTE.focus_ring);
}

#[test]
fn popup_row_style_selector_matches_runtime_extract_state_matrix_for_projected_rows() {
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    let selected = popup_option_row_style(&option("selected", true, false, false, false));
    assert_eq!(selected.state, UiPainterResolvedState::Selected);
    assert_eq!(selected.background, Some(PALETTE.surface_selected));
    assert_eq!(selected.selection_mark, Some(PALETTE.focus_ring));

    let focused = popup_option_row_style(&TemplatePaneOptionData {
        focused: true,
        hovered: true,
        ..option("focused", false, false, false, false)
    });
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, Some(PALETTE.surface_selected));
    assert_eq!(focused.text, PALETTE.focus_ring);

    let disabled = popup_option_row_style(&TemplatePaneOptionData {
        selected: true,
        disabled: true,
        ..option("disabled", false, false, false, false)
    });
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.selection_mark, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);

    let loading = popup_option_row_style(&TemplatePaneOptionData {
        selected: true,
        special: true,
        hovered: true,
        pressed: true,
        loading: true,
        ..option("loading", false, false, false, false)
    });
    assert_eq!(loading.state, UiPainterResolvedState::Loading);
    assert_eq!(loading.background, None);
    assert_eq!(loading.selection_mark, None);
    assert_eq!(loading.text, PALETTE.text_disabled);

    let raw_loading_menu = popup_menu_row_style(&menu_item(
        "Archive|loading,checked,hovered",
        true,
        false,
        true,
    ));
    assert_eq!(raw_loading_menu.state, UiPainterResolvedState::Loading);
    assert_eq!(raw_loading_menu.background, None);
    assert_eq!(raw_loading_menu.selection_mark, None);
    assert_eq!(raw_loading_menu.text, PALETTE.text_disabled);

    let projected_loading_menu = popup_menu_row_style(&TemplatePaneMenuItemData {
        checked: true,
        hovered: true,
        pressed: true,
        loading: true,
        ..menu_item("Archive", false, false, false)
    });
    assert_eq!(
        projected_loading_menu.state,
        UiPainterResolvedState::Loading
    );
    assert_eq!(projected_loading_menu.background, None);
    assert_eq!(projected_loading_menu.selection_mark, None);
    assert_eq!(projected_loading_menu.adornment, PALETTE.text_disabled);
}

#[test]
fn open_popup_menu_paints_right_aligned_item_icons() {
    let bytes = paint_template_nodes_for_test(180, 180, model_rc(vec![popup_menu_node()]));

    assert!(changed_pixel_count(&bytes, 180, 112, 16, 24, 24) > 0);
    assert_eq!(pixel_at(&bytes, 180, 119, 113), POPUP_ROW_DANGER_TEXT);
    assert!(changed_pixel_count(&bytes, 180, 112, 136, 24, 24) > 0);
}

#[test]
fn selected_dropdown_option_paints_right_check_adornment() {
    let bytes = paint_template_nodes_for_test(150, 120, model_rc(vec![dropdown_node()]));

    assert!(changed_pixel_count(&bytes, 150, 96, 50, 22, 22) > 0);
}

#[test]
fn standalone_dropdown_popup_paints_rows_inside_projected_popup_frame() {
    let bytes = paint_template_nodes_for_test(180, 140, model_rc(vec![dropdown_popup_node()]));

    assert_eq!(pixel_at(&bytes, 180, 20, 20), PALETTE.focus_ring);
}

fn popup_menu_node() -> TemplatePaneNodeData {
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

fn dropdown_node() -> TemplatePaneNodeData {
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

fn dropdown_popup_node() -> TemplatePaneNodeData {
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
            option("focused", false, true, false, false),
            option("disabled", false, false, false, true),
            option("loading", false, false, false, false),
        ]),
        ..TemplatePaneNodeData::default()
    }
}

fn menu_item(raw: &str, checked: bool, separator: bool, hovered: bool) -> TemplatePaneMenuItemData {
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

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
