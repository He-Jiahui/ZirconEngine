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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        ChromeCommand, ChromeCommandLayer,
    };
    use crate::ui::retained_host::host_contract::data::FrameRect;

    fn push_quad(stream: &mut ChromeCommandStream, radius: f32) {
        stream.push_command_for_test(ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 1,
            frame: FrameRect {
                x: 8.0,
                y: 10.0,
                width: 48.0,
                height: 32.0,
            },
            clip: None,
            source: None,
            kind: ChromeCommandKind::Quad {
                color: [40, 45, 50, 255],
                corner_radius: radius,
            },
        });
    }

    #[test]
    fn rounded_shape_profile_keeps_physical_radius_and_command_identity() {
        let mut stream = ChromeCommandStream::full_rebuild((128, 64));
        push_quad(&mut stream, 10.5);

        let shapes = collect_rounded_shapes(&stream);

        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].command_index, 0);
        assert_eq!(shapes[0].corner_radius, 10.5);
        assert_eq!(shapes[0].border_width, 0.0);
    }

    #[test]
    fn rounded_shape_profile_omits_zero_radius_and_fully_clipped_commands() {
        let mut stream = ChromeCommandStream::full_rebuild((128, 64));
        push_quad(&mut stream, 0.0);
        stream.push_command_for_test(ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 2,
            frame: FrameRect {
                x: 8.0,
                y: 10.0,
                width: 48.0,
                height: 32.0,
            },
            clip: Some(FrameRect {
                x: 100.0,
                y: 10.0,
                width: 12.0,
                height: 12.0,
            }),
            source: None,
            kind: ChromeCommandKind::Quad {
                color: [40, 45, 50, 255],
                corner_radius: 8.0,
            },
        });

        assert!(collect_rounded_shapes(&stream).is_empty());
    }
}
