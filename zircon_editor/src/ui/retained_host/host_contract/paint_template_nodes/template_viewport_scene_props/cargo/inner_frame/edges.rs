use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::super::super::primitives::push_rect_line;
use super::palette::{CARGO_INNER_CORNER, CARGO_INNER_LINE};

pub(super) fn push_cargo_inner_edges(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_rect_line(
        commands,
        rect.x,
        rect.y,
        rect.width,
        1.0,
        clip,
        order,
        CARGO_INNER_CORNER,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x,
        rect.y + rect.height - 1.0,
        rect.width,
        1.0,
        clip,
        order + 1,
        CARGO_INNER_LINE,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x,
        rect.y,
        1.0,
        rect.height,
        clip,
        order + 2,
        CARGO_INNER_CORNER,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + rect.width - 1.0,
        rect.y,
        1.0,
        rect.height,
        clip,
        order + 3,
        CARGO_INNER_LINE,
        opacity,
    );
}
