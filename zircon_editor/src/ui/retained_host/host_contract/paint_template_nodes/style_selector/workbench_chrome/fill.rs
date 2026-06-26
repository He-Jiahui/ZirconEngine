use super::model::WorkbenchChromeKind;
use super::palette::{
    WORKBENCH_CHROME_DRAWER_BG, WORKBENCH_CHROME_DRAWER_BODY_BG, WORKBENCH_CHROME_MAIN_BG,
    WORKBENCH_CHROME_PANEL_BG, WORKBENCH_CHROME_RAIL_BG, WORKBENCH_CHROME_ROOT_BG,
    WORKBENCH_CHROME_STATUS_BG, WORKBENCH_CHROME_TAB_BG, WORKBENCH_CHROME_TOPBAR_BG,
    WORKBENCH_CHROME_VIEWPORT_FRAME_BG,
};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chrome_fill(
    kind: WorkbenchChromeKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
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
        | WorkbenchChromeKind::ContentPanel
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
