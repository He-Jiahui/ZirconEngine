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
                text:text.clone(),
                source_ref:stream.resolve_command_source(command_index).map(|(frame,reference,fragment)|serde_json::json!({"treeId":frame.tree_id,"generation":frame.generation,"nodeId":reference.node_id,"nodeCommandIndex":reference.node_command_index,"fragmentIndex":fragment})),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/text_runs.rs"]
mod tests;
