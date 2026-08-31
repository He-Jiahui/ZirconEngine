use super::super::super::chrome_command_stream::{ChromeCommandKind, ChromeCommandStream};
use super::super::schema::UiProfileTextRun;
use super::frame_math::{intersect_frames, is_visible_frame};

pub(super) fn collect_text_runs(stream: &ChromeCommandStream) -> Vec<UiProfileTextRun> {
    stream
        .commands()
        .iter()
        .enumerate()
        .filter_map(|(command_index, command)| {
            let ChromeCommandKind::Text {
                text,
                color,
                size,
                line_height,
                ..
            } = &command.kind
            else {
                return None;
            };
            if text.trim().is_empty()
                || color[3] == 0
                || !is_visible_frame(&command.frame)
                || !size.is_finite()
                || *size <= 0.0
                || !line_height.is_finite()
                || *line_height <= 0.0
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
            Some(UiProfileTextRun {
                command_index,
                frame: (&command.frame).into(),
                clip: command.clip.as_ref().map(Into::into),
                color: *color,
                font_size: *size,
                line_height: *line_height,
                text_length: text.chars().count(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

    use super::*;
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        ChromeCommand, ChromeCommandLayer,
    };
    use crate::ui::retained_host::host_contract::data::FrameRect;

    fn push_text(stream: &mut ChromeCommandStream, frame: FrameRect, clip: Option<FrameRect>) {
        stream.push_command_for_test(ChromeCommand {
            layer: ChromeCommandLayer::Text,
            z_index: 1,
            frame,
            clip,
            source: None,
            kind: ChromeCommandKind::Text {
                text: "UI".to_owned(),
                color: [225, 232, 244, 255],
                size: 12.0,
                line_height: 16.0,
                style: UiTextRunPaintStyle::default(),
            },
        });
    }

    #[test]
    fn text_profile_records_render_metadata_without_copying_content() {
        let mut stream = ChromeCommandStream::full_rebuild((128, 64));
        push_text(
            &mut stream,
            FrameRect {
                x: 8.0,
                y: 10.0,
                width: 24.0,
                height: 16.0,
            },
            None,
        );

        let runs = collect_text_runs(&stream);

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].command_index, 0);
        assert_eq!(runs[0].text_length, 2);
        assert_eq!(runs[0].color, [225, 232, 244, 255]);
        assert_eq!(runs[0].font_size, 12.0);
        assert_eq!(runs[0].line_height, 16.0);
    }

    #[test]
    fn text_profile_omits_commands_outside_their_clip() {
        let mut stream = ChromeCommandStream::full_rebuild((128, 64));
        push_text(
            &mut stream,
            FrameRect {
                x: 8.0,
                y: 10.0,
                width: 24.0,
                height: 16.0,
            },
            Some(FrameRect {
                x: 80.0,
                y: 10.0,
                width: 24.0,
                height: 16.0,
            }),
        );

        assert!(collect_text_runs(&stream).is_empty());
    }
}
