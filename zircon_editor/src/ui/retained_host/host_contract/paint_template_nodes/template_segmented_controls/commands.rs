use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{is_segmented_control, is_workbench_tab};
use super::options::segmented_options;
use super::segments::push_segmented_control;
use super::tabs::push_workbench_tab;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segmented_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_segmented_control(node) {
        let options = segmented_options(node);
        if options.is_empty() {
            return false;
        }
        push_segmented_control(commands, node, rect, clip, order, opacity, &options);
        return true;
    }

    if is_workbench_tab(node) {
        push_workbench_tab(commands, node, rect, clip, order, opacity);
        return true;
    }

    false
}
