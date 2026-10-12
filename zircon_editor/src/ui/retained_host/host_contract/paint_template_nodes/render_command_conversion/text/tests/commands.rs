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

#[test]
fn native_typed_short_line_height_preserves_source_family_and_exact_weight() {
    use crate::ui::retained_host::host_contract::paint_text::active_authored_font_request;
    let mut command = text_command("Native input");
    command.style.font_family = Some("Fira Sans".into());
    command.style.font_weight = 650;
    command.style.font_size = 17.0;
    command.style.line_height = 8.0;
    let host_commands = runtime_render_commands_to_host(&[command], None);
    let text = only_text_command(&host_commands);
    assert_eq!((text.font_size, text.line_height), (17.0, 8.0));
    let request = text
        .text_font_request
        .as_ref()
        .expect("typed source request");
    assert_eq!(request.family, "Fira Sans");
    assert_eq!(request.weight, 650);
    assert!(active_authored_font_request().is_none());
}

#[test]
fn rich_run_requests_survive_gpu_record_and_cpu_replay() {
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        chrome_command_from_recorded_for_test, paint_chrome_command_stream_to_frame,
        ui_surface_draw_list_from_stream, ChromeCommandStream,
    };
    use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
    use crate::ui::retained_host::host_contract::paint_text::{
        draw_text_with_size_and_style, font_request_for_face, HostTextFontFace,
        HostTextFontRequestScope, TextPaintEvidenceScope,
    };
    use zircon_runtime::rhi::UiSurfaceCommandKind;
    use zircon_runtime_interface::ui::surface::UiTextPaintRun;
    let mut command = text_command("Plain Strong Code Explicit");
    command.style.font_family = Some("Fira Sans".into());
    command.style.font_weight = 400;
    let mut text = runless_shaped_text_paint();
    text.font_family = command.style.font_family.clone();
    text.font_weight = 400;
    text.font_size = 17.0;
    text.line_height = 8.0;
    text.runs = [
        UiTextRunKind::Plain,
        UiTextRunKind::Strong,
        UiTextRunKind::Code,
        UiTextRunKind::Strong,
    ]
    .into_iter()
    .enumerate()
    .map(|(i, kind)| UiTextPaintRun {
        kind,
        text: ["Plain", "Strong", "Code", "Explicit"][i].into(),
        source_range: text_range(i, i + 1),
        visual_range: text_range(i, i + 1),
        frame: UiFrame {
            x: 8.0,
            y: 16.0 + i as f32 * 24.0,
            width: 160.0,
            height: 8.0,
        },
        color: Some("#ffffffff".into()),
        font: None,
        font_family: Some(
            if i == 3 {
                "Explicit family"
            } else {
                "Fira Sans"
            }
            .into(),
        ),
        font_weight: if i == 3 { 650 } else { 400 },
        font_size: 17.0,
        line_height: 8.0,
        style: UiTextRunPaintStyle::from_run_kind(kind),
    })
    .collect();
    let mut output = Vec::new();
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 120.0,
    };
    push_text_paint_commands(
        &mut output,
        &command,
        &text,
        frame_rect(),
        Some(clip.clone()),
        1,
    );
    let output = output
        .iter()
        .filter(|command| command.text.is_some())
        .collect::<Vec<_>>();
    assert_eq!(output.len(), 4);
    let strong = font_request_for_face(HostTextFontFace::UiStrong);
    let mono = font_request_for_face(HostTextFontFace::Mono);
    let expected = [
        ("Fira Sans".to_owned(), 400),
        ("Fira Sans".to_owned(), 400.max(strong.weight)),
        (mono.family, mono.weight),
        ("Explicit family".to_owned(), 650),
    ];
    let mut recording = HostRgbaFrame::recording_only(180, 120);
    for (command, (family, weight)) in output.iter().zip(&expected) {
        let request = command.text_font_request.as_ref().unwrap();
        assert_eq!((&request.family, request.weight), (family, *weight));
        assert_eq!((command.font_size, command.line_height), (17.0, 8.0));
        let _request = HostTextFontRequestScope::enter(Some(request));
        draw_text_with_size_and_style(
            &mut recording,
            command.frame.clone(),
            command.text.as_deref().unwrap(),
            Some(&clip),
            [255; 4],
            command.font_size,
            command.line_height,
            command.text_style,
        );
    }
    let mut stream = ChromeCommandStream::full_rebuild((180, 120));
    for command in recording.into_recorded_commands() {
        stream.push_command_for_test(
            chrome_command_from_recorded_for_test(command, true, true).unwrap(),
        );
    }
    let gpu = ui_surface_draw_list_from_stream(&stream);
    assert_eq!(gpu.commands.len(), 4);
    for (command, (family, weight)) in gpu.commands.iter().zip(&expected) {
        let UiSurfaceCommandKind::Text {
            font_family,
            font_weight,
            ..
        } = &command.kind
        else {
            panic!("GPU text");
        };
        assert_eq!(
            (font_family.as_ref().unwrap(), *font_weight),
            (family, *weight)
        );
    }
    let evidence = TextPaintEvidenceScope::begin().unwrap();
    let pixels = paint_chrome_command_stream_to_frame(180, 120, &stream);
    let evidence = evidence.finish().unwrap();
    assert!(pixels
        .as_bytes()
        .chunks_exact(4)
        .any(|pixel| pixel != [0, 0, 0, 255]));
    assert_eq!(evidence["runs"].as_array().unwrap().len(), 4);
    for (run, (family, weight)) in evidence["runs"].as_array().unwrap().iter().zip(&expected) {
        assert_eq!(run["fontFamily"], family.as_str());
        assert_eq!(run["fontWeight"], *weight);
    }
}
