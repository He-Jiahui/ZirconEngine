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
mod tests {
    use super::*;

    #[test]
    fn handrail_posts_follow_the_rail_width_and_available_clip_height() {
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 100.0,
        };
        let standard = FrameRect {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 4.0,
        };
        let wide = FrameRect {
            width: 200.0,
            ..standard.clone()
        };
        let narrow = FrameRect {
            width: 24.0,
            ..standard.clone()
        };
        let subpixel = FrameRect {
            width: 0.5,
            ..standard.clone()
        };
        let short_clip = FrameRect {
            height: 30.0,
            ..clip.clone()
        };

        let standard_posts = handrail_post_rects(&standard, &clip);
        let wide_posts = handrail_post_rects(&wide, &clip);
        let narrow_posts = handrail_post_rects(&narrow, &clip);
        let subpixel_posts = handrail_post_rects(&subpixel, &clip);
        let clipped_posts = handrail_post_rects(&standard, &short_clip);

        assert_eq!(standard_posts[0].x, 46.0);
        assert_eq!(standard_posts[1].x, 68.0);
        assert_eq!(standard_posts[0].height, 56.0);
        assert_eq!(wide_posts[0].x, 82.0);
        assert_eq!(wide_posts[1].x, 126.0);
        assert!(narrow_posts[0].x >= narrow.x);
        assert!(narrow_posts[1].right() <= narrow.right());
        assert!(subpixel_posts[0].right() <= subpixel.right());
        assert!(subpixel_posts[1].right() <= subpixel.right());
        assert_eq!(clipped_posts[0].bottom(), short_clip.bottom());
    }
}
