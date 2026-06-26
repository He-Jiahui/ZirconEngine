use super::model::WorkbenchChromeKind;
use super::palette::{
    WORKBENCH_CHROME_DRAWER_BG, WORKBENCH_CHROME_DRAWER_BODY_BG, WORKBENCH_CHROME_MAIN_BG,
    WORKBENCH_CHROME_PANEL_BG, WORKBENCH_CHROME_RAIL_BG, WORKBENCH_CHROME_ROOT_BG,
    WORKBENCH_CHROME_SEPARATOR, WORKBENCH_CHROME_SOFT_SEPARATOR, WORKBENCH_CHROME_STATUS_BG,
    WORKBENCH_CHROME_STRONG_SEPARATOR, WORKBENCH_CHROME_TAB_BG, WORKBENCH_CHROME_TOPBAR_BG,
    WORKBENCH_CHROME_VIEWPORT_FRAME_BG,
};
use super::selection::select_workbench_chrome_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{project_host_palette, PALETTE};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

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

    assert_eq!(WORKBENCH_CHROME_ROOT_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_TOPBAR_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_MAIN_BG, projected.shell_background);
    assert_eq!(WORKBENCH_CHROME_RAIL_BG, projected.surface_inset);
    assert_eq!(WORKBENCH_CHROME_PANEL_BG, projected.surface);
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
fn chrome_active_state_uses_shared_focus_and_hot_visuals() {
    let mut focused = TemplatePaneNodeData::default();
    focused.focused = true;

    let focused_style =
        select_workbench_chrome_style(&focused, WorkbenchChromeKind::InspectorPanel);

    assert_eq!(focused_style.state, UiPainterResolvedState::Focused);
    assert_eq!(focused_style.fill, Some(PALETTE.surface_selected));
    assert_eq!(focused_style.strong_separator, PALETTE.border);

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
