use super::model::WorkbenchChromeKind;
use super::palette::WorkbenchChromePalette;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chrome_fill(
    kind: WorkbenchChromeKind,
    state: UiPainterResolvedState,
    palette: &WorkbenchChromePalette,
) -> Option<[u8; 4]> {
    if kind == WorkbenchChromeKind::ContentPanel {
        return Some(content_panel_fill(state, palette));
    }
    let normal = normal_chrome_fill(kind, palette)?;
    Some(match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.surface_disabled
        }
        UiPainterResolvedState::Pressed => palette.surface_pressed,
        UiPainterResolvedState::Hovered => palette.surface_hover,
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => palette.surface_selected,
        UiPainterResolvedState::Normal | UiPainterResolvedState::Focused => normal,
    })
}

fn content_panel_fill(state: UiPainterResolvedState, palette: &WorkbenchChromePalette) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.surface_disabled
        }
        UiPainterResolvedState::Normal
        | UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => palette.content_bg,
    }
}

fn normal_chrome_fill(
    kind: WorkbenchChromeKind,
    palette: &WorkbenchChromePalette,
) -> Option<[u8; 4]> {
    Some(match kind {
        WorkbenchChromeKind::WindowRoot => palette.root_bg,
        WorkbenchChromeKind::TopToolbar => palette.topbar_bg,
        WorkbenchChromeKind::MainBand => palette.main_bg,
        WorkbenchChromeKind::ActivityRail => palette.rail_bg,
        WorkbenchChromeKind::ScenePanel
        | WorkbenchChromeKind::InspectorPanel
        | WorkbenchChromeKind::InspectorSection => palette.panel_bg,
        WorkbenchChromeKind::ContentPanel => palette.content_bg,
        WorkbenchChromeKind::ViewportPanel => palette.viewport_frame_bg,
        WorkbenchChromeKind::ComponentDrawer => palette.drawer_bg,
        WorkbenchChromeKind::DrawerBody => palette.drawer_body_bg,
        WorkbenchChromeKind::DrawerColumn => return None,
        WorkbenchChromeKind::StatusBar => palette.status_bg,
        WorkbenchChromeKind::TabsBand => palette.tab_bg,
    })
}
