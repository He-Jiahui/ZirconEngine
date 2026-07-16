#[cfg(test)]
use super::super::measure::measure_line_width;
use super::super::measure::measure_line_width_with_provider;
#[cfg(test)]
use crate::text::shaping::DirectTextShapeRunProvider;
use crate::text::shaping::TextShapeRunProvider;
use crate::text::TextStyle;

const LINE_FIT_EPSILON: f32 = 0.01;

#[cfg(test)]
pub(crate) fn should_wrap_before_chunk(
    current_text: &str,
    next_text: &str,
    max_width: f32,
    style: &TextStyle,
) -> bool {
    let mut provider = DirectTextShapeRunProvider;
    should_wrap_before_chunk_with_provider(current_text, next_text, max_width, style, &mut provider)
}

pub(crate) fn should_wrap_before_chunk_with_provider<P>(
    current_text: &str,
    next_text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> bool
where
    P: TextShapeRunProvider + ?Sized,
{
    !current_text.is_empty()
        && !appended_text_fits_with_provider(current_text, next_text, max_width, style, provider)
}

#[cfg(test)]
pub(crate) fn line_text_fits(text: &str, max_width: f32, style: &TextStyle) -> bool {
    measure_line_width(text, style) <= max_width + LINE_FIT_EPSILON
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

#[cfg(test)]
fn appended_text_fits(
    current_text: &str,
    next_text: &str,
    max_width: f32,
    style: &TextStyle,
) -> bool {
    let mut candidate = String::with_capacity(current_text.len() + next_text.len());
    candidate.push_str(current_text);
    candidate.push_str(next_text);
    line_text_fits(&candidate, max_width, style)
}

fn appended_text_fits_with_provider<P>(
    current_text: &str,
    next_text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> bool
where
    P: TextShapeRunProvider + ?Sized,
{
    let mut candidate = String::with_capacity(current_text.len() + next_text.len());
    candidate.push_str(current_text);
    candidate.push_str(next_text);
    line_text_fits_with_provider(&candidate, max_width, style, provider)
}

#[cfg(test)]
mod tests {
    use super::{line_text_fits, should_wrap_before_chunk};
    use crate::text::layout::measure::measure_line_width;
    use crate::text::TextStyle;

    #[test]
    fn should_wrap_before_chunk_keeps_empty_current_line() {
        let style = TextStyle::default();
        let max_width = measure_line_width("a", &style);

        assert!(!should_wrap_before_chunk("", "abcd", max_width, &style));
    }

    #[test]
    fn should_wrap_before_chunk_breaks_when_append_overflows() {
        let style = TextStyle::default();
        let max_width = measure_line_width("a", &style);

        assert!(line_text_fits("a", max_width, &style));
        assert!(should_wrap_before_chunk("a", "b", max_width, &style));
    }

    #[test]
    fn should_wrap_before_chunk_keeps_append_that_fits() {
        let style = TextStyle::default();
        let max_width = measure_line_width("ab", &style);

        assert!(!should_wrap_before_chunk("a", "b", max_width, &style));
    }
}
