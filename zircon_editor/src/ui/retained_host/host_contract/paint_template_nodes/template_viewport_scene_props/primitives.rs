use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_geometry::corner_radius_for_frame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_corner_radius_from_rect(
    rect: &FrameRect,
) -> f32 {
    corner_radius_for_frame(rect, (rect.height * 0.08).clamp(0.0, 4.0))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_rect_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_prop_corner_radius_stays_inside_a_narrow_surface() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 24.0,
        };

        assert_eq!(template_corner_radius_from_rect(&rect), 0.25);
    }
}
