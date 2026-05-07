use unicode_segmentation::UnicodeSegmentation;

// Centralize Unicode grapheme boundaries so layout scaffolds do not split
// combining marks or emoji clusters before real shaping backends take over.
pub(super) fn grapheme_count(text: &str) -> usize {
    text.graphemes(true).count()
}

pub(super) fn grapheme_indices(text: &str) -> impl Iterator<Item = (usize, &str)> + '_ {
    text.grapheme_indices(true)
}

pub(super) fn leading_grapheme_continuation_len(previous_text: &str, next_text: &str) -> usize {
    if previous_text.is_empty() || next_text.is_empty() {
        return 0;
    }

    let split = previous_text.len();
    let mut combined = String::with_capacity(previous_text.len() + next_text.len());
    combined.push_str(previous_text);
    combined.push_str(next_text);

    for (start, grapheme) in combined.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < split && split < end {
            return end - split;
        }
        if start >= split {
            break;
        }
    }

    0
}

pub(super) fn grapheme_prefix(text: &str, max_graphemes: usize) -> &str {
    if max_graphemes == 0 {
        return "";
    }
    let end = grapheme_indices(text)
        .map(|(index, grapheme)| index + grapheme.len())
        .nth(max_graphemes - 1)
        .unwrap_or(text.len());
    &text[..end]
}
