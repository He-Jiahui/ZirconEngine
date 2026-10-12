//! 选择标记后的文字统一使用共享节点标签与宿主密度；空标签或槽高不足时不发文字命令。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_selection_control_geometry::workbench_selection_control_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

/// checkbox、radio、toggle 已计算各自标签槽后共用此出口；它只验证文本和高度，不决定控件整体归属。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_selection_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let label = template_node_label(node, None);
    let metrics = workbench_selection_control_metrics();
    if label.trim().is_empty()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height < metrics.line_height
    {
        return;
    }
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
