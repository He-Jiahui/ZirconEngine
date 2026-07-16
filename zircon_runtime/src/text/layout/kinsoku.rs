use super::line_break::LineBreakChunk;
use crate::text::TextRange;

const FORBIDDEN_LINE_START_CLOSING_PUNCTUATION: &[char] = &[
    '、', '。', '，', '．', '・', '：', '；', '！', '？', '）', '］', '｝', '｠', '】', '〕', '〉',
    '》', '」', '』', '〗', '〙', '〛', '’', '”', '〟', '〞',
];
const FORBIDDEN_LINE_START_SMALL_KANA: &[char] = &[
    'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'っ', 'ゃ', 'ゅ', 'ょ', 'ゎ', 'ゕ', 'ゖ', 'ァ', 'ィ', 'ゥ', 'ェ',
    'ォ', 'ッ', 'ャ', 'ュ', 'ョ', 'ヮ', 'ヵ', 'ヶ', 'ㇰ', 'ㇱ', 'ㇲ', 'ㇳ', 'ㇴ', 'ㇵ', 'ㇶ', 'ㇷ',
    'ㇸ', 'ㇹ', 'ㇺ', 'ㇻ', 'ㇼ', 'ㇽ', 'ㇾ', 'ㇿ',
];
const FORBIDDEN_LINE_START_HALFWIDTH: &[char] = &[
    '｡', '｣', '､', '･', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｯ', 'ｬ', 'ｭ', 'ｮ', 'ｰ', 'ﾞ', 'ﾟ',
];
const FORBIDDEN_LINE_START_JAPANESE_NON_STARTERS: &[char] =
    &['ー', '々', '〻', 'ゝ', 'ゞ', 'ヽ', 'ヾ'];
const FORBIDDEN_LINE_START_SPACING_VOICING_MARKS: &[char] = &['゛', '゜'];
const FORBIDDEN_LINE_START_JLREQ_HYPHENS: &[char] = &['‐', '〜', '゠', '–'];
const JLREQ_INSEPARABLE_PAIRS: &[(char, char)] = &[
    ('—', '—'),
    ('…', '…'),
    ('‥', '‥'),
    ('〳', '〵'),
    ('〴', '〵'),
];
const FORBIDDEN_LINE_END_OPENING_PUNCTUATION: &[char] = &[
    '（', '｛', '｟', '［', '【', '〔', '〈', '《', '「', '『', '〖', '〘', '〚', '‘', '“', '〝',
    '｢',
];

pub(super) fn apply_kinsoku_start_rules<'a>(
    text: &'a str,
    chunks: Vec<LineBreakChunk<'a>>,
) -> Vec<LineBreakChunk<'a>> {
    let mut adjusted: Vec<LineBreakChunk<'a>> = Vec::with_capacity(chunks.len());
    let mut chunk_iter = chunks.into_iter().peekable();

    while let Some(mut chunk) = chunk_iter.next() {
        if starts_with_forbidden_line_start(chunk.text)
            || completes_jlreq_inseparable_pair(adjusted.last(), chunk.text)
        {
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
                    if let Some(next) = chunk_iter.next() {
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
                        if let Some(next) = chunk_iter.next() {
                            chunk.text = &text[start..end];
                            chunk.visual_range.end = end;
                            chunk.source_range.end = next.source_range.end;
                            if chunk.break_suffix.is_none() {
                                chunk.break_suffix = next.break_suffix;
                            }
                        }
                    }
                }
                chunk.allow_glyph_fallback = false;
            }
        }

        // Let closing punctuation overhang with the preceding glyph instead of
        // allowing glyph fallback to put it at the start of the next line.
        if has_protected_forbidden_prefix(chunk.text)
            || has_protected_forbidden_suffix(chunk.text)
            || has_jlreq_inseparable_pair(chunk.text)
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
        visual_range: TextRange {
            start: chunk.visual_range.start,
            end: suffix_start,
        },
        source_range: TextRange {
            start: chunk.source_range.start,
            end: suffix_start,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        break_suffix: None,
    };
    let suffix = LineBreakChunk {
        text: &text[suffix_start..chunk.visual_range.end],
        visual_range: TextRange {
            start: suffix_start,
            end: chunk.visual_range.end,
        },
        source_range: TextRange {
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

fn completes_jlreq_inseparable_pair(
    previous: Option<&LineBreakChunk<'_>>,
    current_text: &str,
) -> bool {
    let Some(previous_char) = previous.and_then(|chunk| chunk.text.chars().next_back()) else {
        return false;
    };
    let Some(current_char) = current_text.chars().next() else {
        return false;
    };

    is_jlreq_inseparable_pair(previous_char, current_char)
}

fn has_jlreq_inseparable_pair(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(mut previous_char) = chars.next() else {
        return false;
    };

    for current_char in chars {
        if is_jlreq_inseparable_pair(previous_char, current_char) {
            return true;
        }
        previous_char = current_char;
    }

    false
}

fn is_jlreq_inseparable_pair(previous_char: char, current_char: char) -> bool {
    JLREQ_INSEPARABLE_PAIRS.contains(&(previous_char, current_char))
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
    FORBIDDEN_LINE_START_CLOSING_PUNCTUATION.contains(&ch)
        || FORBIDDEN_LINE_START_SMALL_KANA.contains(&ch)
        || FORBIDDEN_LINE_START_HALFWIDTH.contains(&ch)
        || FORBIDDEN_LINE_START_JAPANESE_NON_STARTERS.contains(&ch)
        || FORBIDDEN_LINE_START_SPACING_VOICING_MARKS.contains(&ch)
        || FORBIDDEN_LINE_START_JLREQ_HYPHENS.contains(&ch)
}

fn is_forbidden_line_end(ch: char) -> bool {
    FORBIDDEN_LINE_END_OPENING_PUNCTUATION.contains(&ch)
}

#[cfg(test)]
mod tests;
