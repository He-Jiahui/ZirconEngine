use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::{
    WorkbenchChromeKind as ShellPanelKind, WorkbenchChromeStyle,
};
use super::lines::{
    push_bottom_line, push_left_line, push_right_line, push_top_line, push_vertical_line,
};
use super::metrics::DRAWER_COLUMN_SEPARATOR_OFFSET;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shell_panel_separators(
    commands: &mut Vec<HostPaintCommand>,
    kind: ShellPanelKind,
    style: &WorkbenchChromeStyle,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    match kind {
        ShellPanelKind::TopToolbar => {
            push_bottom_line(commands, rect, clip, order, style.strong_separator, opacity);
        }
        ShellPanelKind::ActivityRail | ShellPanelKind::ScenePanel => {
            push_right_line(commands, rect, clip, order, style.strong_separator, opacity);
        }
        ShellPanelKind::ViewportPanel => {
            push_left_line(commands, rect, clip, order, style.soft_separator, opacity);
            push_right_line(commands, rect, clip, order, style.soft_separator, opacity);
        }
        ShellPanelKind::InspectorPanel => {
            push_left_line(commands, rect, clip, order, style.strong_separator, opacity);
        }
        ShellPanelKind::ComponentDrawer | ShellPanelKind::StatusBar => {
            push_top_line(commands, rect, clip, order, style.strong_separator, opacity);
        }
        ShellPanelKind::TabsBand | ShellPanelKind::InspectorSection => {
            push_bottom_line(commands, rect, clip, order, style.separator, opacity);
        }
        ShellPanelKind::DrawerColumn => {
            push_vertical_line(
                commands,
                rect.x + DRAWER_COLUMN_SEPARATOR_OFFSET,
                rect.y,
                rect.height,
                clip,
                order,
                style.soft_separator,
                opacity,
            );
        }
        ShellPanelKind::WindowRoot | ShellPanelKind::MainBand | ShellPanelKind::DrawerBody => {}
    }
}
