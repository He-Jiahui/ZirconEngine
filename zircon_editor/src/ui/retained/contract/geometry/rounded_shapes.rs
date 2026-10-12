use super::super::super::chrome_command_stream::{ChromeCommandKind, ChromeCommandStream};
use super::super::schema::UiProfileRoundedShape;
use super::frame_math::{intersect_frames, is_visible_frame};

pub(super) fn collect_rounded_shapes(stream: &ChromeCommandStream) -> Vec<UiProfileRoundedShape> {
    stream
        .commands()
        .iter()
        .enumerate()
        .filter_map(|(command_index, command)| {
            let (corner_radius, border_width) = match &command.kind {
                ChromeCommandKind::Quad { corner_radius, .. } => (*corner_radius, 0.0),
                ChromeCommandKind::Border {
                    width,
                    corner_radius,
                    ..
                } => (*corner_radius, *width),
                ChromeCommandKind::Text { .. }
                | ChromeCommandKind::Image { .. }
                | ChromeCommandKind::Clip => return None,
            };
            if !corner_radius.is_finite()
                || corner_radius <= 0.0
                || !border_width.is_finite()
                || border_width < 0.0
                || !is_visible_frame(&command.frame)
            {
                return None;
            }
            if command
                .clip
                .as_ref()
                .is_some_and(|clip| intersect_frames(&command.frame, clip).is_none())
            {
                return None;
            }
            Some(UiProfileRoundedShape {
                command_index,
                frame: (&command.frame).into(),
                clip: command.clip.as_ref().map(Into::into),
                corner_radius,
                border_width,
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/rounded_shapes.rs"]
mod tests;
