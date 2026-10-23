use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_text::draw_text;
use super::super::super::paint_theme::{current_host_metrics, current_host_palette};

// A real progress overlay anchored to the viewport, clipped by its content frame.
// The event host retains focus and blocks authoring until the owned worker returns.
pub(super) fn draw(frame: &mut HostRgbaFrame, presentation: &HostWindowPresentationData) {
    let Some(label) = presentation.play_transition.as_deref() else {
        return;
    };
    let bounds = &presentation.host_layout.viewport_content_frame;
    let Some(panel) = transition_panel_frame(presentation) else {
        return;
    };
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    let padding = metrics.gap_l;
    frame.record_quad(
        panel.clone(),
        Some(bounds.clone()),
        palette.surface_panel,
        metrics.radius_panel,
    );
    frame.record_border(
        panel.clone(),
        Some(bounds.clone()),
        palette.border,
        metrics.border_width,
        metrics.radius_panel,
    );
    draw_text(
        frame,
        FrameRect {
            x: panel.x + padding,
            y: panel.y + padding,
            width: (panel.width - padding * 2.0).max(0.0),
            height: metrics.row_height,
        },
        label,
        Some(&panel),
        palette.text,
    );
    let track_width = (panel.width - padding * 2.0).max(0.0);
    let pulse_width = track_width * 0.2;
    let pulse = f32::from(presentation.play_transition_phase) / 31.0;
    // Animate paint only. The worker owns backend work; this never measures or
    // arranges the workbench, and stops requesting frames when it completes.
    frame.record_quad(
        FrameRect {
            x: panel.x + padding + (track_width - pulse_width) * pulse,
            y: panel.y + panel.height - padding - metrics.border_width * 3.0,
            width: pulse_width,
            height: metrics.border_width * 3.0,
        },
        Some(panel),
        palette.accent,
        metrics.border_width,
    );
}

pub(in crate::ui::retained_host) fn transition_panel_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let bounds = &presentation.host_layout.viewport_content_frame;
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return None;
    }
    let metrics = current_host_metrics();
    let width = (metrics.row_height * 16.0).min(bounds.width);
    let height = (metrics.row_height * 3.0).min(bounds.height);
    Some(FrameRect {
        x: bounds.x + (bounds.width - width) * 0.5,
        y: bounds.y + (bounds.height - height) * 0.5,
        width,
        height,
    })
}
