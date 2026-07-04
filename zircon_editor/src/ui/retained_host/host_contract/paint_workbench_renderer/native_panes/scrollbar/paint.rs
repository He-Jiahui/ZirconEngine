use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_rounded_rect_clipped,
};
use super::super::super::super::paint_theme::current_host_palette;
use super::geometry::vertical_scrollbar_geometry;
use super::style::{workbench_scrollbar_metrics, workbench_scrollbar_palette};

pub(super) fn draw_vertical_scrollbar(
    frame: &mut HostRgbaFrame,
    viewport: &FrameRect,
    clip: &FrameRect,
    scroll_offset: f32,
    content_extent: f32,
    active: bool,
) -> bool {
    let metrics = workbench_scrollbar_metrics();
    let palette = workbench_scrollbar_palette();
    let Some(geometry) =
        vertical_scrollbar_geometry(viewport, scroll_offset, content_extent, metrics)
    else {
        return false;
    };

    draw_rounded_rect_clipped(
        frame,
        geometry.track,
        Some(clip),
        palette.track,
        metrics.radius,
    );
    draw_rounded_rect_clipped(
        frame,
        geometry.thumb,
        Some(clip),
        if active {
            palette.thumb_active
        } else {
            palette.thumb
        },
        metrics.radius,
    );
    true
}

#[cfg(test)]
pub(super) fn paint_scrollbar_component_for_test(width: u32, height: u32) -> Vec<u8> {
    let palette = current_host_palette();
    let mut frame = HostRgbaFrame::filled(width, height, palette.shell_background);
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    let panels = [
        ("top", 88.0, 54.0, 192.0, 252.0, 0.0, false),
        ("mid", 354.0, 54.0, 192.0, 252.0, 328.0, true),
        ("end", 620.0, 54.0, 192.0, 252.0, 656.0, false),
    ];
    for (index, (_label, x, y, width, height, scroll, active)) in panels.iter().enumerate() {
        let panel = FrameRect {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        };
        draw_rounded_rect_clipped(&mut frame, panel.clone(), Some(&clip), palette.surface, 4.0);
        draw_border_clipped(&mut frame, panel.clone(), Some(&clip), palette.border);
        draw_demo_rows(&mut frame, &panel, &clip, index, *scroll);
        draw_vertical_scrollbar(&mut frame, &panel, &clip, *scroll, 908.0, *active);
    }
    frame.into_bytes()
}

#[cfg(test)]
fn draw_demo_rows(
    frame: &mut HostRgbaFrame,
    panel: &FrameRect,
    clip: &FrameRect,
    panel_index: usize,
    scroll: f32,
) {
    let palette = current_host_palette();
    for row in 0..14 {
        let y = panel.y + 10.0 + row as f32 * 32.0 - scroll.rem_euclid(32.0);
        let row_frame = FrameRect {
            x: panel.x + 12.0,
            y,
            width: panel.width - 34.0,
            height: 24.0,
        };
        let color = if row % 5 == panel_index {
            palette.surface_selected
        } else if row % 2 == 0 {
            palette.surface_hover
        } else {
            palette.surface_inset
        };
        draw_rect_clipped(frame, row_frame, Some(clip), color);
    }
}
