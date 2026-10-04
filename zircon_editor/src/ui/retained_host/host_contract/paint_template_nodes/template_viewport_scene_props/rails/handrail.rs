use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::palette::{HANDRAIL_BOTTOM, HANDRAIL_POST};

const POST_LEFT_RATIO: f32 = 0.36;
const POST_RIGHT_RATIO: f32 = 0.58;
const POST_WIDTH_RATIO: f32 = 0.04;
const POST_MAX_WIDTH: f32 = 4.0;
const POST_TOP_OFFSET: f32 = 3.0;
const POST_MAX_HEIGHT: f32 = 56.0;

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
    for post in handrail_post_rects(rect, clip) {
        if post.height <= 0.0 {
            continue;
        }
        commands.push(HostPaintCommand::quad(
            post,
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

fn handrail_post_rects(rect: &FrameRect, clip: &FrameRect) -> [FrameRect; 2] {
    let rail_width = rect.width.max(0.0);
    let width = (rail_width * POST_WIDTH_RATIO).min(POST_MAX_WIDTH);
    let top = rect.y - POST_TOP_OFFSET;
    let height = (clip.y + clip.height - top).clamp(0.0, POST_MAX_HEIGHT);
    let max_x = rect.x + rail_width - width;
    let post = |ratio: f32| FrameRect {
        x: (rect.x + rail_width * ratio).min(max_x),
        y: top,
        width,
        height,
    };

    [post(POST_LEFT_RATIO), post(POST_RIGHT_RATIO)]
}

#[cfg(test)]
#[path = "tests/handrail.rs"]
mod tests;
