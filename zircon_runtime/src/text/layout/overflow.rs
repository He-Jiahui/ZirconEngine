use unicode_segmentation::UnicodeSegmentation;

const OVERFLOW_EPSILON: f32 = 0.01;
pub(crate) const ELLIPSIS: &str = "…";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EllipsisPlacement {
    End,
    EndWord,
    Start,
    Middle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EllipsisSegment {
    Text { start: usize, end: usize },
    Ellipsis,
}

pub(crate) fn ellipsize_text<F>(
    text: &str,
    max_width: f32,
    placement: EllipsisPlacement,
    mut measure: F,
) -> Vec<EllipsisSegment>
where
    F: FnMut(&str) -> f32,
{
    if measure(ELLIPSIS) > max_width + OVERFLOW_EPSILON {
        return vec![EllipsisSegment::Ellipsis];
    }

    match placement {
        EllipsisPlacement::End => end_ellipsis(text, max_width, &mut measure),
        EllipsisPlacement::EndWord => end_word_ellipsis(text, max_width, &mut measure),
        EllipsisPlacement::Start => start_ellipsis(text, max_width, &mut measure),
        EllipsisPlacement::Middle => middle_ellipsis(text, max_width, &mut measure),
    }
}

fn end_ellipsis(
    text: &str,
    max_width: f32,
    measure: &mut dyn FnMut(&str) -> f32,
) -> Vec<EllipsisSegment> {
    let mut retained_end = 0;
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        let mut candidate = String::with_capacity(end + ELLIPSIS.len());
        candidate.push_str(&text[..end]);
        candidate.push_str(ELLIPSIS);
        if measure(&candidate) > max_width + OVERFLOW_EPSILON {
            break;
        }
        retained_end = end;
    }

    let mut segments = Vec::new();
    if retained_end > 0 {
        segments.push(EllipsisSegment::Text {
            start: 0,
            end: retained_end,
        });
    }
    segments.push(EllipsisSegment::Ellipsis);
    segments
}

fn end_word_ellipsis(
    text: &str,
    max_width: f32,
    measure: &mut dyn FnMut(&str) -> f32,
) -> Vec<EllipsisSegment> {
    let mut retained_end = 0;
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        let word_end = word_trim_end(text, end);
        if word_end == 0 {
            continue;
        }
        let mut candidate = String::with_capacity(word_end + ELLIPSIS.len());
        candidate.push_str(&text[..word_end]);
        candidate.push_str(ELLIPSIS);
        if measure(&candidate) > max_width + OVERFLOW_EPSILON {
            break;
        }
        retained_end = word_end;
    }

    let mut segments = Vec::new();
    if retained_end > 0 {
        segments.push(EllipsisSegment::Text {
            start: 0,
            end: retained_end,
        });
    }
    segments.push(EllipsisSegment::Ellipsis);
    segments
}

fn start_ellipsis(
    text: &str,
    max_width: f32,
    measure: &mut dyn FnMut(&str) -> f32,
) -> Vec<EllipsisSegment> {
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    let mut retained_start = text.len();
    for tail_count in 1..=graphemes.len() {
        let tail_start = graphemes[graphemes.len() - tail_count].0;
        let mut candidate = String::with_capacity(text.len() - tail_start + ELLIPSIS.len());
        candidate.push_str(ELLIPSIS);
        candidate.push_str(&text[tail_start..]);
        if measure(&candidate) > max_width + OVERFLOW_EPSILON {
            break;
        }
        retained_start = tail_start;
    }

    let mut segments = vec![EllipsisSegment::Ellipsis];
    if retained_start < text.len() {
        segments.push(EllipsisSegment::Text {
            start: retained_start,
            end: text.len(),
        });
    }
    segments
}

fn word_trim_end(text: &str, end: usize) -> usize {
    let prefix = &text[..end];
    let trimmed_end = prefix.trim_end_matches(char::is_whitespace).len();
    if trimmed_end == 0 {
        return 0;
    }
    if trimmed_end < end || text[end..].chars().next().map_or(true, char::is_whitespace) {
        return trimmed_end;
    }

    let mut seen_word = false;
    for (index, grapheme) in text[..trimmed_end].grapheme_indices(true).rev() {
        if grapheme.chars().all(char::is_whitespace) {
            if seen_word {
                return index;
            }
        } else {
            seen_word = true;
        }
    }
    0
}

fn middle_ellipsis(
    text: &str,
    max_width: f32,
    measure: &mut dyn FnMut(&str) -> f32,
) -> Vec<EllipsisSegment> {
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect::<Vec<_>>();
    if graphemes.is_empty() {
        return vec![EllipsisSegment::Ellipsis];
    }

    let mut head_count = 0;
    let mut tail_count = 0;
    let mut prefer_tail = true;
    loop {
        let remaining = graphemes.len().saturating_sub(head_count + tail_count);
        if remaining == 0 {
            break;
        }

        let next_head = if prefer_tail {
            head_count
        } else {
            head_count + 1
        };
        let next_tail = if prefer_tail {
            tail_count + 1
        } else {
            tail_count
        };
        if next_head + next_tail > graphemes.len() {
            break;
        }
        if !middle_candidate_fits(text, &graphemes, next_head, next_tail, max_width, measure) {
            if prefer_tail {
                prefer_tail = false;
                if head_count + 1 + tail_count <= graphemes.len()
                    && middle_candidate_fits(
                        text,
                        &graphemes,
                        head_count + 1,
                        tail_count,
                        max_width,
                        measure,
                    )
                {
                    head_count += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else if prefer_tail {
            tail_count += 1;
            prefer_tail = false;
        } else {
            head_count += 1;
            prefer_tail = true;
        }
    }

    let mut segments = Vec::new();
    if head_count > 0 {
        segments.push(EllipsisSegment::Text {
            start: 0,
            end: graphemes[head_count - 1].1,
        });
    }
    segments.push(EllipsisSegment::Ellipsis);
    if tail_count > 0 {
        let tail_start = graphemes[graphemes.len() - tail_count].0;
        segments.push(EllipsisSegment::Text {
            start: tail_start,
            end: text.len(),
        });
    }
    segments
}

fn middle_candidate_fits(
    text: &str,
    graphemes: &[(usize, usize)],
    head_count: usize,
    tail_count: usize,
    max_width: f32,
    measure: &mut dyn FnMut(&str) -> f32,
) -> bool {
    let mut candidate = String::with_capacity(text.len() + ELLIPSIS.len());
    if head_count > 0 {
        candidate.push_str(&text[..graphemes[head_count - 1].1]);
    }
    candidate.push_str(ELLIPSIS);
    if tail_count > 0 {
        candidate.push_str(&text[graphemes[graphemes.len() - tail_count].0..]);
    }
    measure(&candidate) <= max_width + OVERFLOW_EPSILON
}
