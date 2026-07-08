use super::identity::is_table_tail;
use super::palette::{
    workbench_table_row_palette_from_host, WORKBENCH_TABLE_HOVER_BG, WORKBENCH_TABLE_ROW_BG,
};
use super::selection::select_workbench_table_row_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{project_host_palette, PALETTE};
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn table_row_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchTableSelected".into();
    node.hovered = true;
    node.selected = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(170, 181, 186);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(13, 65, 73, 255)));

    let style = select_workbench_table_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.background, PALETTE.surface_disabled);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
    assert_eq!(style.action, PALETTE.text_disabled);
    assert_eq!(style.text_for_cell(0), PALETTE.text_disabled);
    assert_eq!(style.text_for_cell(3), PALETTE.text_disabled);
}

#[test]
fn selected_focused_table_row_uses_muted_selected_fill_and_neutral_outline() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchTableSelected".into();
    node.selected = true;
    node.focused = true;

    let style = select_workbench_table_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, PALETTE.surface_pressed);
    assert_ne!(style.background, PALETTE.surface_selected);
    assert_eq!(style.border, Some(PALETTE.border));
    assert_ne!(style.border, Some(PALETTE.accent));
    assert_ne!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, 1.0);
}

#[test]
fn focused_unmarked_table_row_keeps_keyboard_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchTableRowFocus".into();
    node.focused = true;

    let style = select_workbench_table_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.background, WORKBENCH_TABLE_ROW_BG);
    assert_ne!(style.background, WORKBENCH_TABLE_HOVER_BG);
    assert_eq!(style.border, Some(PALETTE.focus_ring));
    assert_eq!(style.border_width, 1.0);
}

#[test]
fn hovered_unmarked_table_row_still_uses_hover_background_without_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.control_id = "WorkbenchTableRowHover".into();
    node.hovered = true;

    let style = select_workbench_table_row_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Hovered);
    assert_eq!(style.background, WORKBENCH_TABLE_HOVER_BG);
    assert_eq!(style.border, None);
    assert_eq!(style.border_width, 0.0);
}

#[test]
fn table_header_and_tail_use_recessed_table_surface() {
    let mut header = TemplatePaneNodeData::default();
    header.control_id = "WorkbenchTableHeader".into();
    let header_style = select_workbench_table_row_style(&header);

    let mut tail = TemplatePaneNodeData::default();
    tail.control_id = "WorkbenchTableTail".into();
    let tail_style = select_workbench_table_row_style(&tail);

    assert!(is_table_tail(&tail));
    assert_eq!(header_style.background, WORKBENCH_TABLE_ROW_BG);
    assert_eq!(tail_style.background, WORKBENCH_TABLE_ROW_BG);
    assert_eq!(header_style.background, PALETTE.surface_inset);
    assert_eq!(tail_style.background, PALETTE.surface_inset);
}

#[test]
fn table_row_palette_projects_surface_text_and_focus_roles_from_host_palette() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(9, 12, 15, 255);
    tokens.palette.surface[3] = UiRgbaColor::from_u8(30, 35, 39, 255);
    tokens.palette.surface_disabled = UiRgbaColor::from_u8(20, 24, 28, 255);
    tokens.palette.separator_soft = UiRgbaColor::from_u8(41, 46, 50, 255);
    tokens.palette.text_secondary = UiRgbaColor::from_u8(170, 180, 186, 255);
    tokens.palette.text_primary = UiRgbaColor::from_u8(221, 229, 233, 255);
    tokens.palette.text_disabled = UiRgbaColor::from_u8(112, 121, 126, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(78, 142, 155, 255);

    let palette = workbench_table_row_palette_from_host(project_host_palette(&tokens));

    assert_eq!(palette.row_bg, [9, 12, 15, 255]);
    assert_eq!(palette.header_bg, [9, 12, 15, 255]);
    assert_eq!(palette.tail_bg, [9, 12, 15, 255]);
    assert_eq!(palette.selected_bg, [30, 35, 39, 255]);
    assert_eq!(palette.hover_bg, [30, 35, 39, 255]);
    assert_eq!(palette.separator, [41, 46, 50, 255]);
    assert_eq!(palette.action_muted, [112, 121, 126, 255]);
    assert_eq!(palette.header_text, [170, 180, 186, 255]);
    assert_eq!(palette.tail_value_text, [170, 180, 186, 255]);
    assert_eq!(palette.surface_disabled, [20, 24, 28, 255]);
    assert_eq!(palette.surface_pressed, [30, 35, 39, 255]);
    assert_eq!(palette.text, [221, 229, 233, 255]);
    assert_eq!(palette.text_muted, [170, 180, 186, 255]);
    assert_eq!(palette.text_disabled, [112, 121, 126, 255]);
    assert_eq!(palette.focus_ring, [78, 142, 155, 255]);
}
