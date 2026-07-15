use super::*;

#[path = "proof_commands/rich_table.rs"]
mod rich_table;

pub(super) fn proof_horizontal_rich_table() -> UiRenderCommand {
    rich_table::proof_horizontal_rich_table()
}

pub(super) fn proof_vertical_rich_table() -> UiRenderCommand {
    rich_table::proof_vertical_rich_table()
}

pub(super) fn proof_background(viewport_size: UVec2) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(100),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            background_color: Some("#0b1220".to_string()),
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    }
}

pub(super) fn proof_text(
    node_id: u64,
    frame: UiFrame,
    text: &str,
    direction: UiTextDirection,
    language: Option<&str>,
    render_mode: UiTextRenderMode,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame: None,
        z_index: 1,
        style: UiResolvedStyle {
            foreground_color: Some("#edf6ff".to_string()),
            font: Some("res://fonts/default.font.toml".to_string()),
            language: language.map(str::to_string),
            font_size: if matches!(
                render_mode,
                UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf
            ) {
                24.0
            } else {
                30.0
            },
            line_height: 40.0,
            text_align: if matches!(direction, UiTextDirection::RightToLeft) {
                UiTextAlign::Right
            } else {
                UiTextAlign::Left
            },
            text_direction: direction,
            wrap: UiTextWrap::None,
            text_render_mode: render_mode,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some(text.to_string()),
        image: None,
        opacity: 1.0,
    }
}

pub(super) fn proof_msdf_sharp_corner_sample() -> UiRenderCommand {
    proof_text(
        123,
        UiFrame::new(508.0, 594.0, 510.0, 58.0),
        "A/M/W · MSDF 尖角",
        UiTextDirection::LeftToRight,
        Some("zh-Hans"),
        UiTextRenderMode::Msdf,
    )
}

#[cfg(target_os = "windows")]
pub(super) fn proof_variable_font_instance_samples() -> [UiRenderCommand; 4] {
    const TEXT: &str = "WMWM AVATAR 2026";
    let mut narrow_label = proof_text(
        124,
        UiFrame::new(42.0, 1680.0, 470.0, 32.0),
        "Variable wdth=min · SDF",
        UiTextDirection::LeftToRight,
        Some("en"),
        UiTextRenderMode::Native,
    );
    narrow_label.style.font_size = 20.0;
    narrow_label.style.line_height = 28.0;

    let mut narrow = proof_text(
        125,
        UiFrame::new(42.0, 1720.0, 470.0, 72.0),
        TEXT,
        UiTextDirection::LeftToRight,
        Some("en"),
        UiTextRenderMode::Sdf,
    );
    narrow.style.font = Some(super::product_project_fixture::VARIABLE_FONT_ASSET_URI.to_string());
    narrow.style.font_family =
        Some(super::product_project_fixture::VARIABLE_FONT_NARROW_FAMILY.to_string());
    narrow.style.font_size = 34.0;
    narrow.style.line_height = 48.0;

    let mut wide_label = narrow_label.clone();
    wide_label.node_id = UiNodeId::new(126);
    wide_label.frame = UiFrame::new(568.0, 1680.0, 470.0, 32.0);
    wide_label.text = Some("Variable wdth=max · SDF".to_string());

    let mut wide = narrow.clone();
    wide.node_id = UiNodeId::new(127);
    wide.frame = UiFrame::new(568.0, 1720.0, 470.0, 72.0);
    wide.style.font_family =
        Some(super::product_project_fixture::VARIABLE_FONT_WIDE_FAMILY.to_string());

    [narrow_label, narrow, wide_label, wide]
}

pub(super) fn proof_rich_text(node_id: u64, frame: UiFrame, markup: &str) -> UiRenderCommand {
    proof_rich_text_with_direction(
        node_id,
        frame,
        markup,
        UiTextDirection::LeftToRight,
        Some("en"),
    )
}

pub(super) fn proof_bbcode_text(
    node_id: u64,
    frame: UiFrame,
    markup: &str,
    wrap: UiTextWrap,
) -> UiRenderCommand {
    let mut command = proof_text(
        node_id,
        frame,
        markup,
        UiTextDirection::LeftToRight,
        Some("en"),
        UiTextRenderMode::Native,
    );
    command.style.rich_text_format = UiRichTextFormat::BbCode;
    command.style.font_size = 22.0;
    command.style.line_height = 30.0;
    command.style.tab_size = 3.0;
    command.style.wrap = wrap;
    command.text_layout = Some(layout_text(markup, &command.style, frame, None));
    command
}

pub(super) fn proof_rich_text_with_direction(
    node_id: u64,
    frame: UiFrame,
    markup: &str,
    direction: UiTextDirection,
    language: Option<&str>,
) -> UiRenderCommand {
    let mut command = proof_text(
        node_id,
        frame,
        markup,
        direction,
        language,
        UiTextRenderMode::Native,
    );
    command.style.rich_text_format = UiRichTextFormat::Html;
    command.text_layout = Some(layout_text(markup, &command.style, frame, None));
    command
}

pub(super) fn proof_rich_text_with_wrap(
    node_id: u64,
    frame: UiFrame,
    markup: &str,
    wrap: UiTextWrap,
) -> UiRenderCommand {
    let mut command = proof_rich_text(node_id, frame, markup);
    command.style.font_size = 22.0;
    command.style.line_height = 32.0;
    command.style.wrap = wrap;
    command.text_layout = Some(layout_text(markup, &command.style, frame, None));
    command
}

pub(super) fn proof_rich_text_with_overflow(
    node_id: u64,
    frame: UiFrame,
    markup: &str,
    overflow: UiTextOverflow,
) -> UiRenderCommand {
    let mut command = proof_rich_text(node_id, frame, markup);
    command.style.font_size = 22.0;
    command.style.line_height = 36.0;
    command.style.text_overflow = overflow;
    command.text_layout = Some(layout_text(markup, &command.style, frame, None));
    command
}

pub(super) fn proof_vertical_text(node_id: u64, frame: UiFrame, text: &str) -> UiRenderCommand {
    let mut command = proof_text(
        node_id,
        frame,
        text,
        UiTextDirection::LeftToRight,
        Some("zh-Hans"),
        UiTextRenderMode::Sdf,
    );
    command.style.text_writing_mode = UiTextWritingMode::VerticalRl;
    command.style.font_family = Some("Microsoft YaHei UI".to_string());
    command.style.font_size = 30.0;
    command.style.line_height = 38.0;
    command.style.wrap = UiTextWrap::Word;
    command.text_layout = Some(layout_text(text, &command.style, frame, None));
    command
}

pub(super) fn proof_vertical_rich_text(
    node_id: u64,
    frame: UiFrame,
    markup: &str,
) -> UiRenderCommand {
    let mut command = proof_rich_text(node_id, frame, markup);
    command.style.text_writing_mode = UiTextWritingMode::VerticalRl;
    command.style.font_family = Some("Microsoft YaHei UI".to_string());
    command.style.font_size = 26.0;
    command.style.line_height = 42.0;
    command.style.wrap = UiTextWrap::Glyph;
    command.style.text_render_mode = UiTextRenderMode::Sdf;
    command.text_layout = Some(layout_text(markup, &command.style, frame, None));
    command
}

pub(super) fn proof_vertical_bbcode_paragraphs() -> UiRenderCommand {
    let frame = UiFrame::new(934.0, 468.0, 124.0, 350.0);
    let markup = "[p align=center indent=28][color=#64d8ff]首列缩进居中[/color][img=res://ui/rich-inline-checker.png][/p][p align=right]末端对齐验证[/p]";
    let mut command = proof_text(
        120,
        frame,
        markup,
        UiTextDirection::LeftToRight,
        Some("zh-Hans"),
        UiTextRenderMode::Sdf,
    );
    command.style.rich_text_format = UiRichTextFormat::BbCode;
    command.style.text_writing_mode = UiTextWritingMode::VerticalRl;
    command.style.font_family = Some("Microsoft YaHei UI".to_string());
    command.style.font_size = 24.0;
    command.style.line_height = 34.0;
    command.style.wrap = UiTextWrap::Glyph;
    command.text_layout = Some(layout_text(markup, &command.style, frame, None));
    command
}

pub(super) fn proof_native_sdf_parity() -> [UiRenderCommand; 2] {
    const TEXT: &str = "Native 与 SDF 共用段落布局：Alpha 世界 2026，换行、字距和包围盒保持一致。";
    let native_frame = UiFrame::new(42.0, 1470.0, 470.0, 190.0);
    let sdf_frame = UiFrame::new(568.0, 1470.0, 470.0, 190.0);
    let mut native = proof_text(
        121,
        native_frame,
        TEXT,
        UiTextDirection::LeftToRight,
        Some("zh-Hans"),
        UiTextRenderMode::Native,
    );
    native.style.font_family = Some("Microsoft YaHei UI".to_string());
    native.style.font_size = 24.0;
    native.style.line_height = 34.0;
    native.style.wrap = UiTextWrap::Glyph;
    native.text_layout = Some(layout_text(TEXT, &native.style, native_frame, None));

    let mut sdf = native.clone();
    sdf.node_id = UiNodeId::new(122);
    sdf.frame = sdf_frame;
    sdf.style.text_render_mode = UiTextRenderMode::Sdf;
    sdf.text_layout = Some(layout_text(TEXT, &sdf.style, sdf_frame, None));
    [native, sdf]
}

pub(super) fn proof_mixed_bidi_editable_geometry() -> UiRenderCommand {
    use zircon_runtime_interface::ui::surface::{
        UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextComposition, UiTextRange,
        UiTextSelection,
    };

    const TEXT: &str = "LTR abc אבג XYZ";
    let frame = UiFrame::new(42.0, 1830.0, 996.0, 104.0);
    let mut command = proof_text(
        128,
        frame,
        TEXT,
        UiTextDirection::LeftToRight,
        Some("he"),
        UiTextRenderMode::Native,
    );
    command.style.font_size = 30.0;
    command.style.line_height = 52.0;
    let mut layout = layout_text(TEXT, &command.style, frame, None);
    layout.editable = Some(UiEditableTextState {
        text: TEXT.to_string(),
        caret: UiTextCaret {
            offset: "LTR abc ".len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: Some(UiTextSelection {
            anchor: 0,
            focus: "LTR abc א".len(),
        }),
        composition: Some(UiTextComposition {
            range: UiTextRange {
                start: "LTR abc ".len(),
                end: "LTR abc אבג".len(),
            },
            text: "אבג".to_string(),
            restore_text: None,
        }),
        read_only: false,
    });
    command.text_layout = Some(layout);
    command
}
