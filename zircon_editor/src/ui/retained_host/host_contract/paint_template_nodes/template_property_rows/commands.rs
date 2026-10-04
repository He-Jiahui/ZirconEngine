//! 属性行先在标签与值区之间分配空间，再按摘要中的轴组数选择多轴或单值外观。
//! 身份不符或没有正文时可继续通用 fallback；已识别但被裁剪的行仍表示已处理。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_property_axis_values::property_axis_values;
use super::fields::{push_axis_value_commands, push_scalar_value_commands};
use super::identity::is_property_row;
use super::labels::push_property_label_command;
use super::layers::value_group_order;
use super::layout::{property_label_width, property_value_area_rect};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_property_row_text_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_property_row(node) {
        return false;
    }

    let label = node.text.trim();
    let value = node.value_text.trim();
    if label.is_empty() && value.is_empty() {
        return false;
    }
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };

    let label_width = property_label_width(node, rect);
    if !label.is_empty() {
        push_property_label_command(
            commands,
            node,
            rect,
            &clip,
            order,
            label,
            label_width,
            opacity,
        );
    }

    if value.is_empty() {
        return true;
    }

    let value_area = property_value_area_rect(rect, label_width);
    let axis_values = property_axis_values(value);
    if axis_values.len() >= 2 {
        push_axis_value_commands(
            commands,
            &axis_values,
            &value_area,
            &clip,
            value_group_order(order),
            opacity,
        );
    } else {
        push_scalar_value_commands(
            commands,
            &clip,
            node,
            &value_area,
            value_group_order(order),
            value,
            opacity,
        );
    }
    true
}
