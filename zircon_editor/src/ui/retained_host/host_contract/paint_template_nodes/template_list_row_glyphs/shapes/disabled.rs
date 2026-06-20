use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments::push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_disabled_diamond(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let center_x = rect.x + rect.width * 0.5;
    let center_y = rect.y + rect.height * 0.5;
    push_segments(
        commands,
        &[
            FrameRect {
                x: center_x - 1.0,
                y: center_y - 5.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: center_x + 3.0,
                y: center_y - 1.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: center_x - 1.0,
                y: center_y + 3.0,
                width: 2.0,
                height: 2.0,
            },
            FrameRect {
                x: center_x - 5.0,
                y: center_y - 1.0,
                width: 2.0,
                height: 2.0,
            },
        ],
        clip,
        order,
        PALETTE.text_disabled,
        opacity,
    );
}
