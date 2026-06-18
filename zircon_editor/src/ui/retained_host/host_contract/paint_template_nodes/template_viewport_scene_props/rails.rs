use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_viewport_scene_structure::push_base_surface;

const HANDRAIL_POST: [u8; 4] = [179, 113, 48, 107];
const HANDRAIL_BOTTOM: [u8; 4] = [143, 88, 40, 97];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_rack_detail(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let vertical = [0, 0, 0, 112];
    let horizontal = [172, 109, 55, 31];
    let mut x = rect.x + 8.0;
    while x < rect.x + rect.width {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y,
                width: 2.0,
                height: rect.height,
            },
            Some(clip.clone()),
            order,
            Some(vertical),
            None,
            0.0,
            0.0,
            opacity,
        ));
        x += 28.0;
    }
    let mut y = rect.y + 3.0;
    while y < rect.y + rect.height {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x,
                y,
                width: rect.width,
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1,
            Some(horizontal),
            None,
            0.0,
            0.0,
            opacity,
        ));
        y += 42.0;
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_handrail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height + 1.0,
            width: rect.width,
            height: 2.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(HANDRAIL_BOTTOM),
        None,
        0.0,
        0.0,
        opacity,
    ));
    for x in [rect.x + 36.0, rect.x + rect.width - 42.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y - 3.0,
                width: 4.0,
                height: 56.0,
            },
            Some(clip.clone()),
            order + 2,
            Some(HANDRAIL_POST),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}
