mod identity;
mod scale_link;
mod style;
mod text;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

use identity::{axis_label_kind, AxisLabelKind};
use scale_link::push_scale_link;
use text::push_axis_text;

#[cfg(test)]
use scale_link::scale_link_origin;
#[cfg(test)]
use style::{axis_label_color, AXIS_LABEL_COLOR, AXIS_LABEL_SCALE_COLOR};

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
#[path = "template_axis_labels_tests.rs"]
mod tests;
