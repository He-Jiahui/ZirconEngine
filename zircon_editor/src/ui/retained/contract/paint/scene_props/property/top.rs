use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_viewport_scene_structure::push_base_surface;
use super::super::primitives::push_rect_line;
use super::palette::{PROP_BOTTOM_SHADOW, PROP_SIDE_SHADOW, PROP_TOP_HIGHLIGHT};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_prop_top_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + 2.0,
        (rect.width - 4.0).max(1.0),
        3.0,
        clip,
        order + 1,
        PROP_TOP_HIGHLIGHT,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + rect.width - 4.0,
        rect.y + 3.0,
        2.0,
        (rect.height - 6.0).max(1.0),
        clip,
        order + 2,
        PROP_SIDE_SHADOW,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + rect.height - 3.0,
        (rect.width - 4.0).max(1.0),
        2.0,
        clip,
        order + 3,
        PROP_BOTTOM_SHADOW,
        opacity,
    );
}
