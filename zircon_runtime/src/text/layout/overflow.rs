use unicode_segmentation::UnicodeSegmentation;

pub(crate) const ELLIPSIS: &str = "…";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EllipsisPlacement {
    End,
    EndWord,
    Start,
    Middle,
}

pub(crate) fn retained_grapheme_counts(
    text: &str,
    graphemes: &[(usize, usize)],
    advances: &[f32],
    available: f32,
    placement: EllipsisPlacement,
) -> (usize, usize) {
    match placement {
        EllipsisPlacement::Start => (0, fitting_suffix_count(advances, available)),
        EllipsisPlacement::Middle => middle_retained_grapheme_counts(advances, available),
        EllipsisPlacement::EndWord => {
            let fitted = fitting_prefix_count(advances, available);
            let fitted_end = graphemes
                .get(fitted.saturating_sub(1))
                .map(|(_, end)| *end)
                .unwrap_or_default();
            let word_end = word_ellipsis_prefix_end(text, fitted_end);
            (graphemes.partition_point(|&(_, end)| end <= word_end), 0)
        }
        EllipsisPlacement::End => (fitting_prefix_count(advances, available), 0),
    }
}

fn middle_retained_grapheme_counts(advances: &[f32], available: f32) -> (usize, usize) {
    let mut prefix_count = 0;
    let mut suffix_count = 0;
    let mut retained_width = 0.0;
    let mut prefer_suffix = true;

    loop {
        let remaining = advances.len().saturating_sub(prefix_count + suffix_count);
        if remaining == 0 {
            break;
        }

        let next_index = if prefer_suffix {
            advances.len() - suffix_count - 1
        } else {
            prefix_count
        };
        let next_width = advances[next_index];
        if retained_width + next_width <= available {
            retained_width += next_width;
            if prefer_suffix {
                suffix_count += 1;
            } else {
                prefix_count += 1;
            }
            prefer_suffix = !prefer_suffix;
            continue;
        }

        if !prefer_suffix {
            break;
        }

        let prefix_width = advances[prefix_count];
        if retained_width + prefix_width > available {
            break;
        }
        retained_width += prefix_width;
        prefix_count += 1;
        prefer_suffix = false;
    }

    (prefix_count, suffix_count)
}

pub(crate) fn trim_end_ellipsis_trailing_graphemes(
    text: &str,
    graphemes: &[(usize, usize)],
    prefix_count: &mut usize,
    placement: EllipsisPlacement,
) {
    if !matches!(
        placement,
        EllipsisPlacement::End | EllipsisPlacement::EndWord
    ) {
        return;
    }
    while *prefix_count > 0 {
        let (start, end) = graphemes[*prefix_count - 1];
        if !text[start..end].chars().all(char::is_whitespace) {
            break;
        }
        *prefix_count -= 1;
    }
}

fn word_ellipsis_prefix_end(text: &str, end: usize) -> usize {
    let Some(prefix) = text.get(..end) else {
        return 0;
    };
    let trimmed_end = prefix.trim_end_matches(char::is_whitespace).len();
    if trimmed_end == 0 {
        return 0;
    }
    if trimmed_end < end
        || text
            .get(end..)
            .and_then(|suffix| suffix.chars().next())
            .map_or(true, char::is_whitespace)
    {
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

fn fitting_prefix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .take_while(|advance| {
            let fits = width + **advance <= available;
            if fits {
                width += **advance;
            }
            fits
        })
        .count()
}

fn fitting_suffix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .rev()
        .take_while(|advance| {
            let fits = width + **advance <= available;
            if fits {
                width += **advance;
            }
            fits
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::{retained_grapheme_counts, EllipsisPlacement};
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn end_word_ellipsis_drops_an_incomplete_first_word() {
        let text = "alpha beta";
        let graphemes = text
            .grapheme_indices(true)
            .map(|(start, grapheme)| (start, start + grapheme.len()))
            .collect::<Vec<_>>();
        let advances = vec![1.0; graphemes.len()];

        assert_eq!(
            retained_grapheme_counts(text, &graphemes, &advances, 8.0, EllipsisPlacement::EndWord),
            (5, 0),
            "the incomplete `be` prefix must not survive a word ellipsis"
        );
    }

    #[test]
    fn middle_ellipsis_preserves_the_existing_suffix_first_selection_order() {
        let text = "abcd";
        let graphemes = text
            .grapheme_indices(true)
            .map(|(start, grapheme)| (start, start + grapheme.len()))
            .collect::<Vec<_>>();

        assert_eq!(
            retained_grapheme_counts(
                text,
                &graphemes,
                &[1.0, 5.0, 5.0, 1.0],
                6.0,
                EllipsisPlacement::Middle,
            ),
            (1, 1),
            "middle ellipsis must keep the old suffix-first alternating selection policy"
        );
    }
}
