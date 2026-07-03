mod entry;
mod fallback;
mod runs;
mod shaped;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_text_paint_commands;

#[cfg(test)]
mod tests {
    use super::entry::push_text_paint_commands;
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::commands::runtime_render_commands_to_host;
    use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
    use zircon_runtime_interface::ui::event_ui::UiNodeId;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{
        UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiShapedGlyph, UiShapedText,
        UiShapedTextCluster, UiShapedTextLine, UiTextDirection, UiTextOverflow, UiTextPaint,
        UiTextRange, UiTextRenderMode, UiTextRunKind, UiTextWritingMode,
    };

    #[test]
    fn fallback_text_command_preserves_strong_style_from_resolved_font_weight() {
        let mut command = text_command("Warnings");
        command.style.font_weight = 650;

        let host_commands = runtime_render_commands_to_host(&[command], None);
        let text = only_text_command(&host_commands);

        assert_eq!(text.text.as_deref(), Some("Warnings"));
        assert!(text.text_style.strong);
        assert!(!text.text_style.code);
        assert!(!text.text_style.emphasis);
    }

    #[test]
    fn runless_shaped_text_commands_preserve_cluster_paint_style() {
        let command = text_command("Name id");
        let mut output = Vec::new();
        let text = runless_shaped_text_paint();

        push_text_paint_commands(
            &mut output,
            &command,
            &text,
            frame_rect(),
            None,
            command.z_index,
        );

        let text_commands = output
            .iter()
            .filter(|command| command.text.is_some())
            .collect::<Vec<_>>();
        assert_eq!(text_commands.len(), 2);
        assert_eq!(text_commands[0].text.as_deref(), Some("Name "));
        assert!(!text_commands[0].text_style.code);
        assert_eq!(text_commands[1].text.as_deref(), Some("id"));
        assert!(text_commands[1].text_style.code);
    }

    fn text_command(text: &str) -> UiRenderCommand {
        UiRenderCommand {
            node_id: UiNodeId::new(7),
            kind: UiRenderCommandKind::Text,
            frame: ui_frame(),
            clip_frame: None,
            z_index: 11,
            style: UiResolvedStyle {
                foreground_color: Some("#ffffffff".to_string()),
                font_size: 12.0,
                line_height: 15.0,
                ..UiResolvedStyle::default()
            },
            text_layout: None,
            text: Some(text.to_string()),
            image: None,
            opacity: 1.0,
        }
    }

    fn runless_shaped_text_paint() -> UiTextPaint {
        UiTextPaint {
            source_text: "Name id".to_string(),
            color: Some("#ffffffff".to_string()),
            font: None,
            font_family: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
            font_size: 12.0,
            line_height: 15.0,
            writing_mode: UiTextWritingMode::HorizontalTb,
            render_mode: UiTextRenderMode::Native,
            overflow: UiTextOverflow::Clip,
            shaped: Some(UiShapedText {
                source_text: "Name id".to_string(),
                source_range: text_range(0, 7),
                direction: UiTextDirection::LeftToRight,
                overflow: UiTextOverflow::Clip,
                font_size: 12.0,
                line_height: 15.0,
                measured_width: 42.0,
                measured_height: 15.0,
                writing_mode: UiTextWritingMode::HorizontalTb,
                render_mode: UiTextRenderMode::Native,
                font_key: None,
                atlas_resource: None,
                ellipsis_range: None,
                lines: vec![UiShapedTextLine {
                    text: "Name id".to_string(),
                    frame: ui_frame(),
                    source_range: text_range(0, 7),
                    visual_range: text_range(0, 7),
                    measured_width: 42.0,
                    baseline: 11.0,
                    direction: UiTextDirection::LeftToRight,
                    ellipsized: false,
                    glyphs: vec![
                        UiShapedGlyph::new(
                            1,
                            text_range(0, 5),
                            UiFrame::new(4.0, 5.0, 30.0, 15.0),
                            30.0,
                        ),
                        UiShapedGlyph::new(
                            2,
                            text_range(5, 7),
                            UiFrame::new(34.0, 5.0, 12.0, 15.0),
                            12.0,
                        ),
                    ],
                    clusters: vec![
                        UiShapedTextCluster {
                            kind: UiTextRunKind::Plain,
                            text: "Name ".to_string(),
                            source_range: text_range(0, 5),
                            visual_range: text_range(0, 5),
                            direction: UiTextDirection::LeftToRight,
                        },
                        UiShapedTextCluster {
                            kind: UiTextRunKind::Code,
                            text: "id".to_string(),
                            source_range: text_range(5, 7),
                            visual_range: text_range(5, 7),
                            direction: UiTextDirection::LeftToRight,
                        },
                    ],
                }],
            }),
            selection: None,
            caret: None,
            composition: None,
            decorations: Vec::new(),
            runs: Vec::new(),
        }
    }

    fn only_text_command(commands: &[HostPaintCommand]) -> &HostPaintCommand {
        let text_commands = commands
            .iter()
            .filter(|command| command.text.is_some())
            .collect::<Vec<_>>();
        assert_eq!(text_commands.len(), 1);
        text_commands[0]
    }

    const fn ui_frame() -> UiFrame {
        UiFrame::new(4.0, 5.0, 96.0, 18.0)
    }

    const fn frame_rect() -> FrameRect {
        FrameRect {
            x: 4.0,
            y: 5.0,
            width: 96.0,
            height: 18.0,
        }
    }

    const fn text_range(start: usize, end: usize) -> UiTextRange {
        UiTextRange { start, end }
    }
}
