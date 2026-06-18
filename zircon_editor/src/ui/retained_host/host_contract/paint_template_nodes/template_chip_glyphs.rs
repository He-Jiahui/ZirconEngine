use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

const CHIP_CHEVRON_SIZE: f32 = 12.0;
const CHIP_CHEVRON_RIGHT: f32 = 8.0;
pub(super) const CHIP_CHEVRON_RESERVE: f32 = CHIP_CHEVRON_SIZE + CHIP_CHEVRON_RIGHT + 4.0;

pub(super) fn chip_has_chevron(node: &TemplatePaneNodeData) -> bool {
    node.popup_open
        || node.options.row_count() > 0
        || matches!(
            node.control_id.as_str(),
            "WorkbenchViewportMode" | "WorkbenchViewportAngle" | "WorkbenchViewportSpeed"
        )
}

pub(super) fn push_chip_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let chevron = FrameRect {
        x: rect.x + rect.width - CHIP_CHEVRON_RIGHT - CHIP_CHEVRON_SIZE,
        y: rect.y + (rect.height - CHIP_CHEVRON_SIZE).max(0.0) * 0.5,
        width: CHIP_CHEVRON_SIZE,
        height: CHIP_CHEVRON_SIZE,
    };
    push_segments(
        commands,
        &chevron,
        clip,
        order,
        color,
        opacity,
        &[
            (3.0, 4.0, 2.0, 2.0),
            (5.0, 6.0, 2.0, 2.0),
            (7.0, 4.0, 2.0, 2.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[(f32, f32, f32, f32)],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_rect(origin, *x, *y, *width, *height),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn scaled_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    let scale_x = origin.width / 12.0;
    let scale_y = origin.height / 12.0;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}
