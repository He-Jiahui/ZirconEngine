use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap};

use super::{layout_text, measure_text_size};

#[path = "alignment.rs"]
mod alignment;
#[path = "bidi.rs"]
mod bidi;
#[path = "frame_extent.rs"]
mod frame_extent;
#[path = "glue.rs"]
mod glue;
#[path = "grapheme.rs"]
mod grapheme;
#[path = "justify.rs"]
mod justify;
#[path = "kinsoku.rs"]
mod kinsoku;
#[path = "measure.rs"]
mod measure;
#[path = "overflow.rs"]
mod overflow;
#[path = "performance.rs"]
mod performance;
#[path = "physical_line_metrics.rs"]
mod physical_line_metrics;
#[path = "profiling.rs"]
mod profiling;
#[path = "rich_blocks.rs"]
mod rich_blocks;
#[path = "rich_layout.rs"]
mod rich_layout;
#[path = "rich_table/mod.rs"]
mod rich_table;
#[path = "sizing.rs"]
mod sizing;
#[path = "soft_hyphen.rs"]
mod soft_hyphen;
#[path = "tab.rs"]
mod tab;
#[path = "vertical.rs"]
mod vertical;
#[path = "viewport.rs"]
mod viewport;
#[path = "word_smart.rs"]
mod word_smart;
#[path = "wrap_space.rs"]
mod wrap_space;
#[path = "wrapping.rs"]
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
