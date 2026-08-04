use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_section_title_glyphs::{push_section_icon, section_title_icon};
use super::geometry::{
    frame_is_within, has_paintable_section_title_extent, pixel_aligned_rect, section_icon_rect,
};
use super::identity::is_workbench_section_title;
use super::surface::push_section_title_surface;
use super::text::push_section_label;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_title_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_section_title(node) {
        return false;
    }
    let rect = pixel_aligned_rect(rect);
    if !has_paintable_section_title_extent(&rect) || intersect(&rect, clip).is_none() {
        return true;
    }

    push_section_title_surface(commands, &rect, clip, order, opacity);
    let icon = section_title_icon(node);
    let icon_painted = if let Some(icon) = icon {
        let icon_rect = section_icon_rect(&rect);
        if frame_is_within(&rect, &icon_rect) && intersect(&icon_rect, clip).is_some() {
            push_section_icon(commands, &icon_rect, clip, order + 2, icon, opacity);
            true
        } else {
            false
        }
    } else {
        false
    };
    push_section_label(
        commands,
        node,
        &rect,
        clip,
        order + 3,
        icon_painted,
        opacity,
    );
    true
}
