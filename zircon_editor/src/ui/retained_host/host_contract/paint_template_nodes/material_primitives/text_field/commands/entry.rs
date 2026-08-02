use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::pixel_aligned_rect;
use super::super::identity::is_text_field_node;
use super::super::variant::{TextFieldVariant, text_field_variant};
use super::surface::{push_filled_field, push_outlined_field, push_standard_field};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text_field_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_text_field_node(node) {
        return false;
    }

    let field_rect = pixel_aligned_rect(rect);
    if field_rect.width <= 0.0 || field_rect.height <= 0.0 {
        return true;
    }

    match text_field_variant(node) {
        TextFieldVariant::Filled => {
            push_filled_field(commands, node, &field_rect, clip, order, opacity)
        }
        TextFieldVariant::Standard => {
            push_standard_field(commands, node, &field_rect, clip, order, opacity)
        }
        TextFieldVariant::Outlined => {
            push_outlined_field(commands, node, &field_rect, clip, order, opacity)
        }
    }
    true
}
