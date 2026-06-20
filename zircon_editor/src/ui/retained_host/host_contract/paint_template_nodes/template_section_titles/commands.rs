use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_section_title_glyphs::{push_section_icon, section_title_icon};
use super::geometry::{pixel_aligned_rect, section_icon_rect};
use super::identity::is_workbench_section_title;
use super::text::push_section_label;

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
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let icon = section_title_icon(node);
    if let Some(icon) = icon {
        let icon_rect = section_icon_rect(&rect);
        push_section_icon(commands, &icon_rect, clip, order, icon, opacity);
    }
    push_section_label(
        commands,
        node,
        &rect,
        clip,
        order + 2,
        icon.is_some(),
        opacity,
    );
    true
}
