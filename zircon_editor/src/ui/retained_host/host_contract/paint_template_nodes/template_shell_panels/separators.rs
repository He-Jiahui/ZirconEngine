use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::{WorkbenchChromeKind as ShellPanelKind, WorkbenchChromeStyle};

const DRAWER_COLUMN_SEPARATOR_OFFSET: f32 = -6.0;

pub(super) fn push_shell_panel_separators(
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

fn push_top_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_horizontal_line(commands, rect, rect.y, clip, order, color, opacity);
}

fn push_bottom_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_horizontal_line(
        commands,
        rect,
        rect.y + rect.height - 1.0,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_left_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_vertical_line(
        commands,
        rect.x,
        rect.y,
        rect.height,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_right_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_vertical_line(
        commands,
        rect.x + rect.width - 1.0,
        rect.y,
        rect.height,
        clip,
        order,
        color,
        opacity,
    );
}

fn push_horizontal_line(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    y: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: y.round(),
            width: rect.width,
            height: 1.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_vertical_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    height: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x.round(),
            y: y.round(),
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(super) fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
