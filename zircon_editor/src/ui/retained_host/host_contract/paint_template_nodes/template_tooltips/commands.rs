use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::select_workbench_tooltip_style;
use super::super::template_tooltip_glyphs::{
    push_tooltip_arrow, push_tooltip_info_icon, tooltip_arrow_size, tooltip_icon_size,
};
use super::identity::is_workbench_tooltip;
use super::layout::{pixel_aligned_rect, tooltip_bubble_rect};
use super::surface::push_tooltip_surface;
use super::text::push_tooltip_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_tooltip(node) {
        return false;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let style = select_workbench_tooltip_style(node);
    let bubble = tooltip_bubble_rect(node, &rect);
    let arrow_size = tooltip_arrow_size(node);
    let icon_size = tooltip_icon_size(node);

    push_tooltip_surface(
        commands,
        &bubble,
        clip,
        order,
        style.shadow,
        style.surface,
        style.border,
        opacity,
    );
    push_tooltip_text(
        commands,
        node,
        &bubble,
        clip,
        order + 2,
        style.title,
        style.body,
        opacity,
    );
    push_tooltip_arrow(
        commands,
        &bubble,
        clip,
        order + 3,
        arrow_size,
        style.arrow,
        style.border,
        opacity,
    );
    push_tooltip_info_icon(
        commands,
        node,
        &rect,
        clip,
        order + 4,
        icon_size,
        style.icon,
        opacity,
    );

    true
}
