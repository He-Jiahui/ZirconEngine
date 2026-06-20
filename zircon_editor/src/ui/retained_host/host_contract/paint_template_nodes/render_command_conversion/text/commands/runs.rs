use zircon_runtime_interface::ui::surface::UiTextPaint;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::{
    frame_from_ui, parse_style_color,
};

pub(super) fn push_text_run_commands(
    output: &mut Vec<HostPaintCommand>,
    text: &UiTextPaint,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    fallback_color: [u8; 4],
) -> bool {
    if text.runs.is_empty() {
        return false;
    }

    for run in &text.runs {
        let run_color = parse_style_color(run.color.as_deref()).unwrap_or(fallback_color);
        output.push(HostPaintCommand::text(
            frame_from_ui(run.frame),
            clip_frame.clone(),
            z_index,
            run.text.clone(),
            run_color,
            run.font_size.max(1.0),
            run.line_height.max(run.font_size).max(1.0),
            run.style,
            opacity,
        ));
    }
    true
}
