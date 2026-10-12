//! 变换轴标签与比例链接标记的 secondary 专用接管入口；按控件身份区分文本和图标后抑制普通 fallback。
//! 该入口只画静态视觉，实际坐标编辑与链接交互由工作台事件链负责。

#[path = "axis_labels/identity/mod.rs"]
mod identity;
#[path = "axis_labels/layers.rs"]
mod layers;
#[path = "axis_labels/metrics/mod.rs"]
mod metrics;
#[path = "axis_labels/palette/mod.rs"]
mod palette;
#[path = "axis_labels/scale_link/mod.rs"]
mod scale_link;
#[path = "axis_labels/style/mod.rs"]
mod style;
#[path = "axis_labels/text/mod.rs"]
mod text;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

use identity::{axis_label_kind, AxisLabelKind};
use scale_link::push_scale_link;
use text::push_axis_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_label_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match axis_label_kind(node) {
        Some(AxisLabelKind::Axis(axis)) => {
            push_axis_text(commands, node, rect, clip, order, axis, opacity);
            true
        }
        Some(AxisLabelKind::ScaleLink) => {
            push_scale_link(commands, node, rect, clip, order, opacity);
            true
        }
        None => false,
    }
}

#[cfg(test)]
#[path = "axis_labels_tests/tests/mod.rs"]
mod tests;
