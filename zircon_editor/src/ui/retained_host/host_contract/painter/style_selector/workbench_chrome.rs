use super::super::super::data::TemplatePaneNodeData;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_ROOT_BG: [u8; 4] =
    [8, 11, 14, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_TOPBAR_BG: [u8; 4] =
    [14, 18, 22, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_MAIN_BG: [u8; 4] =
    [10, 13, 16, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_RAIL_BG: [u8; 4] =
    [12, 16, 20, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_PANEL_BG: [u8; 4] =
    [15, 20, 24, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_VIEWPORT_FRAME_BG:
    [u8; 4] = [9, 12, 15, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_DRAWER_BG: [u8; 4] =
    [13, 18, 22, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_DRAWER_BODY_BG:
    [u8; 4] = [12, 16, 20, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_STATUS_BG: [u8; 4] =
    [12, 17, 21, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_TAB_BG: [u8; 4] =
    [14, 19, 23, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_SEPARATOR: [u8; 4] =
    [29, 38, 44, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_STRONG_SEPARATOR:
    [u8; 4] = [38, 49, 56, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_CHROME_SOFT_SEPARATOR:
    [u8; 4] = [24, 31, 36, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) enum WorkbenchChromeKind {
    WindowRoot,
    TopToolbar,
    MainBand,
    ActivityRail,
    ScenePanel,
    ViewportPanel,
    InspectorPanel,
    ComponentDrawer,
    DrawerBody,
    DrawerColumn,
    StatusBar,
    TabsBand,
    InspectorSection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchChromeStyle {
    pub fill: Option<[u8; 4]>,
    pub separator: [u8; 4],
    pub strong_separator: [u8; 4],
    pub soft_separator: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_chrome_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchChromeKind,
) -> WorkbenchChromeStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(UiPainterFamily::Chrome);

    WorkbenchChromeStyle {
        fill: chrome_fill(kind, state),
        separator: chrome_separator(WORKBENCH_CHROME_SEPARATOR, state),
        strong_separator: chrome_separator(WORKBENCH_CHROME_STRONG_SEPARATOR, state),
        soft_separator: chrome_separator(WORKBENCH_CHROME_SOFT_SEPARATOR, state),
        state,
    }
}

fn chrome_fill(kind: WorkbenchChromeKind, state: UiPainterResolvedState) -> Option<[u8; 4]> {
    if kind == WorkbenchChromeKind::DrawerColumn {
        return None;
    }
    let normal = normal_chrome_fill(kind);
    Some(match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.surface_disabled
        }
        UiPainterResolvedState::Pressed => PALETTE.surface_pressed,
        UiPainterResolvedState::Hovered => PALETTE.surface_hover,
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => PALETTE.surface_selected,
        UiPainterResolvedState::Normal => normal,
    })
}

fn normal_chrome_fill(kind: WorkbenchChromeKind) -> [u8; 4] {
    match kind {
        WorkbenchChromeKind::WindowRoot => WORKBENCH_CHROME_ROOT_BG,
        WorkbenchChromeKind::TopToolbar => WORKBENCH_CHROME_TOPBAR_BG,
        WorkbenchChromeKind::MainBand => WORKBENCH_CHROME_MAIN_BG,
        WorkbenchChromeKind::ActivityRail => WORKBENCH_CHROME_RAIL_BG,
        WorkbenchChromeKind::ScenePanel
        | WorkbenchChromeKind::InspectorPanel
        | WorkbenchChromeKind::InspectorSection => WORKBENCH_CHROME_PANEL_BG,
        WorkbenchChromeKind::ViewportPanel => WORKBENCH_CHROME_VIEWPORT_FRAME_BG,
        WorkbenchChromeKind::ComponentDrawer => WORKBENCH_CHROME_DRAWER_BG,
        WorkbenchChromeKind::DrawerBody => WORKBENCH_CHROME_DRAWER_BODY_BG,
        WorkbenchChromeKind::DrawerColumn => unreachable!("drawer columns do not draw a fill"),
        WorkbenchChromeKind::StatusBar => WORKBENCH_CHROME_STATUS_BG,
        WorkbenchChromeKind::TabsBand => WORKBENCH_CHROME_TAB_BG,
    }
}

fn chrome_separator(normal: [u8; 4], state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.border_disabled
        }
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered => PALETTE.border,
        UiPainterResolvedState::Normal => normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(focused_style.strong_separator, PALETTE.focus_ring);

        let mut selected_column = TemplatePaneNodeData::default();
        selected_column.selected = true;

        let selected_column_style =
            select_workbench_chrome_style(&selected_column, WorkbenchChromeKind::DrawerColumn);

        assert_eq!(
            selected_column_style.state,
            UiPainterResolvedState::Selected
        );
        assert_eq!(selected_column_style.fill, None);
        assert_eq!(selected_column_style.soft_separator, PALETTE.focus_ring);
    }
}
