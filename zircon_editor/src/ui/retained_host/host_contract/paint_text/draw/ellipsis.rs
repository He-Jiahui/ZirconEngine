use super::super::font::measure_fallback_text_width;

const ELLIPSIS: char = '\u{2026}';

/// 把单行标签裁到 `max_width` 以内:能放下就原样返回(借用),放不下就裁到
/// `max_width - 省略号宽` 并追加 `…`,让 `Scene Editor` 优雅降级为 `Scene…` 而不是被字形硬裁成 `Sce`。
///
/// 纯函数,只依赖嵌入字体的字形步进,便于单测。`max_width` 非正或文本本就放得下时零额外开销。
pub(super) fn ellipsize_single_line(
    text: &str,
    font_size: f32,
    max_width: f32,
) -> EllipsizedLabel<'_> {
    if !max_width.is_finite() || max_width <= 0.0 || !font_size.is_finite() || font_size <= 0.0 {
        return EllipsizedLabel::Borrowed(text);
    }
    if measure_fallback_text_width(text, font_size) <= max_width {
        return EllipsizedLabel::Borrowed(text);
    }

    let ellipsis_width = measure_fallback_text_width("\u{2026}", font_size);
    // 连省略号都放不下:退化为不带文本(由上层 clip 决定是否还画一截),不再硬塞字形。
    if ellipsis_width > max_width {
        return EllipsizedLabel::Owned(String::new());
    }

    let budget = max_width - ellipsis_width;
    let mut used = 0.0_f32;
    let mut prefix = String::new();
    for glyph in text.chars() {
        let advance = measure_fallback_text_width(&glyph.to_string(), font_size);
        if used + advance > budget {
            break;
        }
        used += advance;
        prefix.push(glyph);
    }
    prefix.push(ELLIPSIS);
    EllipsizedLabel::Owned(prefix)
}

/// 裁切结果:未裁切时借用原串,裁切时持有新串。`as_str` 给 `TextStyle` 用。
pub(super) enum EllipsizedLabel<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl EllipsizedLabel<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            EllipsizedLabel::Borrowed(text) => text,
            EllipsizedLabel::Owned(text) => text.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::font::measure_fallback_text_width;
    use super::*;

    const FONT_SIZE: f32 = 10.0;

    #[test]
    fn keeps_text_that_already_fits() {
        let width = measure_fallback_text_width("Scene", FONT_SIZE) + 1.0;
        let label = ellipsize_single_line("Scene", FONT_SIZE, width);
        assert!(matches!(label, EllipsizedLabel::Borrowed("Scene")));
    }

    #[test]
    fn appends_ellipsis_when_too_narrow() {
        let full = measure_fallback_text_width("Scene Editor", FONT_SIZE);
        let label = ellipsize_single_line("Scene Editor", FONT_SIZE, full * 0.5);
        let rendered = label.as_str().to_string();
        assert!(rendered.ends_with('\u{2026}'), "got {rendered:?}");
        assert!(rendered != "Scene Editor");
        // 省略后整体宽度不超过给定上限。
        assert!(measure_fallback_text_width(&rendered, FONT_SIZE) <= full * 0.5 + 0.01);
    }

    #[test]
    fn degrades_to_empty_when_even_ellipsis_does_not_fit() {
        let label = ellipsize_single_line("Scene", FONT_SIZE, 0.5);
        assert_eq!(label.as_str(), "");
    }

    #[test]
    fn non_positive_width_borrows_original() {
        let label = ellipsize_single_line("Scene", FONT_SIZE, 0.0);
        assert!(matches!(label, EllipsizedLabel::Borrowed("Scene")));
    }
}
