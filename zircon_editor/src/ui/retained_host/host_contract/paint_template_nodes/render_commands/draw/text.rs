use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_text::draw_text_with_size_and_style_and_layout_policy;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::command::HostPaintCommand;
use super::color::color_with_opacity;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_text_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    let Some(text) = command.text.as_ref() else {
        return false;
    };
    let color = color_with_opacity(
        command
            .foreground_color
            .unwrap_or_else(|| fallback_text_from_host(current_host_palette())),
        command.opacity,
    );
    draw_text_with_size_and_style_and_layout_policy(
        frame,
        command.frame.clone(),
        text,
        command.clip_frame.as_ref(),
        color,
        command.font_size,
        command.line_height,
        command.text_style,
        command.text_layout_policy,
    );
    true
}

fn fallback_text_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn draw_text_fallback_color_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.text = [10, 11, 12, 255];

        assert_eq!(fallback_text_from_host(palette), [10, 11, 12, 255]);
    }
}
