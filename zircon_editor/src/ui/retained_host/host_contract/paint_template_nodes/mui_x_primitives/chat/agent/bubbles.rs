use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::metrics::{MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION, MUI_X_CHAT_INSET};

pub(super) fn push_agent_bubbles(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let bubble_height = (rect.height * MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION).max(8.0);
    super::super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + MUI_X_CHAT_INSET,
            width: rect.width * 0.58,
            height: bubble_height,
        },
        clip,
        order,
        PALETTE.surface,
        0.0,
        5.0,
        opacity,
    );
    super::super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + MUI_X_CHAT_INSET + bubble_height + 3.0,
            width: (rect.width * 0.58 - MUI_X_CHAT_INSET).max(1.0),
            height: bubble_height,
        },
        clip,
        order + 1,
        PALETTE.surface_selected,
        0.0,
        5.0,
        opacity,
    );
}
