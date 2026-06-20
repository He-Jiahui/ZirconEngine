use zircon_runtime_interface::ui::surface::{UiTextPaint, UiTextRunPaintStyle};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::frame_from_ui;

pub(super) fn push_shaped_text_commands(
    output: &mut Vec<HostPaintCommand>,
    text: &UiTextPaint,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    color: [u8; 4],
) -> bool {
    let Some(shaped) = text.shaped.as_ref() else {
        return false;
    };

    for line in &shaped.lines {
        output.push(HostPaintCommand::text(
            frame_from_ui(line.frame),
            clip_frame.clone(),
            z_index,
            line.text.clone(),
            color,
            text.font_size.max(1.0),
            text.line_height.max(text.font_size).max(1.0),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
    true
}
