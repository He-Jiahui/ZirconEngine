use super::super::measure::measure_line_width;
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

const LINE_FIT_EPSILON: f32 = 0.01;

pub(crate) fn should_wrap_before_chunk(
    current_text: &str,
    next_text: &str,
    max_width: f32,
    style: &UiResolvedStyle,
) -> bool {
    !current_text.is_empty() && !appended_text_fits(current_text, next_text, max_width, style)
}

pub(crate) fn line_text_fits(text: &str, max_width: f32, style: &UiResolvedStyle) -> bool {
    measure_line_width(text, style) <= max_width + LINE_FIT_EPSILON
}

fn appended_text_fits(
    current_text: &str,
    next_text: &str,
    max_width: f32,
    style: &UiResolvedStyle,
) -> bool {
    let mut candidate = String::with_capacity(current_text.len() + next_text.len());
    candidate.push_str(current_text);
    candidate.push_str(next_text);
    line_text_fits(&candidate, max_width, style)
}

#[cfg(test)]
mod tests {
    use super::{line_text_fits, should_wrap_before_chunk};
    use crate::graphics::text::layout::measure::measure_line_width;
    use zircon_runtime_interface::ui::surface::UiResolvedStyle;

    #[test]
    fn should_wrap_before_chunk_keeps_empty_current_line() {
        let style = UiResolvedStyle::default();
        let max_width = measure_line_width("a", &style);

        assert!(!should_wrap_before_chunk("", "abcd", max_width, &style));
    }

    #[test]
    fn should_wrap_before_chunk_breaks_when_append_overflows() {
        let style = UiResolvedStyle::default();
        let max_width = measure_line_width("a", &style);

        assert!(line_text_fits("a", max_width, &style));
        assert!(should_wrap_before_chunk("a", "b", max_width, &style));
    }

    #[test]
    fn should_wrap_before_chunk_keeps_append_that_fits() {
        let style = UiResolvedStyle::default();
        let max_width = measure_line_width("ab", &style);

        assert!(!should_wrap_before_chunk("a", "b", max_width, &style));
    }
}
