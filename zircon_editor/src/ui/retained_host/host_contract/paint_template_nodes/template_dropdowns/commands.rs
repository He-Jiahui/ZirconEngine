use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::dropdown_paint_rect;
use super::identity::is_workbench_dropdown;
use super::style::dropdown_style;
use super::surface::push_dropdown_surface;
use super::text::push_dropdown_label;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_dropdown(node) {
        return false;
    }
    let rect = dropdown_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }
    let style = dropdown_style(node);

    push_dropdown_surface(commands, &rect, clip, order, opacity, &style);
    push_dropdown_label(commands, node, &rect, clip, order + 2, opacity, &style);
    super::super::template_dropdown_glyphs::push_dropdown_chevron(
        commands,
        &rect,
        clip,
        order + 3,
        opacity,
        &style,
    );
    true
}
