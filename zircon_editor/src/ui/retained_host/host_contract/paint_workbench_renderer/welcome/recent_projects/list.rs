use super::super::super::super::data::{FrameRect, WelcomePaneLayoutData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_rounded_box_clipped;
use super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::SEPARATOR;
use super::super::layout::translated_welcome_frame;
use super::super::style::WELCOME_SURFACE_INSET;

pub(super) fn recent_projects_list_frame(
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
    recent_panel: &FrameRect,
    header: &FrameRect,
) -> FrameRect {
    translated_welcome_frame(layout.recent_list_panel.as_ref(), body).unwrap_or_else(|| {
        let metrics = current_host_metrics();
        let y = header.y + header.height + metrics.gap_m;
        FrameRect {
            x: recent_panel.x + metrics.gap_l,
            y,
            width: (recent_panel.width - metrics.gap_l * 2.0).max(0.0),
            height: (recent_panel.y + recent_panel.height - y).max(0.0),
        }
    })
}

pub(super) fn draw_recent_projects_list_surface(
    frame: &mut HostRgbaFrame,
    list: &FrameRect,
    clip: &FrameRect,
) {
    let metrics = current_host_metrics();
    draw_rounded_box_clipped(
        frame,
        list.clone(),
        Some(clip),
        WELCOME_SURFACE_INSET,
        SEPARATOR,
        metrics.border_width,
        metrics.radius_control,
    );
}

#[cfg(test)]
#[path = "tests/list.rs"]
mod tests;
