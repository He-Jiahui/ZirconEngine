use serde::{Deserialize, Serialize};

use crate::ui::layout::UiFrame;

use super::{UiResolvedStyle, UiTextAlign, UiTextWrap};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLayout {
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub font_size: f32,
    pub line_height: f32,
    pub lines: Vec<UiResolvedTextLine>,
    pub overflow_clipped: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLine {
    pub text: String,
    pub frame: UiFrame,
}

pub fn layout_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
) -> UiResolvedTextLayout {
    let font_size = style.font_size.max(1.0);
    let line_height = style.line_height.max(font_size);
    let char_advance = (font_size * 0.5).max(1.0);
    let max_chars = (frame.width.max(char_advance) / char_advance)
        .floor()
        .max(1.0) as usize;
    let lines = wrap_text_lines(text, style.wrap, max_chars);
    let clip = clip_frame.unwrap_or(frame);

    let mut resolved_lines = Vec::new();
    let mut overflow_clipped = false;
    for (index, line) in lines.iter().enumerate() {
        let y = frame.y + index as f32 * line_height;
        let line_width = measure_line_width(line, char_advance).min(frame.width.max(0.0));
        let line_frame = UiFrame::new(
            aligned_x(frame, line_width, style.text_align),
            y,
            line_width,
            line_height,
        );
        if line_frame.intersection(clip).is_some() {
            resolved_lines.push(UiResolvedTextLine {
                text: line.clone(),
                frame: line_frame,
            });
        } else {
            overflow_clipped = true;
        }
    }

    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        font_size,
        line_height,
        lines: resolved_lines,
        overflow_clipped,
    }
}

fn wrap_text_lines(text: &str, wrap: UiTextWrap, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for segment in text.lines() {
        match wrap {
            UiTextWrap::None => lines.push(segment.to_string()),
            UiTextWrap::Word => push_word_wrapped_lines(&mut lines, segment, max_chars),
            UiTextWrap::Glyph => push_glyph_wrapped_lines(&mut lines, segment, max_chars),
        }
    }
    if lines.is_empty() && !text.is_empty() {
        lines.push(text.to_string());
    }
    lines
}

fn push_word_wrapped_lines(lines: &mut Vec<String>, text: &str, max_chars: usize) {
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            push_wrapped_word(lines, &mut current, word, max_chars);
            continue;
        }
        let next_len = current.chars().count() + 1 + word.chars().count();
        if next_len <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            push_wrapped_word(lines, &mut current, word, max_chars);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
}

fn push_wrapped_word(lines: &mut Vec<String>, current: &mut String, word: &str, max_chars: usize) {
    if word.chars().count() <= max_chars {
        current.push_str(word);
        return;
    }

    let chars = word.chars().collect::<Vec<_>>();
    let chunks = chars.chunks(max_chars).collect::<Vec<_>>();
    for (index, chunk) in chunks.iter().enumerate() {
        let chunk = chunk.iter().copied().collect::<String>();
        if index + 1 == chunks.len() {
            current.push_str(&chunk);
        } else {
            lines.push(chunk);
        }
    }
}

fn push_glyph_wrapped_lines(lines: &mut Vec<String>, text: &str, max_chars: usize) {
    let mut current = String::new();
    for ch in text.chars() {
        if current.chars().count() >= max_chars {
            lines.push(std::mem::take(&mut current));
        }
        current.push(ch);
    }
    if !current.is_empty() {
        lines.push(current);
    }
}

fn measure_line_width(line: &str, char_advance: f32) -> f32 {
    line.chars().count() as f32 * char_advance
}

fn aligned_x(frame: UiFrame, line_width: f32, align: UiTextAlign) -> f32 {
    match align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - line_width) * 0.5,
        UiTextAlign::Right => frame.right() - line_width,
    }
}
