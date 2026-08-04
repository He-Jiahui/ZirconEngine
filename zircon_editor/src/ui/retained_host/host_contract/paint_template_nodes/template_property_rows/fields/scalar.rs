use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_row_metrics::{workbench_row_palette, WorkbenchRowPalette};
use super::super::layers::field_text_order;
use super::super::layout::{scalar_field_rect, value_text_rect};
use super::super::text::text_command;
use super::surface::push_property_value_field_surface;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_scalar_value_commands(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    order: i32,
    value: &str,
    opacity: f32,
) {
    let palette = workbench_row_palette();
    let field_rect = scalar_field_rect(rect);
    push_property_value_field_surface(
        commands,
        &field_rect,
        clip,
        order,
        value_field_border_color(node, palette),
        opacity,
    );
    let Some(command) = text_command(
        value_text_rect(&field_rect),
        clip,
        field_text_order(order),
        value,
        palette.property_value_text,
        opacity,
    ) else {
        return;
    };
    commands.push(command);
}

fn value_field_border_color(node: &TemplatePaneNodeData, palette: WorkbenchRowPalette) -> [u8; 4] {
    if node.focused || node.pressed {
        palette.property_field_focus_border
    } else {
        palette.property_field_border
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_scalar_property_value_keeps_neutral_field_border() {
        let mut node = TemplatePaneNodeData::default();
        node.selected = true;
        let palette = workbench_row_palette();
        let border = value_field_border_color(&node, palette);

        assert_eq!(border, palette.property_field_border);
        assert_ne!(border, palette.property_field_focus_border);
    }

    #[test]
    fn focused_scalar_property_value_uses_focus_border() {
        let mut node = TemplatePaneNodeData::default();
        node.focused = true;
        let palette = workbench_row_palette();

        assert_eq!(
            value_field_border_color(&node, palette),
            palette.property_field_focus_border
        );
    }

    #[test]
    fn pressed_scalar_property_value_uses_focus_border() {
        let mut node = TemplatePaneNodeData::default();
        node.pressed = true;
        let palette = workbench_row_palette();

        assert_eq!(
            value_field_border_color(&node, palette),
            palette.property_field_focus_border
        );
    }
}
