use super::super::measure::measure_line_width_with_provider;
use crate::text::layout_geometry::finite_sum;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider};
use crate::text::TextStyle;

const LINE_FIT_EPSILON: f32 = 0.01;

pub(crate) fn should_wrap_before_accumulated(
    current_is_empty: bool,
    current_advance: f32,
    next_advance: f32,
    max_width: f32,
) -> bool {
    if current_is_empty {
        return false;
    }
    let current_advance = finite_non_negative(current_advance);
    let next_advance = finite_non_negative(next_advance);
    let max_width = if max_width.is_nan() {
        0.0
    } else {
        max_width.max(0.0)
    };
    finite_sum([current_advance, next_advance]) > fit_limit(max_width)
}

pub(crate) fn line_text_fits_with_provider<P>(
    text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<bool>
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_line_width_with_provider(text, style, provider)
        .map(|width| width <= fit_limit(max_width))
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn fit_limit(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else if value.is_infinite() {
        value
    } else {
        finite_sum([value.max(0.0), LINE_FIT_EPSILON])
    }
}

#[cfg(test)]
#[path = "tests/greedy.rs"]
mod tests;
