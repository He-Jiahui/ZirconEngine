use crate::text::layout_geometry::{finite_f32_or_geometry, finite_sum, FiniteGeometryAccumulator};
use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

const MIN_TAB_SIZE: f32 = 1.0;
const MIN_TAB_ADVANCE: f32 = 0.01;

pub(crate) fn tab_aligned_advances(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> Vec<f32> {
    if !has_matching_tab_graphemes(text, advances.len()) {
        return advances.to_vec();
    }

    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut cursor = FiniteGeometryAccumulator::default();
    let mut adjusted = Vec::with_capacity(advances.len());
    for (grapheme, advance) in text.graphemes(true).zip(advances.iter().copied()) {
        let resolved_advance = if grapheme == "\t" {
            next_tab_advance(
                cursor.value(),
                cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        cursor.add(resolved_advance);
        adjusted.push(resolved_advance);
    }
    adjusted
}

pub(crate) fn tab_aligned_width(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> f32 {
    let all_advances = advances;
    let mut advances = all_advances.iter().copied();
    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut raw_cursor = FiniteGeometryAccumulator::default();
    let mut tabbed_cursor = FiniteGeometryAccumulator::default();
    let mut has_tab = false;

    for grapheme in text.graphemes(true) {
        let Some(advance) = advances.next() else {
            return finite_sum(all_advances.iter().copied());
        };
        raw_cursor.add(advance);
        let resolved_advance = if grapheme == "\t" {
            has_tab = true;
            next_tab_advance(
                tabbed_cursor.value(),
                tabbed_cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        tabbed_cursor.add(resolved_advance);
    }

    if advances.next().is_some() {
        return finite_sum(all_advances.iter().copied());
    }
    if has_tab {
        tabbed_cursor.value()
    } else {
        raw_cursor.value()
    }
}

/// Streams a tabbed sequence whose grapheme count and tab presence were already validated.
pub(super) fn tab_aligned_width_for_matching_graphemes(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> f32 {
    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut cursor = FiniteGeometryAccumulator::default();
    for (grapheme, advance) in text.graphemes(true).zip(advances.iter().copied()) {
        let resolved_advance = if grapheme == "\t" {
            next_tab_advance(
                cursor.value(),
                cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        cursor.add(resolved_advance);
    }
    cursor.value()
}

pub(crate) fn tab_interval_width(style: &TextStyle, space_width: f32) -> f32 {
    finite_f32_or_geometry(
        space_width.max(MIN_TAB_ADVANCE) * resolved_tab_size(style),
        tab_interval_exact_width(style, space_width),
    )
}

fn tab_interval_exact_width(style: &TextStyle, space_width: f32) -> f64 {
    f64::from(space_width.max(MIN_TAB_ADVANCE)) * f64::from(resolved_tab_size(style))
}

fn resolved_tab_size(style: &TextStyle) -> f32 {
    if style.tab_size.is_finite() {
        style.tab_size.max(MIN_TAB_SIZE)
    } else {
        TextStyle::DEFAULT_TAB_SIZE
    }
}

fn next_tab_advance(
    cursor: f32,
    cursor_exact: f64,
    tab_interval: f32,
    tab_interval_exact: f64,
) -> f32 {
    let tab_interval = tab_interval.max(MIN_TAB_ADVANCE);
    let next_stop = ((cursor / tab_interval).floor() + 1.0) * tab_interval;
    let next_stop_exact = ((cursor_exact / tab_interval_exact).floor() + 1.0) * tab_interval_exact;
    finite_f32_or_geometry(
        (next_stop - cursor).max(MIN_TAB_ADVANCE),
        (next_stop_exact - cursor_exact).max(f64::from(MIN_TAB_ADVANCE)),
    )
}

fn has_matching_tab_graphemes(text: &str, advance_count: usize) -> bool {
    let (grapheme_count, has_tab) = text
        .graphemes(true)
        .fold((0_usize, false), |(count, has_tab), grapheme| {
            (count + 1, has_tab || grapheme == "\t")
        });
    grapheme_count == advance_count && has_tab
}

#[cfg(test)]
#[path = "tests/tab.rs"]
mod tests;
