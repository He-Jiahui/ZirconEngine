use super::super::super::text::push_slider_label;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::WorkbenchSliderStyle;

pub(super) fn push_sequence_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: Option<String>,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    if let Some(label) = label {
        push_slider_label(commands, rect, clip, order, label, style, opacity);
    }
}
