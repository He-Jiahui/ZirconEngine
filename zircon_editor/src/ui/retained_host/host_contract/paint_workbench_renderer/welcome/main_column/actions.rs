use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::SEPARATOR;
use super::super::style::{
    WELCOME_ACTION_DISABLED_SURFACE, WELCOME_ACTION_DISABLED_TEXT, WELCOME_PRIMARY_ACTION,
    WELCOME_SURFACE_HOVERED, WELCOME_TEXT,
};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_actions(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    actions: &FrameRect,
    clip: &FrameRect,
) {
    let create_width = 154.0_f32.min(actions.width * 0.45);
    let open_width = 116.0_f32.min(actions.width * 0.34);
    let gap = 10.0_f32.min(actions.width * 0.04);
    let create = FrameRect {
        x: actions.x + actions.width - create_width,
        y: actions.y,
        width: create_width,
        height: actions.height,
    };
    let open = FrameRect {
        x: (create.x - gap - open_width).max(actions.x),
        y: actions.y,
        width: open_width,
        height: actions.height,
    };
    draw_welcome_button(
        frame,
        &open,
        "Open",
        false,
        pane.welcome.form.can_open_existing,
        clip,
    );
    draw_welcome_button(
        frame,
        &create,
        "Create Project",
        true,
        pane.welcome.form.can_create,
        clip,
    );
}

fn draw_welcome_button(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    label: &str,
    primary: bool,
    enabled: bool,
    clip: &FrameRect,
) {
    let color = if !enabled {
        WELCOME_ACTION_DISABLED_SURFACE
    } else if primary {
        WELCOME_PRIMARY_ACTION
    } else {
        WELCOME_SURFACE_HOVERED
    };
    let text = if enabled {
        WELCOME_TEXT
    } else {
        WELCOME_ACTION_DISABLED_TEXT
    };
    draw_rect_clipped(frame, rect.clone(), Some(clip), color);
    draw_border_clipped(frame, rect.clone(), Some(clip), SEPARATOR);
    draw_text_bars_clipped(frame, rect.x + 14.0, rect.y + 8.0, label, Some(clip), text);
}
