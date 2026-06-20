use super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::{first_non_empty, ACCENT, SEPARATOR};
use super::super::style::{
    WELCOME_MUTED_TEXT, WELCOME_SUCCESS, WELCOME_SURFACE_INSET, WELCOME_TEXT,
};

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_hero(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    hero: &FrameRect,
    clip: &FrameRect,
) {
    draw_text_bars_clipped(
        frame,
        hero.x,
        hero.y + 4.0,
        first_non_empty(&[pane.welcome.title.as_str(), "Open or Create"]),
        Some(clip),
        WELCOME_TEXT,
    );
    draw_text_bars_clipped(
        frame,
        hero.x,
        hero.y + 30.0,
        first_non_empty(&[
            pane.welcome.subtitle.as_str(),
            "Recent projects and a renderable empty-project template",
        ]),
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
    let accent = FrameRect {
        x: hero.x,
        y: hero.y + hero.height - 10.0,
        width: 96.0_f32.min(hero.width),
        height: 2.0,
    };
    draw_rect_clipped(frame, accent, Some(clip), ACCENT);
}

pub(in crate::ui::retained_host::host_contract) fn draw_welcome_status(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    status: &FrameRect,
    clip: &FrameRect,
) {
    draw_rect_clipped(frame, status.clone(), Some(clip), WELCOME_SURFACE_INSET);
    draw_border_clipped(frame, status.clone(), Some(clip), SEPARATOR);
    let marker = FrameRect {
        x: status.x + 10.0,
        y: status.y + 10.0,
        width: 8.0,
        height: 8.0,
    };
    draw_rect_clipped(frame, marker, Some(clip), WELCOME_SUCCESS);
    draw_text_bars_clipped(
        frame,
        status.x + 28.0,
        status.y + 7.0,
        first_non_empty(&[pane.welcome.status_message.as_str(), "Ready"]),
        Some(clip),
        WELCOME_MUTED_TEXT,
    );
}
