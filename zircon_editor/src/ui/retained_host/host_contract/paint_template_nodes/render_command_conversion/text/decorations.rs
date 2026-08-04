use zircon_runtime_interface::ui::surface::{UiTextPaint, UiTextPaintDecorationKind};

use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::style::{frame_from_ui, parse_style_color};

pub(super) fn push_text_decorations(
    output: &mut Vec<HostPaintCommand>,
    text: &UiTextPaint,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    before_text: bool,
) {
    for decoration in &text.decorations {
        let decoration_before_text = matches!(
            decoration.kind,
            UiTextPaintDecorationKind::Selection | UiTextPaintDecorationKind::TableCellBackground
        );
        if decoration_before_text != before_text {
            continue;
        }
        let color =
            parse_style_color(Some(decoration.color.as_str())).unwrap_or(match decoration.kind {
                UiTextPaintDecorationKind::Selection => [77, 137, 255, 102],
                UiTextPaintDecorationKind::CompositionHighlight => [77, 137, 255, 76],
                UiTextPaintDecorationKind::CompositionUnderline => [77, 137, 255, 255],
                UiTextPaintDecorationKind::Caret => [232, 238, 247, 255],
                UiTextPaintDecorationKind::Outline => [232, 238, 247, 255],
                UiTextPaintDecorationKind::TableCellBackground => [23, 28, 32, 255],
                UiTextPaintDecorationKind::TableCellBorder => [42, 50, 56, 255],
            });
        let decoration_z = if decoration_before_text {
            z_index - 1
        } else {
            z_index + 1
        };
        output.push(HostPaintCommand::quad(
            frame_from_ui(decoration.frame),
            clip_frame.clone(),
            decoration_z,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
