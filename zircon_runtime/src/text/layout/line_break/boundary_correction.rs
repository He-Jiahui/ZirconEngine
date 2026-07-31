use crate::core::framework::text::TextDirection;
use crate::text::shaping::TextShapeRunProvider;
use crate::text::{TextRange, TextStyle};

use super::super::advance_index::{GraphemeAdvanceIndex, GraphemeAdvanceMetric};
use super::super::measure::measured_width;

/// Edge shaping sees enough neighboring graphemes for common contextual substitutions while
/// keeping every correction request independent of the complete paragraph length.
pub(crate) const BOUNDARY_SHAPING_CONTEXT_GRAPHEMES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoundaryAdvanceUnit<'a> {
    pub(crate) text: &'a str,
    pub(crate) advance: f32,
}

pub(crate) fn corrected_glyph_ranges_with_provider<P>(
    text: &str,
    index: &GraphemeAdvanceIndex,
    style: &TextStyle,
    direction: TextDirection,
    first_max_advance: f32,
    continuation_max_advance: f32,
    provider: &mut P,
) -> Vec<(usize, usize)>
where
    P: TextShapeRunProvider + ?Sized,
{
    let metrics = index.metrics_in_range(0, text.len());
    corrected_metric_ranges(
        metrics,
        first_max_advance,
        continuation_max_advance,
        |first, after_last| {
            corrected_index_advance_with_provider(
                text,
                index,
                metrics[first].source_start,
                metrics[after_last.saturating_sub(1)].source_end,
                style,
                direction,
                None,
                provider,
            )
        },
    )
    .into_iter()
    .map(|(first, after_last)| {
        (
            metrics[first].source_start,
            metrics[after_last.saturating_sub(1)].source_end,
        )
    })
    .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn corrected_index_advance_with_provider<P>(
    text: &str,
    index: &GraphemeAdvanceIndex,
    start: usize,
    end: usize,
    style: &TextStyle,
    direction: TextDirection,
    break_suffix: Option<&str>,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    let metrics = index.metrics_in_range(start, end);
    let raw_advance = index.advance(start, end);
    let context_span = BOUNDARY_SHAPING_CONTEXT_GRAPHEMES.saturating_mul(2);
    if metrics.len() <= context_span {
        let units = collect_metric_units(text, metrics);
        return corrected_bounded_advance_with_provider(
            raw_advance,
            metrics.len(),
            &units,
            &[],
            style,
            direction,
            break_suffix,
            provider,
        );
    }

    let leading = collect_metric_units(text, &metrics[..context_span]);
    let trailing = collect_metric_units(text, &metrics[metrics.len() - context_span..]);
    corrected_bounded_advance_with_provider(
        raw_advance,
        metrics.len(),
        &leading,
        &trailing,
        style,
        direction,
        break_suffix,
        provider,
    )
}

fn collect_metric_units<'a>(
    text: &'a str,
    metrics: &[GraphemeAdvanceMetric],
) -> Vec<BoundaryAdvanceUnit<'a>> {
    metrics
        .iter()
        .filter_map(|metric| {
            text.get(metric.source_start..metric.source_end)
                .map(|text| BoundaryAdvanceUnit {
                    text,
                    advance: metric.advance,
                })
        })
        .collect()
}

pub(crate) fn corrected_metric_ranges<F>(
    metrics: &[GraphemeAdvanceMetric],
    first_max_advance: f32,
    continuation_max_advance: f32,
    mut corrected_advance: F,
) -> Vec<(usize, usize)>
where
    F: FnMut(usize, usize) -> f32,
{
    let mut ranges = Vec::new();
    let mut first = 0_usize;
    while first < metrics.len() {
        let max_advance = normalized_limit(if ranges.is_empty() {
            first_max_advance
        } else {
            continuation_max_advance
        });
        let mut after_last = first;
        let mut raw_advance = 0.0_f32;
        while let Some(metric) = metrics.get(after_last) {
            let next = raw_advance + finite_non_negative(metric.advance);
            if after_last > first && next > max_advance {
                break;
            }
            raw_advance = next;
            after_last = after_last.saturating_add(1);
        }
        after_last = after_last.max(first.saturating_add(1)).min(metrics.len());
        after_last = corrected_metric_end(
            metrics.len(),
            first,
            after_last,
            max_advance,
            &mut corrected_advance,
        );
        ranges.push((first, after_last));
        first = after_last;
    }
    ranges
}

fn corrected_metric_end<F>(
    metric_count: usize,
    first: usize,
    tentative_after_last: usize,
    max_advance: f32,
    corrected_advance: &mut F,
) -> usize
where
    F: FnMut(usize, usize) -> f32,
{
    let mut after_last = tentative_after_last;
    let mut corrected = corrected_advance(first, after_last);
    let mut steps = 0_usize;
    while corrected > max_advance
        && after_last > first.saturating_add(1)
        && steps < BOUNDARY_SHAPING_CONTEXT_GRAPHEMES
    {
        after_last = after_last.saturating_sub(1);
        steps = steps.saturating_add(1);
        corrected = corrected_advance(first, after_last);
    }
    if corrected > max_advance {
        return first.saturating_add(1).min(metric_count);
    }

    steps = 0;
    while after_last < metric_count && steps < BOUNDARY_SHAPING_CONTEXT_GRAPHEMES {
        let candidate = after_last.saturating_add(1);
        if corrected_advance(first, candidate) > max_advance {
            break;
        }
        after_last = candidate;
        steps = steps.saturating_add(1);
    }
    after_last
}

pub(crate) fn corrected_line_advance_with_provider<P>(
    units: &[BoundaryAdvanceUnit<'_>],
    style: &TextStyle,
    direction: TextDirection,
    break_suffix: Option<&str>,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    let raw_advance = units
        .iter()
        .map(|unit| finite_non_negative(unit.advance))
        .sum();
    let context_span = BOUNDARY_SHAPING_CONTEXT_GRAPHEMES.saturating_mul(2);
    if units.len() <= context_span {
        return corrected_bounded_advance_with_provider(
            raw_advance,
            units.len(),
            units,
            &[],
            style,
            direction,
            break_suffix,
            provider,
        );
    }
    corrected_bounded_advance_with_provider(
        raw_advance,
        units.len(),
        &units[..context_span],
        &units[units.len() - context_span..],
        style,
        direction,
        break_suffix,
        provider,
    )
}

#[allow(clippy::too_many_arguments)]
fn corrected_bounded_advance_with_provider<P>(
    raw_advance: f32,
    unit_count: usize,
    leading: &[BoundaryAdvanceUnit<'_>],
    trailing: &[BoundaryAdvanceUnit<'_>],
    style: &TextStyle,
    direction: TextDirection,
    break_suffix: Option<&str>,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    if unit_count == 0 {
        return break_suffix.map_or(0.0, |suffix| {
            shape_window_width(suffix, 0, suffix.len(), style, direction, provider)
        });
    }

    let context_span = BOUNDARY_SHAPING_CONTEXT_GRAPHEMES.saturating_mul(2);
    if unit_count <= context_span {
        let (text, _) = collect_window(leading, break_suffix);
        return shape_window_width(&text, 0, text.len(), style, direction, provider);
    }

    let Some(leading) = leading.get(..context_span) else {
        return finite_non_negative(raw_advance);
    };
    let (leading_text, leading_offsets) = collect_window(leading, None);
    let leading_end = leading_offsets[BOUNDARY_SHAPING_CONTEXT_GRAPHEMES];
    let raw_leading = leading[..BOUNDARY_SHAPING_CONTEXT_GRAPHEMES]
        .iter()
        .map(|unit| finite_non_negative(unit.advance))
        .sum::<f32>();
    let shaped_leading =
        shape_window_width(&leading_text, 0, leading_end, style, direction, provider);

    let Some(trailing) = trailing.get(..context_span) else {
        return finite_non_negative(raw_advance);
    };
    let (trailing_text, trailing_offsets) = collect_window(trailing, break_suffix);
    let trailing_start = trailing_offsets[BOUNDARY_SHAPING_CONTEXT_GRAPHEMES];
    let raw_trailing = trailing[BOUNDARY_SHAPING_CONTEXT_GRAPHEMES..]
        .iter()
        .map(|unit| finite_non_negative(unit.advance))
        .sum::<f32>();
    let shaped_trailing = shape_window_width(
        &trailing_text,
        trailing_start,
        trailing_text.len(),
        style,
        direction,
        provider,
    );

    finite_non_negative(
        raw_advance + (shaped_leading - raw_leading) + (shaped_trailing - raw_trailing),
    )
}

fn collect_window(units: &[BoundaryAdvanceUnit<'_>], suffix: Option<&str>) -> (String, Vec<usize>) {
    let suffix_len = suffix.map_or(0, str::len);
    let capacity = units
        .iter()
        .map(|unit| unit.text.len())
        .sum::<usize>()
        .saturating_add(suffix_len);
    let mut text = String::with_capacity(capacity);
    let mut offsets = Vec::with_capacity(units.len().saturating_add(1));
    offsets.push(0);
    for unit in units {
        text.push_str(unit.text);
        offsets.push(text.len());
    }
    if let Some(suffix) = suffix {
        text.push_str(suffix);
    }
    (text, offsets)
}

fn shape_window_width<P>(
    text: &str,
    start: usize,
    end: usize,
    style: &TextStyle,
    direction: TextDirection,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    let shaped = provider.shape_horizontal_line_with_kerning(
        text,
        style,
        direction,
        TextRange {
            start: 0,
            end: text.len(),
        },
        true,
    );
    finite_non_negative(measured_width(&shaped, start, end, true))
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn normalized_limit(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.max(0.0)
    }
}

#[cfg(test)]
mod tests;
