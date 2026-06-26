mod base;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::{
    WorkbenchStatusSignalKind as StatusSignalKind, WorkbenchStatusSignalStyle,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_status_signal_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    _kind: StatusSignalKind,
    style: WorkbenchStatusSignalStyle,
    opacity: f32,
) {
    base::push_signal_circle(commands, rect, clip, order, style.icon_fill, opacity);
}
