use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::rich::parser_registry::parse_rich_text as try_parse_rich_text;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::{
    RichParseResult, RichTextFormat, ShapedGlyphRun, TextRange, TextStyle, TextWrap,
};

use super::*;

fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    try_parse_rich_text(markup, format).expect("test rich source fits parser budgets")
}

#[test]
fn text_rich_vertical_glyph_wrap_keeps_boundary_shaping_context_bounded() {
    let source = format!("A{}Z", "x".repeat(38));
    let parsed = parse_rich_text(&source, RichTextFormat::Plain);
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: TextWrap::Glyph,
        ..TextStyle::default()
    };
    let mut provider = CountingShapeRunProvider::default();

    let columns = rich_vertical_columns_with_provider(&parsed, &style, |_, _| 12.0, &mut provider)
        .into_result()
        .expect("layout vertical rich columns");

    assert!(
        columns.len() > 1,
        "the fixture must exercise glyph wrapping"
    );
    let long_requests = provider
        .shaped_grapheme_counts
        .iter()
        .filter(|count| **count > 2 * super::super::line_break::BOUNDARY_SHAPING_CONTEXT_GRAPHEMES)
        .count();
    assert_eq!(
        long_requests, 1,
        "only the canonical continuous style span may exceed the boundary context"
    );
    assert!(
        provider.shaped_grapheme_counts.len()
            <= 1 + columns.len()
                * (2 * super::super::line_break::BOUNDARY_SHAPING_CONTEXT_GRAPHEMES + 1),
        "boundary correction calls must remain linear in the produced column count"
    );
}

#[derive(Default)]
struct CountingShapeRunProvider {
    direct: DirectTextShapeRunProvider,
    shaped_grapheme_counts: Vec<usize>,
}

impl TextShapeRunProvider for CountingShapeRunProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> crate::text::shaping::TextShapingOutcome {
        self.shaped_grapheme_counts
            .push(text.graphemes(true).count());
        self.direct.shape_horizontal_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }
}
