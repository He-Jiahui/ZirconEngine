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
mod tests {
    use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn recent_list_fallback_uses_shared_horizontal_padding_and_header_gap() {
        let layout = WelcomePaneLayoutData::default();
        let body = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 240.0,
        };
        let recent_panel = FrameRect {
            x: 20.0,
            y: 10.0,
            width: 280.0,
            height: 220.0,
        };
        let header = FrameRect {
            x: 20.0,
            y: 28.0,
            width: 280.0,
            height: 46.0,
        };

        let list = recent_projects_list_frame(&layout, &body, &recent_panel, &header);

        let metrics = current_host_metrics();
        assert_eq!(list.x, recent_panel.x + metrics.gap_l);
        assert_eq!(list.y, header.y + header.height + metrics.gap_m);
        assert_eq!(list.width, recent_panel.width - metrics.gap_l * 2.0);
        assert_eq!(list.y + list.height, recent_panel.y + recent_panel.height);
    }

    #[test]
    fn recent_list_surface_uses_shared_radius_and_border_width() {
        let mut frame = HostRgbaFrame::recording_only(320, 160);
        let list = FrameRect {
            x: 12.0,
            y: 8.0,
            width: 296.0,
            height: 144.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 160.0,
        };

        draw_recent_projects_list_surface(&mut frame, &list, &clip);

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 2);
        let metrics = current_host_metrics();
        assert!(matches!(
            &commands[0].kind,
            HostRecordedPaintKind::Quad { corner_radius, .. }
                if *corner_radius == metrics.radius_control
        ));
        assert!(matches!(
            &commands[1].kind,
            HostRecordedPaintKind::Border {
                width,
                corner_radius,
                ..
            } if *width == metrics.border_width && *corner_radius == metrics.radius_control
        ));
    }
}
