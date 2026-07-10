use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_rounded_rect_clipped,
};
#[cfg(test)]
use super::super::super::super::paint_text::draw_text;
use super::super::super::super::paint_theme::current_host_palette;
use super::geometry::vertical_scrollbar_geometry;
use super::style::{workbench_scrollbar_metrics, workbench_scrollbar_palette};

pub(in crate::ui::retained_host::host_contract::paint_workbench_renderer) fn draw_vertical_scrollbar(
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
    draw_text(
        &mut frame,
        FrameRect {
            x: 56.0,
            y: 18.0,
            width: 320.0,
            height: 22.0,
        },
        "Scrollbar Components",
        Some(&clip),
        palette.text,
    );
    draw_text(
        &mut frame,
        FrameRect {
            x: 56.0,
            y: 42.0,
            width: 620.0,
            height: 18.0,
        },
        "Native pane scrollbars use relative content extents, clipped rows and stateful thumbs",
        Some(&clip),
        palette.text_muted,
    );
    let panels = [
        ("Top", 70.0, 82.0, 206.0, 226.0, 0.0, false),
        ("Active", 347.0, 82.0, 206.0, 226.0, 347.0, true),
        ("End", 624.0, 82.0, 206.0, 226.0, 694.0, false),
    ];
    for (index, (label, x, y, width, height, scroll, active)) in panels.iter().enumerate() {
        draw_text(
            &mut frame,
            FrameRect {
                x: *x,
                y: *y - 22.0,
                width: *width,
                height: 18.0,
            },
            label,
            Some(&clip),
            if *active {
                palette.text
            } else {
                palette.text_muted
            },
        );
        let panel = FrameRect {
            x: *x,
            y: *y,
            width: *width,
            height: *height,
        };
        draw_rounded_rect_clipped(&mut frame, panel.clone(), Some(&clip), palette.surface, 4.0);
        draw_border_clipped(&mut frame, panel.clone(), Some(&clip), palette.border);
        draw_demo_rows(&mut frame, &panel, index, *scroll);
        draw_vertical_scrollbar(&mut frame, &panel, &clip, *scroll, 920.0, *active);
    }
    frame.into_bytes()
}

#[cfg(test)]
fn draw_demo_rows(frame: &mut HostRgbaFrame, panel: &FrameRect, panel_index: usize, scroll: f32) {
    let palette = current_host_palette();
    let content_clip = FrameRect {
        x: panel.x + 1.0,
        y: panel.y + 1.0,
        width: panel.width - 10.0,
        height: panel.height - 2.0,
    };
    for row in 0..18 {
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
        draw_rect_clipped(frame, row_frame.clone(), Some(&content_clip), color);
        draw_text(
            frame,
            FrameRect {
                x: row_frame.x + 8.0,
                y: row_frame.y + 5.0,
                width: row_frame.width - 16.0,
                height: 14.0,
            },
            &format!("Item {:02}", row + 1),
            Some(&content_clip),
            if row % 5 == panel_index {
                palette.text
            } else {
                palette.text_muted
            },
        );
    }
}
