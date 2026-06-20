use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{local_rect, push_segments};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_kebab_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 6.0, 2.0, 2.0, 2.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
            local_rect(rect, 6.0, 10.0, 2.0, 2.0),
        ],
    );
}
