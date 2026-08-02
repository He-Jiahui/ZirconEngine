use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::chip_delete_icon_frame;
use super::super::style::chip_delete_icon_color;
use super::dot::push_chip_delete_dot;
use super::metrics::CHIP_DELETE_DIAGONAL_DOT_COUNT;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_delete_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = chip_delete_icon_frame(node, rect);
    let stroke = chip_delete_dot_stroke(&frame);
    if stroke <= 0.0 {
        return;
    }
    let color = chip_delete_icon_color(node);
    let start_x = frame.x + frame.width * 0.25;
    let end_x = frame.x + frame.width * 0.75;
    let start_y = frame.y + frame.height * 0.25;
    let end_y = frame.y + frame.height * 0.75;
    for index in 0..CHIP_DELETE_DIAGONAL_DOT_COUNT {
        let ratio = if CHIP_DELETE_DIAGONAL_DOT_COUNT <= 1 {
            0.0
        } else {
            index as f32 / (CHIP_DELETE_DIAGONAL_DOT_COUNT - 1) as f32
        };
        push_chip_delete_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            start_y + (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
            stroke,
        );
        push_chip_delete_dot(
            commands,
            start_x + (end_x - start_x) * ratio,
            end_y - (end_y - start_y) * ratio,
            clip,
            order,
            color,
            opacity,
            stroke,
        );
    }
}

fn chip_delete_dot_stroke(frame: &FrameRect) -> f32 {
    CHIP_DELETE_STROKE
        .min(frame.width.min(frame.height) * 0.5)
        .max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_delete_dot_stroke_fits_short_icon_frame() {
        let frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.4,
            height: 0.6,
        };

        assert!(chip_delete_dot_stroke(&frame) <= frame.width * 0.5);
        assert!(chip_delete_dot_stroke(&frame) <= frame.height * 0.5);
    }
}
