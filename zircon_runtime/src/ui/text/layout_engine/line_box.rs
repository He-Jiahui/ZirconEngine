use crate::text::layout::{
    arabic_kashida_insertion_offsets_bounded, justify_line_advances,
    measure_line_width_with_provider, measured_grapheme_widths_with_provider, tab_aligned_advances,
};
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextAlign, UiTextDirection};

use super::candidate_line::{insert_virtual_text, CandidateLine};
use super::direction::is_rtl_direction;
use crate::text::text_style;

pub(super) const MIN_TEXT_FONT_SIZE: f32 = 1.0;
const ARABIC_TATWEEL: &str = "\u{0640}";
const MAX_ARABIC_TATWEELS_PER_LINE: usize = 32;
const MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS: usize = 5;
const TATWEEL_FIT_EPSILON: f32 = 0.01;

pub(super) fn resolve_line_widths_with_provider(
    line: &CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
    provider: &mut SharedTextLayoutSession,
) -> (f32, Vec<f32>, f32) {
    let natural_advances = line_grapheme_advances_with_provider(&line.text, style, provider);
    let natural_width = natural_advances.iter().sum();
    if should_justify_line(line, style, frame_width, is_last_line) {
        if let Some(justified_advances) =
            justify_line_advances(&line.text, &natural_advances, natural_width, frame_width)
        {
            let justified_width = justified_advances.iter().sum();
            return (justified_width, justified_advances, frame_width);
        }
    }

    let line_width = natural_width.min(frame_width);
    (natural_width, natural_advances, line_width)
}

pub(super) fn materialize_arabic_tatweels_for_justified_line(
    line: &mut CandidateLine,
    style: &UiResolvedStyle,
    frame_width: f32,
    is_last_line: bool,
    provider: &mut SharedTextLayoutSession,
) {
    if !should_justify_line(line, style, frame_width, is_last_line) {
        return;
    }
    let natural_advances = line_grapheme_advances_with_provider(&line.text, style, provider);
    let natural_width = natural_advances.iter().sum();
    materialize_arabic_tatweels(line, natural_width, frame_width, style, provider);
}

fn materialize_arabic_tatweels(
    line: &mut CandidateLine,
    natural_width: f32,
    target_width: f32,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> bool {
    let extra = target_width - natural_width;
    if !extra.is_finite() || extra <= TATWEEL_FIT_EPSILON {
        return false;
    }
    let insertion_offsets =
        arabic_kashida_insertion_offsets_bounded(&line.text, MAX_ARABIC_TATWEELS_PER_LINE);
    if insertion_offsets.is_empty() {
        return false;
    }

    let neutral_style = text_style(style);
    let tatweel_width = measure_line_width_with_provider(ARABIC_TATWEEL, &neutral_style, provider);
    if !tatweel_width.is_finite() || tatweel_width <= TATWEEL_FIT_EPSILON {
        return false;
    }
    let requested_count = ((extra / tatweel_width).floor() as usize)
        .max(1)
        .min(MAX_ARABIC_TATWEELS_PER_LINE);

    let Some(count) = bounded_arabic_tatweel_fit_count(
        requested_count,
        natural_width,
        target_width,
        |candidate_count| {
            let (candidate, _) =
                text_with_arabic_tatweels(&line.text, &insertion_offsets, candidate_count);
            measure_line_width_with_provider(&candidate, &neutral_style, provider)
        },
    ) else {
        return false;
    };
    let (_, offsets) = text_with_arabic_tatweels(&line.text, &insertion_offsets, count);
    let mut materialized = line.clone();
    if offsets
        .into_iter()
        .all(|offset| insert_virtual_text(&mut materialized, offset, ARABIC_TATWEEL))
    {
        *line = materialized;
        return true;
    }
    false
}

fn bounded_arabic_tatweel_fit_count(
    requested_count: usize,
    natural_width: f32,
    target_width: f32,
    mut candidate_width: impl FnMut(usize) -> f32,
) -> Option<usize> {
    let extra = target_width - natural_width;
    if requested_count == 0 || !extra.is_finite() || extra <= TATWEEL_FIT_EPSILON {
        return None;
    }

    let mut count = requested_count;
    for probe_index in 0..MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS {
        if probe_index + 1 == MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS {
            count = 1;
        }
        let width = candidate_width(count);
        if width.is_finite() && width <= target_width + TATWEEL_FIT_EPSILON {
            return Some(count);
        }
        if count == 1 {
            return None;
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
    None
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
) -> Vec<f32> {
    let neutral_style = text_style(style);
    let advances = measured_grapheme_widths_with_provider(text, &neutral_style, provider);
    if !text.contains('\t') {
        return advances;
    }

    tab_aligned_advances(
        text,
        &advances,
        &neutral_style,
        measure_line_width_with_provider(" ", &neutral_style, provider),
    )
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
    fn arabic_tatweel_fit_uses_a_proportional_bounded_probe() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            candidate_count as f32 * 4.0
        });

        assert_eq!(count, Some(25));
        assert_eq!(attempted_counts, vec![32, 25]);
    }

    #[test]
    fn arabic_tatweel_fit_limits_unsuccessful_shape_probes() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            1_000.0
        });

        assert_eq!(count, None);
        assert!(attempted_counts.len() <= MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS);
        assert_eq!(attempted_counts.last(), Some(&1));
    }

    #[test]
    fn arabic_tatweel_fit_reserves_its_last_probe_for_one_real_tatweel() {
        let mut attempted_counts = Vec::new();
        let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
            attempted_counts.push(candidate_count);
            100.0 + candidate_count as f32 * 0.001
        });

        assert_eq!(count, Some(1));
        assert_eq!(attempted_counts, vec![32, 31, 30, 29, 1]);
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
