use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextOverflow,
    UiTextRenderMode, UiTextWrap, UiTextWritingMode,
};

pub(super) fn parse_text_align(value: &str) -> Option<UiTextAlign> {
    if matches_ascii_alias(value, &["left"]) {
        Some(UiTextAlign::Left)
    } else if matches_ascii_alias(value, &["center", "middle"]) {
        Some(UiTextAlign::Center)
    } else if matches_ascii_alias(value, &["right"]) {
        Some(UiTextAlign::Right)
    } else if matches_ascii_alias(value, &["start"]) {
        Some(UiTextAlign::Start)
    } else if matches_ascii_alias(value, &["end"]) {
        Some(UiTextAlign::End)
    } else if matches_ascii_alias(value, &["justify", "justified"]) {
        Some(UiTextAlign::Justify)
    } else {
        None
    }
}

pub(super) fn parse_text_wrap(value: &str) -> Option<UiTextWrap> {
    if matches_ascii_alias(value, &["none", "off", "nowrap"]) {
        Some(UiTextWrap::None)
    } else if matches_ascii_alias(value, &["word", "normal"]) {
        Some(UiTextWrap::Word)
    } else if matches_ascii_alias(
        value,
        &["word_smart", "word-smart", "smart_word", "smart-word"],
    ) {
        Some(UiTextWrap::WordSmart)
    } else if matches_ascii_alias(value, &["glyph", "char", "character"]) {
        Some(UiTextWrap::Glyph)
    } else {
        None
    }
}

pub(super) fn parse_text_render_mode(value: &str) -> Option<UiTextRenderMode> {
    if matches_ascii_alias(value, &["auto", "default"]) {
        Some(UiTextRenderMode::Auto)
    } else if matches_ascii_alias(value, &["native", "glyphon"]) {
        Some(UiTextRenderMode::Native)
    } else if matches_ascii_alias(value, &["sdf"]) {
        Some(UiTextRenderMode::Sdf)
    } else if matches_ascii_alias(value, &["msdf"]) {
        Some(UiTextRenderMode::Msdf)
    } else if matches_ascii_alias(value, &["mtsdf"]) {
        Some(UiTextRenderMode::Mtsdf)
    } else {
        None
    }
}

pub(super) fn parse_rich_text_format(value: &str) -> Option<UiRichTextFormat> {
    if matches_ascii_alias(value, &["plain"]) {
        Some(UiRichTextFormat::Plain)
    } else if matches_ascii_alias(value, &["markdown_inline_v1"]) {
        Some(UiRichTextFormat::MarkdownInlineV1)
    } else if matches_ascii_alias(value, &["bbcode_v1"]) {
        Some(UiRichTextFormat::BbCodeV1)
    } else if matches_ascii_alias(value, &["html_subset_v1"]) {
        Some(UiRichTextFormat::HtmlSubsetV1)
    } else {
        None
    }
}

pub(super) fn parse_text_direction(value: &str) -> Option<UiTextDirection> {
    if matches_ascii_alias(value, &["auto", "default"]) {
        Some(UiTextDirection::Auto)
    } else if matches_ascii_alias(value, &["ltr", "left_to_right", "left-to-right"]) {
        Some(UiTextDirection::LeftToRight)
    } else if matches_ascii_alias(value, &["rtl", "right_to_left", "right-to-left"]) {
        Some(UiTextDirection::RightToLeft)
    } else if matches_ascii_alias(value, &["mixed"]) {
        Some(UiTextDirection::Mixed)
    } else {
        None
    }
}

pub(super) fn parse_text_writing_mode(value: &str) -> Option<UiTextWritingMode> {
    if matches_ascii_alias(value, &["horizontal", "horizontal_tb", "horizontal-tb"]) {
        Some(UiTextWritingMode::HorizontalTb)
    } else if matches_ascii_alias(value, &["vertical", "vertical_rl", "vertical-rl"]) {
        Some(UiTextWritingMode::VerticalRl)
    } else {
        None
    }
}

pub(super) fn parse_text_overflow(value: &str) -> Option<UiTextOverflow> {
    if matches_ascii_alias(value, &["clip", "clipped"]) {
        Some(UiTextOverflow::Clip)
    } else if matches_ascii_alias(value, &["ellipsis", "truncate"]) {
        Some(UiTextOverflow::Ellipsis)
    } else if matches_ascii_alias(
        value,
        &[
            "ellipsis_word",
            "word_ellipsis",
            "trim_word_ellipsis",
            "truncate_word",
        ],
    ) {
        Some(UiTextOverflow::EllipsisWord)
    } else if matches_ascii_alias(
        value,
        &["ellipsis_start", "start_ellipsis", "truncate_start"],
    ) {
        Some(UiTextOverflow::EllipsisStart)
    } else if matches_ascii_alias(
        value,
        &["ellipsis_middle", "middle_ellipsis", "truncate_middle"],
    ) {
        Some(UiTextOverflow::EllipsisMiddle)
    } else if matches_ascii_alias(
        value,
        &[
            "shrink_to_fit",
            "shrink-to-fit",
            "shrink",
            "fit",
            "scale_down",
        ],
    ) {
        Some(UiTextOverflow::ShrinkToFit)
    } else if matches_ascii_alias(
        value,
        &[
            "clamp_font_size",
            "clamp-font-size",
            "font_size_clamp",
            "font-size-clamp",
            "clamp",
        ],
    ) {
        Some(default_clamp_font_size_overflow())
    } else {
        None
    }
}

pub(super) fn clamp_font_size_overflow(min_px: Option<f32>, max_px: Option<f32>) -> UiTextOverflow {
    let min_px = min_px.unwrap_or(UiResolvedStyle::DEFAULT_FONT_SIZE);
    let max_px = max_px.unwrap_or(UiResolvedStyle::DEFAULT_FONT_SIZE);
    UiTextOverflow::ClampFontSize { min_px, max_px }
}

fn default_clamp_font_size_overflow() -> UiTextOverflow {
    UiTextOverflow::ClampFontSize {
        min_px: UiResolvedStyle::DEFAULT_FONT_SIZE,
        max_px: UiResolvedStyle::DEFAULT_FONT_SIZE,
    }
}

fn matches_ascii_alias(value: &str, aliases: &[&str]) -> bool {
    let value = value.trim();
    aliases
        .iter()
        .any(|alias| value.eq_ignore_ascii_case(alias))
}
