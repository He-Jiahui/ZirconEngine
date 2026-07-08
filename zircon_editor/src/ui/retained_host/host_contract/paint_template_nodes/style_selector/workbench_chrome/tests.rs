use super::model::WorkbenchChromeKind;
use super::palette::{
    workbench_chrome_palette_from_host, WORKBENCH_CHROME_CONTENT_BG, WORKBENCH_CHROME_DRAWER_BG,
    WORKBENCH_CHROME_DRAWER_BODY_BG, WORKBENCH_CHROME_MAIN_BG, WORKBENCH_CHROME_PANEL_BG,
    WORKBENCH_CHROME_RAIL_BG, WORKBENCH_CHROME_ROOT_BG, WORKBENCH_CHROME_SEPARATOR,
    WORKBENCH_CHROME_SOFT_SEPARATOR, WORKBENCH_CHROME_STATUS_BG, WORKBENCH_CHROME_STRONG_SEPARATOR,
    WORKBENCH_CHROME_TAB_BG, WORKBENCH_CHROME_TOPBAR_BG, WORKBENCH_CHROME_VIEWPORT_FRAME_BG,
};
use super::selection::select_workbench_chrome_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{project_host_palette, PALETTE};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor};

#[test]
fn chrome_selector_preserves_normal_shell_panel_colors() {
    let node = TemplatePaneNodeData::default();

    let style = select_workbench_chrome_style(&node, WorkbenchChromeKind::TopToolbar);

    assert_eq!(style.state, UiPainterResolvedState::Normal);
    assert_eq!(style.fill, Some(WORKBENCH_CHROME_TOPBAR_BG));
    assert_eq!(style.strong_separator, WORKBENCH_CHROME_STRONG_SEPARATOR);
    assert_eq!(style.separator, WORKBENCH_CHROME_SEPARATOR);
    assert_eq!(style.soft_separator, WORKBENCH_CHROME_SOFT_SEPARATOR);
}

#[test]
fn chrome_backgrounds_track_central_surface_ladder() {
    let projected = project_host_palette(&EditorDesignTokens::workbench_dark());
    let chrome = workbench_chrome_palette_from_host(projected);

    assert_eq!(WORKBENCH_CHROME_ROOT_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_TOPBAR_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_MAIN_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_RAIL_BG, projected.surface_inset);
    assert_eq!(WORKBENCH_CHROME_PANEL_BG, projected.surface);
    assert_eq!(WORKBENCH_CHROME_CONTENT_BG, projected.surface_inset);
    assert_eq!(WORKBENCH_CHROME_VIEWPORT_FRAME_BG, projected.surface_inset);
    assert_eq!(WORKBENCH_CHROME_DRAWER_BG, projected.surface);
    assert_eq!(WORKBENCH_CHROME_DRAWER_BODY_BG, projected.surface_inset);
    assert_eq!(WORKBENCH_CHROME_STATUS_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_TAB_BG, projected.surface_pressed);
    assert_eq!(WORKBENCH_CHROME_SEPARATOR, projected.border);
    assert_eq!(
        WORKBENCH_CHROME_STRONG_SEPARATOR,
        projected.separator_strong
    );
    assert_eq!(WORKBENCH_CHROME_SOFT_SEPARATOR, projected.separator_soft);
    assert_eq!(chrome.root_bg, projected.shell_background);
    assert_eq!(chrome.topbar_bg, projected.shell_background);
    assert_eq!(chrome.content_bg, projected.surface_inset);
    assert_eq!(chrome.separator, projected.border);
    assert_eq!(chrome.strong_separator, projected.separator_strong);
    assert_eq!(chrome.soft_separator, projected.separator_soft);
}

#[test]
fn chrome_palette_projects_from_host_appearance_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface[0] = UiRgbaColor::from_u8(1, 2, 3, 255);
    tokens.palette.surface[2] = UiRgbaColor::from_u8(11, 12, 13, 255);
    tokens.palette.surface[3] = UiRgbaColor::from_u8(21, 22, 23, 255);
    tokens.palette.surface_recessed = UiRgbaColor::from_u8(31, 32, 33, 255);
    tokens.palette.surface_hover = UiRgbaColor::from_u8(41, 42, 43, 255);
    tokens.palette.surface_selected = UiRgbaColor::from_u8(51, 52, 53, 255);
    tokens.palette.surface_disabled = UiRgbaColor::from_u8(61, 62, 63, 255);
    tokens.palette.border = UiRgbaColor::from_u8(71, 72, 73, 255);
    tokens.palette.border_disabled = UiRgbaColor::from_u8(81, 82, 83, 255);
    tokens.palette.separator_strong = UiRgbaColor::from_u8(91, 92, 93, 255);
    tokens.palette.separator_soft = UiRgbaColor::from_u8(101, 102, 103, 255);
    let projected = project_host_palette(&tokens);

    let chrome = workbench_chrome_palette_from_host(projected);

    assert_eq!(chrome.root_bg, [1, 2, 3, 255]);
    assert_eq!(chrome.panel_bg, [11, 12, 13, 255]);
    assert_eq!(chrome.tab_bg, [21, 22, 23, 255]);
    assert_eq!(chrome.content_bg, [31, 32, 33, 255]);
    assert_eq!(chrome.surface_hover, [41, 42, 43, 255]);
    assert_eq!(chrome.surface_selected, [51, 52, 53, 255]);
    assert_eq!(chrome.surface_disabled, [61, 62, 63, 255]);
    assert_eq!(chrome.separator, [71, 72, 73, 255]);
    assert_eq!(chrome.border_disabled, [81, 82, 83, 255]);
    assert_eq!(chrome.strong_separator, [91, 92, 93, 255]);
    assert_eq!(chrome.soft_separator, [101, 102, 103, 255]);
}

#[test]
fn chrome_content_panel_uses_recessed_content_layer_without_focus_fill() {
    let node = TemplatePaneNodeData::default();

    let style = select_workbench_chrome_style(&node, WorkbenchChromeKind::ContentPanel);

    assert_eq!(style.state, UiPainterResolvedState::Normal);
    assert_eq!(style.fill, Some(WORKBENCH_CHROME_CONTENT_BG));

    let mut selected = TemplatePaneNodeData::default();
    selected.selected = true;
    selected.focused = true;
    selected.hovered = true;

    let selected_style =
        select_workbench_chrome_style(&selected, WorkbenchChromeKind::ContentPanel);

    assert_eq!(selected_style.state, UiPainterResolvedState::Focused);
    assert_eq!(selected_style.fill, Some(WORKBENCH_CHROME_CONTENT_BG));
    assert_ne!(selected_style.fill, Some(PALETTE.surface_selected));
}

#[test]
fn chrome_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.button_style.loading = true;
    node.focused = true;
    node.hovered = true;
    node.selected = true;

    let style = select_workbench_chrome_style(&node, WorkbenchChromeKind::StatusBar);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.fill, Some(PALETTE.surface_disabled));
    assert_eq!(style.strong_separator, PALETTE.border_disabled);
    assert_eq!(style.separator, PALETTE.border_disabled);
    assert_eq!(style.soft_separator, PALETTE.border_disabled);
}

#[test]
fn chrome_focused_panel_keeps_normal_fill_with_focus_separator() {
    let mut focused = TemplatePaneNodeData::default();
    focused.focused = true;

    let focused_style =
        select_workbench_chrome_style(&focused, WorkbenchChromeKind::InspectorPanel);

    assert_eq!(focused_style.state, UiPainterResolvedState::Focused);
    assert_eq!(focused_style.fill, Some(WORKBENCH_CHROME_PANEL_BG));
    assert_ne!(focused_style.fill, Some(PALETTE.surface_selected));
    assert_eq!(focused_style.strong_separator, PALETTE.border);
}

#[test]
fn chrome_hover_and_selected_panel_keep_pointer_and_identity_fills() {
    let mut hovered = TemplatePaneNodeData::default();
    hovered.hovered = true;

    let hovered_style =
        select_workbench_chrome_style(&hovered, WorkbenchChromeKind::InspectorPanel);

    assert_eq!(hovered_style.state, UiPainterResolvedState::Hovered);
    assert_eq!(hovered_style.fill, Some(PALETTE.surface_hover));

    let mut selected = TemplatePaneNodeData::default();
    selected.selected = true;

    let selected_style =
        select_workbench_chrome_style(&selected, WorkbenchChromeKind::InspectorPanel);

    assert_eq!(selected_style.state, UiPainterResolvedState::Selected);
    assert_eq!(selected_style.fill, Some(PALETTE.surface_selected));
}

#[test]
fn chrome_drawer_column_stays_fillless_when_selected() {
    let mut selected_column = TemplatePaneNodeData::default();
    selected_column.selected = true;

    let selected_column_style =
        select_workbench_chrome_style(&selected_column, WorkbenchChromeKind::DrawerColumn);

    assert_eq!(
        selected_column_style.state,
        UiPainterResolvedState::Selected
    );
    assert_eq!(selected_column_style.fill, None);
    assert_eq!(selected_column_style.soft_separator, PALETTE.border);
}
