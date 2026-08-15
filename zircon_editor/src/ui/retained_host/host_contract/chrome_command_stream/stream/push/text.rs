use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;

use super::super::super::command::{ChromeCommandKind, ChromeCommandLayer};
use super::super::model::ChromeCommandStream;

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn push_text(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        text: impl Into<String>,
        color: [u8; 4],
        size: f32,
    ) {
        self.push_command(
            ChromeCommandLayer::Text,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Text {
                text: text.into(),
                color,
                size,
                line_height: current_host_metrics().line_height(size.max(1.0)),
                style: UiTextRunPaintStyle::default(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_from_tokens_for_test,
    };
    use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

    #[test]
    fn text_commands_use_the_active_tokenized_line_height() {
        let mut tokens = EditorDesignTokens::workbench_dark();
        tokens.typography.line_height = 1.35;
        let _theme_scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_from_tokens_for_test(&tokens));
        let mut stream = ChromeCommandStream::full_rebuild((64, 64));
        stream.push_text(
            0,
            FrameRect::default(),
            None,
            "Tokenized label",
            [255, 255, 255, 255],
            13.0,
        );

        let command = stream
            .commands()
            .last()
            .expect("text command should be appended");
        let ChromeCommandKind::Text { line_height, .. } = &command.kind else {
            panic!("expected the appended command to be text");
        };

        assert_eq!(*line_height, current_host_metrics().line_height(13.0));
        assert_ne!(*line_height, 13.0 * 1.2);
    }
}
