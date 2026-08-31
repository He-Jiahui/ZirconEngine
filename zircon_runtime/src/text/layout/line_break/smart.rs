use super::LineBreakChunk;
use crate::text::{TextRange, WordBoundaryMap};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

pub(super) fn apply_word_smart_rules<'a>(
    text: &'a str,
    chunks: Vec<LineBreakChunk<'a>>,
) -> Vec<LineBreakChunk<'a>> {
    let mut adjusted: Vec<LineBreakChunk<'a>> = Vec::with_capacity(chunks.len());
    let mut word_ends = WordEndCursor::new(WordBoundaryMap::new(text).ranges());

    for chunk in chunks {
        append_word_smart_chunk(text, chunk, &mut adjusted, &mut word_ends);
    }

    adjusted
}

fn append_word_smart_chunk<'a, I>(
    text: &'a str,
    mut chunk: LineBreakChunk<'a>,
    adjusted: &mut Vec<LineBreakChunk<'a>>,
    word_ends: &mut WordEndCursor<I>,
) where
    I: Iterator<Item = TextRange>,
{
    loop {
        if let Some((prefix, suffix)) = split_after_leading_trailing_punctuation(text, chunk) {
            append_word_smart_leaf(text, prefix, adjusted, word_ends);
            chunk = suffix;
            continue;
        }

        if let Some((prefix, suffix)) =
            split_after_internal_trailing_punctuation(text, chunk, word_ends)
        {
            append_word_smart_leaf(text, prefix, adjusted, word_ends);
            chunk = suffix;
            continue;
        }

        break;
    }

    append_word_smart_leaf(text, chunk, adjusted, word_ends);
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

fn append_word_smart_leaf<'a, I>(
    text: &'a str,
    mut chunk: LineBreakChunk<'a>,
    adjusted: &mut Vec<LineBreakChunk<'a>>,
    word_ends: &mut WordEndCursor<I>,
) where
    I: Iterator<Item = TextRange>,
{
    if starts_with_word_smart_trailing_punctuation(chunk.text)
        && word_ends.has_word_ending_at(chunk.visual_range.start)
    {
        if let Some(previous) = adjusted.last_mut().filter(|previous| {
            !previous.mandatory_break
                && has_isomorphic_source_mapping(previous)
                && has_isomorphic_source_mapping(&chunk)
                && previous.visual_range.end == chunk.visual_range.start
                && previous.source_range.end == chunk.source_range.start
        }) {
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
                previous.mandatory_break |= chunk.mandatory_break;
                previous.allow_glyph_fallback = false;
                return;
            }
        }
    }

    if trailing_punctuation_suffix_start(chunk.text).is_some_and(|offset| {
        word_ends.has_word_ending_at(chunk.visual_range.start.saturating_add(offset))
    }) {
        chunk.allow_glyph_fallback = false;
    }

    adjusted.push(chunk);
}

fn split_after_internal_trailing_punctuation<'a, I>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
    word_ends: &mut WordEndCursor<I>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)>
where
    I: Iterator<Item = TextRange>,
{
    for (offset, ch) in chunk.text.char_indices() {
        let split_offset = word_smart_protected_run_end(chunk.text, offset, ch);
        if offset == 0
            || split_offset >= chunk.text.len()
            || !is_word_smart_trailing_punctuation(ch)
        {
            continue;
        }
        let source_offset = chunk.visual_range.start.saturating_add(offset);
        if !word_ends.has_word_ending_at(source_offset) {
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
    let visual_len = chunk
        .visual_range
        .end
        .saturating_sub(chunk.visual_range.start);
    if !has_isomorphic_source_mapping(&chunk) || split_offset >= visual_len {
        return None;
    }

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
        mandatory_break: false,
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
        mandatory_break: chunk.mandatory_break,
        break_suffix: chunk.break_suffix,
    };

    Some((prefix, suffix))
}

fn has_isomorphic_source_mapping(chunk: &LineBreakChunk<'_>) -> bool {
    let visual_len = chunk
        .visual_range
        .end
        .saturating_sub(chunk.visual_range.start);
    let source_len = chunk
        .source_range
        .end
        .saturating_sub(chunk.source_range.start);
    visual_len == source_len && visual_len == chunk.text.len()
}

fn starts_with_word_smart_trailing_punctuation(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(is_word_smart_trailing_punctuation)
}

fn trailing_punctuation_suffix_start(text: &str) -> Option<usize> {
    let mut trigger_start = None;
    for (offset, ch) in text.char_indices().rev() {
        if is_word_smart_trailing_extension(ch) {
            if is_word_smart_trailing_punctuation(ch) && offset > 0 {
                trigger_start = Some(offset);
            }
            continue;
        }
        break;
    }

    trigger_start
}

fn word_smart_protected_run_end(text: &str, offset: usize, ch: char) -> usize {
    let mut end = offset + ch.len_utf8();
    if !is_word_smart_trailing_punctuation(ch) {
        return end;
    }

    while let Some(next) = text[end..].chars().next() {
        if !is_word_smart_trailing_extension(next) {
            break;
        }
        end += next.len_utf8();
    }

    end
}

fn is_word_smart_trailing_punctuation(ch: char) -> bool {
    matches!(ch.general_category(), GeneralCategory::OtherPunctuation)
}

fn is_word_smart_trailing_extension(ch: char) -> bool {
    matches!(
        ch.general_category(),
        GeneralCategory::OtherPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::FinalPunctuation
    )
}

struct WordEndCursor<I> {
    ranges: I,
    next: Option<TextRange>,
    last_query: usize,
}

impl<I> WordEndCursor<I>
where
    I: Iterator<Item = TextRange>,
{
    fn new(mut ranges: I) -> Self {
        let next = ranges.next();
        Self {
            ranges,
            next,
            last_query: 0,
        }
    }

    fn has_word_ending_at(&mut self, offset: usize) -> bool {
        debug_assert!(offset >= self.last_query);
        self.last_query = offset;
        while self.next.is_some_and(|range| range.end < offset) {
            self.next = self.ranges.next();
        }
        self.next.is_some_and(|range| range.end == offset)
    }
}

#[cfg(test)]
mod tests;
