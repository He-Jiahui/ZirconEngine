mod profile;

use crate::text::SharedTextLayoutSession;
use crate::text::layout::{
    arabic_kashida_insertion_offsets_bounded, justify_line_advances,
    measure_line_width_with_provider, measure_line_with_provider,
    measured_grapheme_widths_with_provider, tab_aligned_advances,
    validate_arabic_tatweel_candidate,
};
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextAlign, UiTextDirection};

use super::candidate_line::{CandidateLine, insert_virtual_text};
use super::direction::is_rtl_direction;
use crate::text::text_style;
use profile::{ARABIC_TATWEEL_RECEIPT_COUNT_MISMATCH_CODE, ArabicTatweelLineProfile};

pub(super) const MIN_TEXT_FONT_SIZE: f32 = 1.0;
const ARABIC_TATWEEL: &str = "\u{0640}";
const MAX_ARABIC_TATWEELS_PER_LINE: usize = 32;
const MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS: usize = 5;
const TATWEEL_FIT_EPSILON: f32 = 0.01;

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArabicTatweelBudgetSnapshot {
    max_materialized_tatweels_per_line: usize,
    max_fit_measurements_per_line: usize,
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
const fn arabic_tatweel_budget_snapshot() -> ArabicTatweelBudgetSnapshot {
    ArabicTatweelBudgetSnapshot {
        max_materialized_tatweels_per_line: MAX_ARABIC_TATWEELS_PER_LINE,
        max_fit_measurements_per_line: MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS,
    }
}

pub(super) fn resolve_line_widths_with_provider(
    line: &CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
    canonical_advances: Option<Vec<f32>>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<(f32, Vec<f32>, f32)> {
    let natural_advances = match canonical_advances {
        Some(advances) => advances,
        None => match line_grapheme_advances_with_provider(&line.text, style, provider) {
            TextShapingOutcome::Ready(advances) => advances,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        },
    };
    let natural_width = natural_advances.iter().sum();
    if should_justify_line(line, style, frame_width, is_last_line) {
        if let Some(justified_advances) =
            justify_line_advances(&line.text, &natural_advances, natural_width, frame_width)
        {
            let justified_width = justified_advances.iter().sum();
            return TextShapingOutcome::Ready((justified_width, justified_advances, frame_width));
        }
    }

    let line_width = natural_width.min(frame_width);
    TextShapingOutcome::Ready((natural_width, natural_advances, line_width))
}

pub(super) fn materialize_arabic_tatweels_for_justified_line(
    line: &mut CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<()> {
    if !should_justify_line(line, style, frame_width, is_last_line) {
        return TextShapingOutcome::Ready(());
    }
    let natural_advances = match line_grapheme_advances_with_provider(&line.text, style, provider) {
        TextShapingOutcome::Ready(advances) => advances,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let natural_width = natural_advances.iter().sum();
    materialize_arabic_tatweels(line, natural_width, frame_width, style, provider).map(|_| ())
}

fn materialize_arabic_tatweels(
    line: &mut CandidateLine,
    natural_width: f32,
    target_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<bool> {
    let extra = target_width - natural_width;
    if !extra.is_finite() || extra <= TATWEEL_FIT_EPSILON {
        return TextShapingOutcome::Ready(false);
    }
    let insertion_offsets =
        arabic_kashida_insertion_offsets_bounded(&line.text, MAX_ARABIC_TATWEELS_PER_LINE);
    if insertion_offsets.is_empty() {
        return TextShapingOutcome::Ready(false);
    }

    let neutral_style = text_style(style);
    let tatweel_width =
        match measure_line_width_with_provider(ARABIC_TATWEEL, &neutral_style, provider) {
            TextShapingOutcome::Ready(width) => width,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    if !tatweel_width.is_finite() || tatweel_width <= TATWEEL_FIT_EPSILON {
        return TextShapingOutcome::Ready(false);
    }
    let requested_count = ((extra / tatweel_width).floor() as usize)
        .max(1)
        .min(MAX_ARABIC_TATWEELS_PER_LINE);
    let mut line_profile = ArabicTatweelLineProfile::new(requested_count);
    crate::profile_scope!("runtime", "text.layout", "arabic_tatweel_candidate_fit");

    let count = match bounded_arabic_tatweel_fit_count(
        requested_count,
        natural_width,
        target_width,
        |candidate_count| {
            let (candidate, offsets) =
                text_with_arabic_tatweels(&line.text, &insertion_offsets, candidate_count);
            line_profile.record_probe(candidate.len());
            measure_line_with_provider(&candidate, &neutral_style, provider).map(|measured| {
                match validate_arabic_tatweel_candidate(
                    &measured,
                    &candidate,
                    &offsets,
                    natural_width,
                ) {
                    Ok(receipt) if receipt.insertion_count() == candidate_count => {
                        line_profile.record_safe_candidate();
                        Some(receipt.width())
                    }
                    Ok(_) => {
                        line_profile.record_rejection(ARABIC_TATWEEL_RECEIPT_COUNT_MISMATCH_CODE);
                        None
                    }
                    Err(rejection) => {
                        line_profile.record_rejection(rejection.profile_code());
                        None
                    }
                }
            })
        },
    ) {
        TextShapingOutcome::Ready(Some(count)) => count,
        TextShapingOutcome::Ready(None) => return TextShapingOutcome::Ready(false),
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let (_, offsets) = text_with_arabic_tatweels(&line.text, &insertion_offsets, count);
    let mut materialized = line.clone();
    if offsets
        .into_iter()
        .all(|offset| insert_virtual_text(&mut materialized, offset, ARABIC_TATWEEL))
    {
        *line = materialized;
        line_profile.record_accepted(count);
        return TextShapingOutcome::Ready(true);
    }
    TextShapingOutcome::Ready(false)
}

fn bounded_arabic_tatweel_fit_count(
    requested_count: usize,
    natural_width: f32,
    target_width: f32,
    mut candidate_width: impl FnMut(usize) -> TextLayoutOutcome<Option<f32>>,
) -> TextLayoutOutcome<Option<usize>> {
    let extra = target_width - natural_width;
    if requested_count == 0 || !extra.is_finite() || extra <= TATWEEL_FIT_EPSILON {
        return TextShapingOutcome::Ready(None);
    }

    let mut count = requested_count;
    for probe_index in 0..MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS {
        if probe_index + 1 == MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS {
            count = 1;
        }
        let width = match candidate_width(count) {
            TextShapingOutcome::Ready(Some(width)) => width,
            TextShapingOutcome::Ready(None) if count == 1 => {
                return TextShapingOutcome::Ready(None);
            }
            TextShapingOutcome::Ready(None) => {
                count = 1;
                continue;
            }
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        if width.is_finite() && width <= target_width + TATWEEL_FIT_EPSILON {
            return TextShapingOutcome::Ready(Some(count));
        }
        if count == 1 {
            return TextShapingOutcome::Ready(None);
        }

        let materialized_extra = width - natural_width;
        let estimated_count =
            if materialized_extra.is_finite() && materialized_extra > TATWEEL_FIT_EPSILON {
                ((count as f32 * extra / materialized_extra).floor() as usize)
                    .clamp(1, count.saturating_sub(1))
            } else {
                1
            };
        count = estimated_count;
    }
    TextShapingOutcome::Ready(None)
}

fn text_with_arabic_tatweels(
    text: &str,
    insertion_offsets: &[usize],
    count: usize,
) -> (String, Vec<usize>) {
    let mut per_offset_counts = vec![0usize; insertion_offsets.len()];
    for index in 0..count {
        let offset_index = index % per_offset_counts.len();
        per_offset_counts[offset_index] += 1;
    }

    let mut candidate = String::with_capacity(text.len() + count * ARABIC_TATWEEL.len());
    let mut offsets = Vec::with_capacity(count);
    let mut source_cursor = 0;
    for (offset, tatweel_count) in insertion_offsets.iter().copied().zip(per_offset_counts) {
        candidate.push_str(&text[source_cursor..offset]);
        for _ in 0..tatweel_count {
            offsets.push(candidate.len());
            candidate.push_str(ARABIC_TATWEEL);
        }
        source_cursor = offset;
    }
    candidate.push_str(&text[source_cursor..]);
    (candidate, offsets)
}

fn line_grapheme_advances_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Vec<f32>> {
    let neutral_style = text_style(style);
    let advances = match measured_grapheme_widths_with_provider(text, &neutral_style, provider) {
        TextShapingOutcome::Ready(advances) => advances,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    if !text.contains('\t') {
        return TextShapingOutcome::Ready(advances);
    }

    measure_line_width_with_provider(" ", &neutral_style, provider)
        .map(|space_width| tab_aligned_advances(text, &advances, &neutral_style, space_width))
}

fn should_justify_line(
    line: &CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
) -> bool {
    matches!(style.text_align, UiTextAlign::Justify)
        && !is_last_line
        && frame_width > 0.0
        && !line.text.trim().is_empty()
        && !line.ellipsized
}

pub(super) fn available_wrap_extent(extent: f32) -> f32 {
    if extent.is_nan() {
        0.0
    } else {
        extent.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_wrap_extent_preserves_narrow_finite_constraints() {
        assert_eq!(available_wrap_extent(0.25), 0.25);
        assert_eq!(available_wrap_extent(-0.25), 0.0);
        assert_eq!(available_wrap_extent(f32::NAN), 0.0);
        assert_eq!(available_wrap_extent(f32::INFINITY), f32::INFINITY);
    }

    #[test]
    fn arabic_tatweel_budget_snapshot_matches_the_fit_algorithm_bounds() {
        let budget = arabic_tatweel_budget_snapshot();

        assert_eq!(
            budget.max_materialized_tatweels_per_line,
            MAX_ARABIC_TATWEELS_PER_LINE
        );
        assert_eq!(
            budget.max_fit_measurements_per_line,
            MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS
        );
    }

    #[test]
    fn arabic_tatweel_fit_uses_a_proportional_bounded_probe() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            TextShapingOutcome::Ready(Some(candidate_count as f32 * 4.0))
        })
        .into_result()
        .expect("candidate probes remain available");

        assert_eq!(count, Some(25));
        assert_eq!(attempted_counts, vec![32, 25]);
    }

    #[test]
    fn arabic_tatweel_fit_limits_unsuccessful_shape_probes() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            TextShapingOutcome::Ready(Some(1_000.0))
        })
        .into_result()
        .expect("candidate probes remain available");

        assert_eq!(count, None);
        assert!(attempted_counts.len() <= MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS);
        assert_eq!(attempted_counts.last(), Some(&1));
    }

    #[test]
    fn arabic_tatweel_fit_reserves_its_last_probe_for_one_real_tatweel() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            TextShapingOutcome::Ready(Some(100.0 + candidate_count as f32 * 0.001))
        })
        .into_result()
        .expect("candidate probes remain available");

        assert_eq!(count, Some(1));
        assert_eq!(attempted_counts, vec![32, 31, 30, 29, 1]);
    }

    #[test]
    fn arabic_tatweel_fit_retries_one_candidate_after_backend_safety_rejection() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            TextShapingOutcome::Ready((candidate_count == 1).then_some(4.0))
        })
        .into_result()
        .expect("candidate probes remain available");

        assert_eq!(count, Some(1));
        assert_eq!(attempted_counts, vec![32, 1]);
    }
}

pub(super) fn aligned_x(
    frame: UiFrame,
    line_width: f32,
    align: UiTextAlign,
    direction: UiTextDirection,
) -> f32 {
    match align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - line_width) * 0.5,
        UiTextAlign::Right => frame.right() - line_width,
        UiTextAlign::Start if is_rtl_direction(direction) => frame.right() - line_width,
        UiTextAlign::Start => frame.x,
        UiTextAlign::End if is_rtl_direction(direction) => frame.x,
        UiTextAlign::End => frame.right() - line_width,
        UiTextAlign::Justify => frame.x,
    }
}
