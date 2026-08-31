use unicode_segmentation::UnicodeSegmentation;

use super::{RichTextParseError, StyleOverride, StyledRun, range_contains, styled_run};

pub(super) fn align_runs_to_graphemes(text: &str, runs: &[StyledRun]) -> Vec<StyledRun> {
    align_runs_to_graphemes_bounded(text, runs, usize::MAX)
        .expect("unbounded test alignment must fit its representable run index")
}

pub(super) fn align_runs_to_graphemes_bounded(
    text: &str,
    runs: &[StyledRun],
    max_runs: usize,
) -> Result<Vec<StyledRun>, RichTextParseError> {
    if runs.len() > max_runs {
        return Err(RichTextParseError::RunCountBudgetExceeded {
            attempted_runs: runs.len(),
            max_runs,
        });
    }
    if ascii_runs_are_canonical(text, runs) {
        return Ok(runs.to_vec());
    }

    let mut aligned: Vec<StyledRun> = Vec::with_capacity(runs.len().min(max_runs));
    let mut run_index = 0;
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        while run_index < runs.len() && runs[run_index].byte_range.1 as usize <= start {
            run_index += 1;
        }
        let source = runs
            .get(run_index)
            .filter(|run| range_contains(run.byte_range, start));
        if let Some(previous) = aligned.last_mut() {
            if previous.byte_range.1 == start as u32 && source_metadata_matches(previous, source) {
                previous.byte_range.1 = end as u32;
                continue;
            }
        }
        if aligned.len() >= max_runs {
            return Err(RichTextParseError::RunCountBudgetExceeded {
                attempted_runs: aligned.len().saturating_add(1),
                max_runs,
            });
        }
        aligned.push(clone_source_run(start, end, source));
    }
    Ok(aligned)
}

fn ascii_runs_are_canonical(text: &str, runs: &[StyledRun]) -> bool {
    if !text.is_ascii() {
        return false;
    }
    let Ok(text_end) = u32::try_from(text.len()) else {
        return false;
    };
    let mut expected_start = 0;
    let mut previous = None;
    for run in runs {
        if run.byte_range.0 != expected_start
            || run.byte_range.1 <= expected_start
            || run.byte_range.1 > text_end
            || previous.is_some_and(|previous| run_metadata_matches(previous, run))
        {
            return false;
        }
        expected_start = run.byte_range.1;
        previous = Some(run);
    }
    expected_start == text_end
}

fn source_metadata_matches(previous: &StyledRun, source: Option<&StyledRun>) -> bool {
    source.map_or_else(
        || {
            previous.style == StyleOverride::default()
                && previous.inline.is_none()
                && previous.link.is_none()
        },
        |source| run_metadata_matches(previous, source),
    )
}

fn run_metadata_matches(left: &StyledRun, right: &StyledRun) -> bool {
    left.style == right.style && left.inline == right.inline && left.link == right.link
}

fn clone_source_run(start: usize, end: usize, source: Option<&StyledRun>) -> StyledRun {
    let Some(source) = source else {
        return styled_run(start as u32, end as u32, StyleOverride::default());
    };
    StyledRun {
        byte_range: (start as u32, end as u32),
        style: source.style.clone(),
        inline: source.inline.clone(),
        link: source.link.clone(),
    }
}
