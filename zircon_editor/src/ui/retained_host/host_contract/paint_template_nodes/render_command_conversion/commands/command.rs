use zircon_runtime_interface::ui::surface::UiRenderCommand;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::frame_from_ui;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::element::push_runtime_paint_element;

/// Runtime 命令的自有裁剪优先于父裁剪，再把临时绘制元素依序投影为宿主命令；这条路径不附加 retained 来源引用。
pub(super) fn push_runtime_command(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    parent_clip: Option<&FrameRect>,
) {
    let command_clip = command
        .clip_frame
        .map(frame_from_ui)
        .or_else(|| parent_clip.cloned());

    for element in command.to_transient_paint_elements(0) {
        push_runtime_paint_element(output, command, &element, command_clip.clone());
    }
}
