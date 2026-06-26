use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::METRICS;
use super::super::super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: &str,
    color: [u8; 4],
    opacity: f32,
) {
    if text.trim().is_empty() {
        return;
    }
    let line_height = METRICS.line_height(METRICS.font_body);
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        text.to_string(),
        color,
        METRICS.font_body,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
