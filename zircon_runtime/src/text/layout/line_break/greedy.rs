use super::super::measure::measure_line_width_with_provider;
use crate::text::shaping::TextShapeRunProvider;
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
    current_advance + next_advance > max_width + LINE_FIT_EPSILON
}

pub(crate) fn line_text_fits_with_provider<P>(
    text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> bool
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_line_width_with_provider(text, style, provider) <= max_width + LINE_FIT_EPSILON
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::should_wrap_before_accumulated;

    #[test]
    fn accumulated_wrap_uses_existing_and_next_advances_without_text_candidates() {
        assert!(!should_wrap_before_accumulated(true, 8.0, 8.0, 10.0));
        assert!(!should_wrap_before_accumulated(false, 4.0, 6.0, 10.0));
        assert!(should_wrap_before_accumulated(false, 4.0, 6.1, 10.0));
    }
}
