use super::line_break::LineBreakChunk;
use zircon_runtime_interface::ui::surface::UiTextRange;

pub(super) fn apply_kinsoku_start_rules<'a>(
    text: &'a str,
    chunks: Vec<LineBreakChunk<'a>>,
) -> Vec<LineBreakChunk<'a>> {
    let mut adjusted: Vec<LineBreakChunk<'a>> = Vec::with_capacity(chunks.len());
    let mut chunk_iter = chunks.into_iter().peekable();

    while let Some(mut chunk) = chunk_iter.next() {
        if starts_with_forbidden_line_start(chunk.text) {
            if let Some(previous) = adjusted.last_mut() {
                let start = previous.visual_range.start;
                let end = chunk.visual_range.end;
                if start < end
                    && end <= text.len()
                    && text.is_char_boundary(start)
                    && text.is_char_boundary(end)
                {
                    previous.text = &text[start..end];
                    previous.visual_range.end = end;
                    previous.source_range.end = chunk.source_range.end;
                    if previous.break_suffix.is_none() {
                        previous.break_suffix = chunk.break_suffix;
                    }
                    previous.allow_glyph_fallback = false;
                    continue;
                }
            }
        }

        if starts_with_forbidden_line_end(chunk.text) {
            if let Some(next) = chunk_iter.peek() {
                let start = chunk.visual_range.start;
                let end = next.visual_range.end;
                if start < end
                    && end <= text.len()
                    && text.is_char_boundary(start)
                    && text.is_char_boundary(end)
                {
                    let next = chunk_iter.next().expect("peeked next chunk must exist");
                    chunk.text = &text[start..end];
                    chunk.visual_range.end = end;
                    chunk.source_range.end = next.source_range.end;
                    if chunk.break_suffix.is_none() {
                        chunk.break_suffix = next.break_suffix;
                    }
                    chunk.allow_glyph_fallback = false;
                }
            }
        }
        if ends_with_forbidden_line_end(chunk.text) {
            if let Some((prefix, suffix)) = split_forbidden_line_end_suffix(text, chunk) {
                adjusted.push(prefix);
                chunk = suffix;
                if let Some(next) = chunk_iter.peek() {
                    let start = chunk.visual_range.start;
                    let end = next.visual_range.end;
                    if start < end
                        && end <= text.len()
                        && text.is_char_boundary(start)
                        && text.is_char_boundary(end)
                    {
                        let next = chunk_iter.next().expect("peeked next chunk must exist");
                        chunk.text = &text[start..end];
                        chunk.visual_range.end = end;
                        chunk.source_range.end = next.source_range.end;
                        if chunk.break_suffix.is_none() {
                            chunk.break_suffix = next.break_suffix;
                        }
                    }
                }
                chunk.allow_glyph_fallback = false;
            }
        }

        // Let closing punctuation overhang with the preceding glyph instead of
        // allowing glyph fallback to put it at the start of the next line.
        if has_protected_forbidden_prefix(chunk.text) || has_protected_forbidden_suffix(chunk.text)
        {
            chunk.allow_glyph_fallback = false;
        }

        adjusted.push(chunk);
    }

    adjusted
}

fn starts_with_forbidden_line_end(text: &str) -> bool {
    text.chars().next().is_some_and(is_forbidden_line_end)
}

fn ends_with_forbidden_line_end(text: &str) -> bool {
    text.chars().count() > 1 && text.chars().next_back().is_some_and(is_forbidden_line_end)
}

fn split_forbidden_line_end_suffix<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    let (suffix_offset, _) = chunk.text.char_indices().next_back()?;
    if suffix_offset == 0 {
        return None;
    }

    let suffix_start = chunk.visual_range.start + suffix_offset;
    if !text.is_char_boundary(suffix_start) {
        return None;
    }

    let prefix = LineBreakChunk {
        text: &text[chunk.visual_range.start..suffix_start],
        visual_range: UiTextRange {
            start: chunk.visual_range.start,
            end: suffix_start,
        },
        source_range: UiTextRange {
            start: chunk.source_range.start,
            end: suffix_start,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        break_suffix: None,
    };
    let suffix = LineBreakChunk {
        text: &text[suffix_start..chunk.visual_range.end],
        visual_range: UiTextRange {
            start: suffix_start,
            end: chunk.visual_range.end,
        },
        source_range: UiTextRange {
            start: suffix_start,
            end: chunk.source_range.end,
        },
        allow_glyph_fallback: false,
        break_suffix: chunk.break_suffix,
    };
    Some((prefix, suffix))
}

fn starts_with_forbidden_line_start(text: &str) -> bool {
    text.chars().next().is_some_and(is_forbidden_line_start)
}

fn has_protected_forbidden_prefix(text: &str) -> bool {
    text.chars().count() > 1 && text.chars().next().is_some_and(is_forbidden_line_end)
}

fn has_protected_forbidden_suffix(text: &str) -> bool {
    text.chars().count() > 1
        && text
            .chars()
            .next_back()
            .is_some_and(is_forbidden_line_start)
}

fn is_forbidden_line_start(ch: char) -> bool {
    matches!(
        ch,
        '、' | '。'
            | '，'
            | '．'
            | '・'
            | '：'
            | '；'
            | '！'
            | '？'
            | '）'
            | '］'
            | '｝'
            | '】'
            | '〕'
            | '〉'
            | '》'
            | '」'
            | '』'
            | '’'
            | '”'
            | 'ぁ'
            | 'ぃ'
            | 'ぅ'
            | 'ぇ'
            | 'ぉ'
            | 'っ'
            | 'ゃ'
            | 'ゅ'
            | 'ょ'
            | 'ゎ'
            | 'ァ'
            | 'ィ'
            | 'ゥ'
            | 'ェ'
            | 'ォ'
            | 'ッ'
            | 'ャ'
            | 'ュ'
            | 'ョ'
            | 'ヮ'
    )
}

fn is_forbidden_line_end(ch: char) -> bool {
    matches!(
        ch,
        '（' | '｛' | '［' | '【' | '〔' | '〈' | '《' | '「' | '『' | '‘' | '“'
    )
}
