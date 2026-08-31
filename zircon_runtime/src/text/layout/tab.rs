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
    let mut cursor = 0.0_f32;
    let mut adjusted = Vec::with_capacity(advances.len());
    for (grapheme, advance) in text.graphemes(true).zip(advances.iter().copied()) {
        let resolved_advance = if grapheme == "\t" {
            next_tab_advance(cursor, tab_interval)
        } else {
            advance.max(0.0)
        };
        cursor += resolved_advance;
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
    if !has_matching_tab_graphemes(text, advances.len()) {
        return advances.iter().sum();
    }

    let tab_interval = tab_interval_width(style, space_width);
    let mut cursor = 0.0_f32;
    for (grapheme, advance) in text.graphemes(true).zip(advances.iter().copied()) {
        let resolved_advance = if grapheme == "\t" {
            next_tab_advance(cursor, tab_interval)
        } else {
            advance.max(0.0)
        };
        cursor += resolved_advance;
    }
    cursor
}

pub(crate) fn tab_interval_width(style: &TextStyle, space_width: f32) -> f32 {
    space_width.max(MIN_TAB_ADVANCE) * resolved_tab_size(style)
}

fn resolved_tab_size(style: &TextStyle) -> f32 {
    if style.tab_size.is_finite() {
        style.tab_size.max(MIN_TAB_SIZE)
    } else {
        TextStyle::DEFAULT_TAB_SIZE
    }
}

fn next_tab_advance(cursor: f32, tab_interval: f32) -> f32 {
    let tab_interval = tab_interval.max(MIN_TAB_ADVANCE);
    let next_stop = ((cursor / tab_interval).floor() + 1.0) * tab_interval;
    (next_stop - cursor).max(MIN_TAB_ADVANCE)
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
mod tests {
    use super::*;

    fn tab_style() -> TextStyle {
        TextStyle {
            tab_size: 4.0,
            ..TextStyle::default()
        }
    }

    #[test]
    fn streaming_tab_layout_preserves_tab_stops_and_unicode_graphemes() {
        let style = tab_style();
        let text = "a\u{301}\tb";
        let advances = [1.0, -2.0, 4.0];

        assert_eq!(
            tab_aligned_advances(text, &advances, &style, 2.0),
            vec![1.0, 7.0, 4.0]
        );
        assert_eq!(tab_aligned_width(text, &advances, &style, 2.0), 12.0);
    }

    #[test]
    fn streaming_tab_layout_preserves_unaligned_advance_fallbacks() {
        let style = tab_style();
        let no_tab_advances = [2.0, -1.0];
        let mismatched_advances = [3.0];

        assert_eq!(
            tab_aligned_advances("ab", &no_tab_advances, &style, 2.0),
            no_tab_advances
        );
        assert_eq!(tab_aligned_width("ab", &no_tab_advances, &style, 2.0), 1.0);
        assert_eq!(
            tab_aligned_advances("a\t", &mismatched_advances, &style, 2.0),
            mismatched_advances
        );
        assert_eq!(
            tab_aligned_width("a\t", &mismatched_advances, &style, 2.0),
            3.0
        );
    }
}
