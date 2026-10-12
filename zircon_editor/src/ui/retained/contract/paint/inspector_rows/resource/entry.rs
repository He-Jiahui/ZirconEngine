//! 资源属性先画标签，再画 Material 单槽计数，最后画选择字段；该顺序保证数值与图标落在相同字段层。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_kind::InspectorResourceKind;
use super::super::primitives::push_label;
use super::super::style::resource_label_color;
use super::count::{push_resource_count, resource_count_width};
use super::field::push_resource_field;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_resource_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    resource: InspectorResourceKind,
    opacity: f32,
) {
    push_label(
        commands,
        rect,
        clip,
        order,
        node.text.trim(),
        resource_label_color(node),
        opacity,
    );
    push_resource_count(commands, node, rect, clip, order + 1, resource, opacity);
    push_resource_field(
        commands,
        node,
        rect,
        clip,
        order + 2,
        resource_count_width(resource),
        resource,
        opacity,
    );
}
