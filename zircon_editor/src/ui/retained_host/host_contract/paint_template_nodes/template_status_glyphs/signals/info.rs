use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::style_selector::WorkbenchStatusSignalStyle;

use super::super::geometry::{local_rect_scaled, STATUS_ITEM_ICON_SIZE};
use super::super::segments::push_segments;
use super::base::push_signal_circle;

pub(super) fn push_info_signal(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchStatusSignalStyle,
    opacity: f32,
) {
    push_signal_circle(commands, rect, clip, order, style.icon_fill, opacity);
    push_segments(
        commands,
        clip,
        order + 1,
        style.mark,
        opacity,
        &[
            local_rect_scaled(rect, 6.0, 3.0, 2.0, 2.0, STATUS_ITEM_ICON_SIZE),
            local_rect_scaled(rect, 6.0, 6.0, 2.0, 5.0, STATUS_ITEM_ICON_SIZE),
        ],
    );
}
