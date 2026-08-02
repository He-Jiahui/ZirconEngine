use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::{WorkbenchAlertKind, workbench_alert_kind};
use super::inline::push_inline_alert;
use super::layout::{frame_is_within, has_paintable_alert_extent, pixel_aligned_rect};
use super::toast::push_toast;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = workbench_alert_kind(node) else {
        return false;
    };
    if !has_paintable_alert_extent(rect) {
        return true;
    }
    let rect = pixel_aligned_rect(rect);
    if !frame_is_within(&rect, clip) {
        return true;
    }

    match kind {
        WorkbenchAlertKind::Inline(tone) => {
            push_inline_alert(commands, node, &rect, clip, order, tone, opacity);
        }
        WorkbenchAlertKind::Toast => {
            push_toast(commands, node, &rect, clip, order, opacity);
        }
    }
    true
}
