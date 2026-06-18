use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_viewport_scene_structure::push_base_surface;
use super::primitives::push_rect_line;

const PROP_TOP_HIGHLIGHT: [u8; 4] = [255, 255, 255, 24];
const PROP_EDGE_LIGHT: [u8; 4] = [180, 198, 202, 22];
const PROP_SIDE_SHADOW: [u8; 4] = [0, 0, 0, 72];
const PROP_BOTTOM_SHADOW: [u8; 4] = [0, 0, 0, 54];

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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_prop_body_detail(
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
        5.0,
        clip,
        order + 1,
        PROP_TOP_HIGHLIGHT,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + 3.0,
        rect.y + 8.0,
        2.0,
        (rect.height - 14.0).max(1.0),
        clip,
        order + 2,
        PROP_EDGE_LIGHT,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + rect.width - 5.0,
        rect.y + 6.0,
        3.0,
        (rect.height - 12.0).max(1.0),
        clip,
        order + 3,
        PROP_SIDE_SHADOW,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + rect.height - 5.0,
        (rect.width - 4.0).max(1.0),
        4.0,
        clip,
        order + 4,
        PROP_BOTTOM_SHADOW,
        opacity,
    );
}
