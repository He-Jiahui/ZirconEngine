use crate::text::layout_geometry::finite_sum;
use crate::text::WordBoundaryMap;

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
            let word_end = WordBoundaryMap::new(text).completed_prefix_end(fitted_end);
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
        if finite_sum([retained_width, next_width]) <= available {
            retained_width = finite_sum([retained_width, next_width]);
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
        if finite_sum([retained_width, prefix_width]) > available {
            break;
        }
        retained_width = finite_sum([retained_width, prefix_width]);
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

fn fitting_prefix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .take_while(|advance| {
            let fits = finite_sum([width, **advance]) <= available;
            if fits {
                width = finite_sum([width, **advance]);
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
            let fits = finite_sum([width, **advance]) <= available;
            if fits {
                width = finite_sum([width, **advance]);
            }
            fits
        })
        .count()
}

#[cfg(test)]
#[path = "tests/overflow.rs"]
mod tests;
