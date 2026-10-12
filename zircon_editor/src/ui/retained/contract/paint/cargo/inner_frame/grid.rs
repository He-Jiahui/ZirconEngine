use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::super::super::primitives::push_rect_line;
use super::palette::CARGO_INNER_LINE;

pub(super) fn push_cargo_inner_grid(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if rect.width >= 48.0 {
        let mut divider_order = order + 4;
        for x_factor in [0.34_f32, 0.66] {
            push_rect_line(
                commands,
                (rect.x + rect.width * x_factor).round(),
                rect.y + 4.0,
                1.0,
                (rect.height - 8.0).max(1.0),
                clip,
                divider_order,
                CARGO_INNER_LINE,
                opacity,
            );
            divider_order += 1;
        }
    }

    if rect.height >= 32.0 {
        push_rect_line(
            commands,
            rect.x + 4.0,
            (rect.y + rect.height * 0.52).round(),
            (rect.width - 8.0).max(1.0),
            1.0,
            clip,
            order + 6,
            CARGO_INNER_LINE,
            opacity,
        );
    }
}
