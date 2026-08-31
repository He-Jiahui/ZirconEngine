mod entry;
mod fallback;
mod metrics;
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
        UiTextRange, UiTextRenderMode, UiTextRunKind, UiTextRunPaintStyle, UiTextShapeArtifact,
        UiTextWritingMode,
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

    #[test]
    fn runless_same_style_shaped_text_preserves_line_as_single_command() {
        let command = text_command("folder-op\u{2026}line.svg");
        let mut output = Vec::new();
        let text = runless_fragmented_plain_shaped_text_paint();

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
        assert_eq!(
            text_commands.len(),
            1,
            "same-style shaped clusters should not be re-laid out as separate retained-host text commands"
        );
        assert_eq!(
            text_commands[0].text.as_deref(),
            Some("folder-op\u{2026}line.svg")
        );
        assert_eq!(text_commands[0].frame, FRAGMENTED_LINE_FRAME.host());
        assert_eq!(text_commands[0].text_style, UiTextRunPaintStyle::default());
    }

    #[derive(Clone, Copy)]
    struct TestFrameSpec {
        x_px: u16,
        y_px: u16,
        width_px: u16,
        height_px: u16,
    }

    impl TestFrameSpec {
        const fn ui(self) -> UiFrame {
            UiFrame::new(
                self.x_px as f32,
                self.y_px as f32,
                self.width_px as f32,
                self.height_px as f32,
            )
        }

        const fn host(self) -> FrameRect {
            FrameRect {
                x: self.x_px as f32,
                y: self.y_px as f32,
                width: self.width_px as f32,
                height: self.height_px as f32,
            }
        }
    }

    #[derive(Clone, Copy)]
    struct TestTextMetrics {
        font_size_px: u16,
        line_height_px: u16,
        baseline_px: u16,
    }

    impl TestTextMetrics {
        const fn font_size(self) -> f32 {
            self.font_size_px as f32
        }

        const fn line_height(self) -> f32 {
            self.line_height_px as f32
        }

        const fn baseline(self) -> f32 {
            self.baseline_px as f32
        }
    }

    const TEXT_COMMAND_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 4,
        y_px: 5,
        width_px: 96,
        height_px: 18,
    };
    const RUNLESS_TEXT_METRICS: TestTextMetrics = TestTextMetrics {
        font_size_px: 12,
        line_height_px: 15,
        baseline_px: 11,
    };
    const FRAGMENTED_TEXT_METRICS: TestTextMetrics = TestTextMetrics {
        font_size_px: 13,
        line_height_px: 16,
        baseline_px: 12,
    };
    const RUNLESS_LINE_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 4,
        y_px: 5,
        width_px: 42,
        height_px: 15,
    };
    const RUNLESS_NAME_GLYPH_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 4,
        y_px: 5,
        width_px: 30,
        height_px: 15,
    };
    const RUNLESS_CODE_GLYPH_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 34,
        y_px: 5,
        width_px: 12,
        height_px: 15,
    };
    const FRAGMENTED_LINE_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 4,
        y_px: 5,
        width_px: 112,
        height_px: 15,
    };
    const FRAGMENTED_PREFIX_GLYPH_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 4,
        y_px: 5,
        width_px: 56,
        height_px: 15,
    };
    const FRAGMENTED_ELLIPSIS_GLYPH_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 60,
        y_px: 5,
        width_px: 8,
        height_px: 15,
    };
    const FRAGMENTED_SUFFIX_GLYPH_FRAME: TestFrameSpec = TestFrameSpec {
        x_px: 68,
        y_px: 5,
        width_px: 48,
        height_px: 15,
    };

    fn text_command(text: &str) -> UiRenderCommand {
        UiRenderCommand {
            node_id: UiNodeId::new(7),
            kind: UiRenderCommandKind::Text,
            frame: ui_frame(),
            clip_frame: None,
            z_index: 11,
            style: UiResolvedStyle {
                foreground_color: Some("#ffffffff".to_string()),
                font_size: RUNLESS_TEXT_METRICS.font_size(),
                line_height: RUNLESS_TEXT_METRICS.line_height(),
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
            font_size: RUNLESS_TEXT_METRICS.font_size(),
            line_height: RUNLESS_TEXT_METRICS.line_height(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            render_mode: UiTextRenderMode::Native,
            text_effects: Default::default(),
            text_decorations: Default::default(),
            overflow: UiTextOverflow::Clip,
            shaped: UiTextShapeArtifact::Canonical(UiShapedText {
                source_text: "Name id".to_string(),
                source_range: text_range(0, 7),
                direction: UiTextDirection::LeftToRight,
                overflow: UiTextOverflow::Clip,
                font_size: RUNLESS_TEXT_METRICS.font_size(),
                line_height: RUNLESS_TEXT_METRICS.line_height(),
                measured_width: RUNLESS_LINE_FRAME.width_px as f32,
                measured_height: RUNLESS_TEXT_METRICS.line_height(),
                writing_mode: UiTextWritingMode::HorizontalTb,
                render_mode: UiTextRenderMode::Native,
                font_key: None,
                atlas_resource: None,
                ellipsis_range: None,
                lines: vec![UiShapedTextLine {
                    text: "Name id".to_string(),
                    frame: RUNLESS_LINE_FRAME.ui(),
                    source_range: text_range(0, 7),
                    visual_range: text_range(0, 7),
                    measured_width: RUNLESS_LINE_FRAME.width_px as f32,
                    baseline: RUNLESS_TEXT_METRICS.baseline(),
                    direction: UiTextDirection::LeftToRight,
                    ellipsized: false,
                    glyphs: vec![
                        UiShapedGlyph::new(
                            1,
                            text_range(0, 5),
                            RUNLESS_NAME_GLYPH_FRAME.ui(),
                            RUNLESS_NAME_GLYPH_FRAME.width_px as f32,
                        ),
                        UiShapedGlyph::new(
                            2,
                            text_range(5, 7),
                            RUNLESS_CODE_GLYPH_FRAME.ui(),
                            RUNLESS_CODE_GLYPH_FRAME.width_px as f32,
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

    fn runless_fragmented_plain_shaped_text_paint() -> UiTextPaint {
        UiTextPaint {
            source_text: "folder-op\u{2026}line.svg".to_string(),
            color: Some("#ffffffff".to_string()),
            font: None,
            font_family: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
            font_size: FRAGMENTED_TEXT_METRICS.font_size(),
            line_height: FRAGMENTED_TEXT_METRICS.line_height(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            render_mode: UiTextRenderMode::Native,
            text_effects: Default::default(),
            text_decorations: Default::default(),
            overflow: UiTextOverflow::Ellipsis,
            shaped: UiTextShapeArtifact::Canonical(UiShapedText {
                source_text: "folder-open-line.svg".to_string(),
                source_range: text_range(0, 20),
                direction: UiTextDirection::LeftToRight,
                overflow: UiTextOverflow::Ellipsis,
                font_size: FRAGMENTED_TEXT_METRICS.font_size(),
                line_height: FRAGMENTED_TEXT_METRICS.line_height(),
                measured_width: FRAGMENTED_LINE_FRAME.width_px as f32,
                measured_height: FRAGMENTED_TEXT_METRICS.line_height(),
                writing_mode: UiTextWritingMode::HorizontalTb,
                render_mode: UiTextRenderMode::Native,
                font_key: None,
                atlas_resource: None,
                ellipsis_range: Some(text_range(9, 14)),
                lines: vec![UiShapedTextLine {
                    text: "folder-op\u{2026}line.svg".to_string(),
                    frame: FRAGMENTED_LINE_FRAME.ui(),
                    source_range: text_range(0, 20),
                    visual_range: text_range(0, 20),
                    measured_width: FRAGMENTED_LINE_FRAME.width_px as f32,
                    baseline: FRAGMENTED_TEXT_METRICS.baseline(),
                    direction: UiTextDirection::LeftToRight,
                    ellipsized: true,
                    glyphs: vec![
                        UiShapedGlyph::new(
                            1,
                            text_range(0, 9),
                            FRAGMENTED_PREFIX_GLYPH_FRAME.ui(),
                            FRAGMENTED_PREFIX_GLYPH_FRAME.width_px as f32,
                        ),
                        UiShapedGlyph::new(
                            2,
                            text_range(9, 14),
                            FRAGMENTED_ELLIPSIS_GLYPH_FRAME.ui(),
                            FRAGMENTED_ELLIPSIS_GLYPH_FRAME.width_px as f32,
                        ),
                        UiShapedGlyph::new(
                            3,
                            text_range(14, 20),
                            FRAGMENTED_SUFFIX_GLYPH_FRAME.ui(),
                            FRAGMENTED_SUFFIX_GLYPH_FRAME.width_px as f32,
                        ),
                    ],
                    clusters: vec![
                        UiShapedTextCluster {
                            kind: UiTextRunKind::Plain,
                            text: "folder-op".to_string(),
                            source_range: text_range(0, 9),
                            visual_range: text_range(0, 9),
                            direction: UiTextDirection::LeftToRight,
                        },
                        UiShapedTextCluster {
                            kind: UiTextRunKind::Plain,
                            text: "\u{2026}".to_string(),
                            source_range: text_range(9, 14),
                            visual_range: text_range(9, 12),
                            direction: UiTextDirection::LeftToRight,
                        },
                        UiShapedTextCluster {
                            kind: UiTextRunKind::Plain,
                            text: "line.svg".to_string(),
                            source_range: text_range(14, 20),
                            visual_range: text_range(12, 20),
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
        TEXT_COMMAND_FRAME.ui()
    }

    const fn frame_rect() -> FrameRect {
        TEXT_COMMAND_FRAME.host()
    }

    const fn text_range(start: usize, end: usize) -> UiTextRange {
        UiTextRange { start, end }
    }
}
