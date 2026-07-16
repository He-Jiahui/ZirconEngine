use super::LineBreakChunk;
use crate::text::TextRange;

const WORD_SMART_TRAILING_PUNCTUATION: &[char] = &[
    ',', '.', ':', ';', '!', '?', '…', '‥', '‽', '‼', '⁇', '⁈', '⁉', '、', '。', '，', '．', '・',
    '：', '；', '！', '？', '،', '؛', '؟',
];
const WORD_SMART_CLOSING_DELIMITERS: &[char] = &[
    '"', '\'', '\u{2019}', '\u{201d}', '）', '］', '｝', '｠', '】', '〕', '〉', '》', '」', '』',
    '〗', '〙', '〛', '〟', '〞', '＂', '＇',
];

pub(super) fn apply_word_smart_rules<'a>(
    text: &'a str,
    chunks: Vec<LineBreakChunk<'a>>,
) -> Vec<LineBreakChunk<'a>> {
    let mut adjusted: Vec<LineBreakChunk<'a>> = Vec::with_capacity(chunks.len());

    for chunk in chunks {
        append_word_smart_chunk(text, chunk, &mut adjusted);
    }

    adjusted
}

fn append_word_smart_chunk<'a>(
    text: &'a str,
    mut chunk: LineBreakChunk<'a>,
    adjusted: &mut Vec<LineBreakChunk<'a>>,
) {
    loop {
        if let Some((prefix, suffix)) = split_after_leading_trailing_punctuation(text, chunk) {
            append_word_smart_leaf(text, prefix, adjusted);
            chunk = suffix;
            continue;
        }

        if let Some((prefix, suffix)) = split_after_internal_trailing_punctuation(text, chunk) {
            append_word_smart_leaf(text, prefix, adjusted);
            chunk = suffix;
            continue;
        }

        break;
    }

    append_word_smart_leaf(text, chunk, adjusted);
}

fn split_after_leading_trailing_punctuation<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    let (offset, ch) = chunk.text.char_indices().next()?;
    if offset != 0 || !is_word_smart_trailing_punctuation(ch) {
        return None;
    }

    let split_offset = word_smart_protected_run_end(chunk.text, offset, ch);
    if split_offset >= chunk.text.len() {
        return None;
    }

    split_word_smart_chunk(text, chunk, split_offset)
}

fn append_word_smart_leaf<'a>(
    text: &'a str,
    mut chunk: LineBreakChunk<'a>,
    adjusted: &mut Vec<LineBreakChunk<'a>>,
) {
    if starts_with_word_smart_trailing_punctuation(chunk.text) {
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
                return;
            }
        }
    }

    if has_word_smart_trailing_punctuation_suffix(chunk.text) {
        chunk.allow_glyph_fallback = false;
    }

    adjusted.push(chunk);
}

fn split_after_internal_trailing_punctuation<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    for (offset, ch) in chunk.text.char_indices() {
        let split_offset = word_smart_protected_run_end(chunk.text, offset, ch);
        if offset == 0
            || split_offset >= chunk.text.len()
            || !is_word_smart_trailing_punctuation(ch)
        {
            continue;
        }

        return split_word_smart_chunk(text, chunk, split_offset);
    }

    None
}

fn split_word_smart_chunk<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
    split_offset: usize,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    let split = chunk.visual_range.start + split_offset;
    let source_split = chunk.source_range.start + split_offset;
    if split >= chunk.visual_range.end || source_split > chunk.source_range.end {
        return None;
    }
    if !text.is_char_boundary(split) {
        return None;
    }

    let prefix = LineBreakChunk {
        text: &text[chunk.visual_range.start..split],
        visual_range: TextRange {
            start: chunk.visual_range.start,
            end: split,
        },
        source_range: TextRange {
            start: chunk.source_range.start,
            end: source_split,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        break_suffix: None,
    };
    let suffix = LineBreakChunk {
        text: &text[split..chunk.visual_range.end],
        visual_range: TextRange {
            start: split,
            end: chunk.visual_range.end,
        },
        source_range: TextRange {
            start: source_split,
            end: chunk.source_range.end,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        break_suffix: chunk.break_suffix,
    };

    Some((prefix, suffix))
}

fn starts_with_word_smart_trailing_punctuation(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(is_word_smart_trailing_punctuation)
}

fn has_word_smart_trailing_punctuation_suffix(text: &str) -> bool {
    for (offset, ch) in text.char_indices().rev() {
        if is_word_smart_closing_delimiter(ch) {
            continue;
        }
        return offset > 0 && is_word_smart_trailing_punctuation(ch);
    }

    false
}

fn word_smart_protected_run_end(text: &str, offset: usize, ch: char) -> usize {
    let mut end = offset + ch.len_utf8();
    if !is_word_smart_trailing_punctuation(ch) {
        return end;
    }

    while let Some(next) = text[end..].chars().next() {
        if !is_word_smart_closing_delimiter(next) {
            break;
        }
        end += next.len_utf8();
    }

    end
}

fn is_word_smart_trailing_punctuation(ch: char) -> bool {
    WORD_SMART_TRAILING_PUNCTUATION.contains(&ch)
}

fn is_word_smart_closing_delimiter(ch: char) -> bool {
    WORD_SMART_CLOSING_DELIMITERS.contains(&ch)
}

#[cfg(test)]
mod tests;
