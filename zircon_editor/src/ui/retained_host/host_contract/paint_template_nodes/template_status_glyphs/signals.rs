mod base;
mod info;
mod success;
mod warning;

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
    kind: StatusSignalKind,
    style: WorkbenchStatusSignalStyle,
    mark_width: f32,
    opacity: f32,
) {
    match kind {
        StatusSignalKind::Ready => {
            base::push_signal_circle(commands, rect, clip, order, style.icon_fill, opacity);
        }
        StatusSignalKind::Success => {
            success::push_success_signal(commands, rect, clip, order, style, opacity);
        }
        StatusSignalKind::Warning => {
            warning::push_warning_signal(commands, rect, clip, order, style, mark_width, opacity);
        }
        StatusSignalKind::Info => {
            info::push_info_signal(commands, rect, clip, order, style, opacity);
        }
    }
}
