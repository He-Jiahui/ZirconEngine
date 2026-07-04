use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::DialogKind;
use super::{layout, metrics::dialog_metrics, style};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dialog_chrome(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: DialogKind,
    unavailable: bool,
    opacity: f32,
) {
    let metrics = dialog_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style::dialog_surface_color(unavailable)),
        Some(style::dialog_border_color(node, kind, unavailable)),
        metrics.border_width,
        metrics.radius,
        opacity,
    ));

    if matches!(kind, DialogKind::ConfirmDialog) {
        commands.push(HostPaintCommand::quad(
            layout::severity_mark_rect(rect),
            Some(clip.clone()),
            order + 1,
            Some(style::severity_mark_color(node)),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
