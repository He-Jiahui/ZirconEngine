use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap};

use super::{layout_text, measure_text_size};

mod alignment;
mod bidi;
mod frame_extent;
mod glue;
mod grapheme;
mod justify;
mod kinsoku;
mod measure;
mod overflow;
mod performance;
mod profiling;
mod rich_blocks;
mod rich_inline;
mod rich_table;
mod sizing;
mod soft_hyphen;
mod tab;
mod vertical;
mod viewport;
mod word_smart;
mod wrap_space;
mod wrapping;

fn test_style(wrap: UiTextWrap, overflow: UiTextOverflow) -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap,
        text_overflow: overflow,
        ..UiResolvedStyle::default()
    }
}

fn ellipsis_width_for_test(style: &UiResolvedStyle) -> f32 {
    let minimum = measure_text_size("a\u{0301}…", style).width + 0.1;
    let maximum = measure_text_size("a\u{0301}b…", style).width - 0.1;
    minimum
        .min(maximum)
        .max(measure_text_size("…", style).width)
}
