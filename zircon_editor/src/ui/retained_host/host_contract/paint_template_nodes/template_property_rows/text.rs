use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_row_metrics::workbench_row_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_command(
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) -> HostPaintCommand {
    let metrics = workbench_row_metrics();
    HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        metrics.text_font_size,
        metrics.text_line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    )
}
