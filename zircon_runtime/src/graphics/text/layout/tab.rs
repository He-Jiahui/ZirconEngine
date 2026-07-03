use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

const MIN_TAB_SIZE: f32 = 1.0;
const MIN_TAB_ADVANCE: f32 = 0.01;

pub(crate) fn tab_aligned_advances(
    text: &str,
    advances: &[f32],
    style: &UiResolvedStyle,
    space_width: f32,
) -> Vec<f32> {
    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != advances.len() || !graphemes.iter().any(|grapheme| *grapheme == "\t") {
        return advances.to_vec();
    }

    let tab_interval = tab_interval_width(style, space_width);
    let mut cursor = 0.0_f32;
    let mut adjusted = Vec::with_capacity(advances.len());
    for (grapheme, advance) in graphemes.iter().zip(advances.iter().copied()) {
        let resolved_advance = if *grapheme == "\t" {
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
    style: &UiResolvedStyle,
    space_width: f32,
) -> f32 {
    tab_aligned_advances(text, advances, style, space_width)
        .iter()
        .sum()
}

fn tab_interval_width(style: &UiResolvedStyle, space_width: f32) -> f32 {
    space_width.max(MIN_TAB_ADVANCE) * resolved_tab_size(style)
}

fn resolved_tab_size(style: &UiResolvedStyle) -> f32 {
    if style.tab_size.is_finite() {
        style.tab_size.max(MIN_TAB_SIZE)
    } else {
        UiResolvedStyle::DEFAULT_TAB_SIZE
    }
}

fn next_tab_advance(cursor: f32, tab_interval: f32) -> f32 {
    let tab_interval = tab_interval.max(MIN_TAB_ADVANCE);
    let next_stop = ((cursor / tab_interval).floor() + 1.0) * tab_interval;
    (next_stop - cursor).max(MIN_TAB_ADVANCE)
}
