use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_axis_value_field_style::axis_field_text_color;
use super::metrics::{AxisValueFieldMetrics, axis_value_field_metrics};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_field_value(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    field: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let value = axis_field_value(node);
    if value.is_empty() {
        return;
    }

    let metrics = axis_value_field_metrics();
    let text_rect = axis_field_text_rect(field, metrics);
    if text_rect.width <= 0.0 || text_rect.height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        value.to_string(),
        axis_field_text_color(node),
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn axis_field_text_rect(field: &FrameRect, metrics: AxisValueFieldMetrics) -> FrameRect {
    let inset_x = metrics.text_inset_x.min(field.width.max(0.0) * 0.5);
    let line_height = metrics.line_height.min(field.height.max(0.0));
    FrameRect {
        x: field.x + inset_x,
        y: field.y + (field.height - line_height).max(0.0) * 0.5,
        width: (field.width - inset_x * 2.0).max(0.0),
        height: line_height,
    }
}

fn axis_field_value(node: &TemplatePaneNodeData) -> &str {
    let value = node.value_text.trim();
    if value.is_empty() {
        node.text.trim()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_slot_stays_inside_a_narrow_short_axis_field() {
        let field = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 4.0,
            height: 6.0,
        };
        let metrics = AxisValueFieldMetrics {
            max_height: 30.0,
            radius: 4.0,
            font_size: 13.33,
            line_height: 16.0,
            text_inset_x: 8.0,
        };

        assert_eq!(
            axis_field_text_rect(&field, metrics),
            FrameRect {
                x: 12.0,
                y: 20.0,
                width: 0.0,
                height: 6.0,
            }
        );
    }

    #[test]
    fn text_slot_keeps_the_authored_regular_field_density() {
        let field = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 58.0,
            height: 24.0,
        };
        let metrics = AxisValueFieldMetrics {
            max_height: 30.0,
            radius: 4.0,
            font_size: 13.33,
            line_height: 16.0,
            text_inset_x: 8.0,
        };

        assert_eq!(
            axis_field_text_rect(&field, metrics),
            FrameRect {
                x: 16.0,
                y: 12.0,
                width: 42.0,
                height: 16.0,
            }
        );
    }
}
