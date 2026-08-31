use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::rich::parser_registry::parse_rich_text as try_parse_rich_text;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::{
    InlineBaseline, LayoutItem, OpenTypeFeature, RichParseResult, RichTextFormat, StyleOverride,
};
use crate::text::{ShapedGlyphRun, TextRange, TextStyle};

use super::*;

fn parse_rich_text(markup: &str, format: RichTextFormat) -> RichParseResult {
    try_parse_rich_text(markup, format).expect("test rich source fits parser budgets")
}

#[test]
fn text_rich_inline_image_reserves_metric_in_layout() {
    let parsed = parse_rich_text(
        "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\" baseline=\"baseline\">after",
        RichTextFormat::HtmlSubsetV1,
    );
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let mut provider = DirectTextShapeRunProvider::default();

    let layout = layout_rich_line_with_provider(&parsed, &style, &mut provider)
        .into_result()
        .expect("layout rich line");

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.size.x > 16.0);
    let line = &layout.lines[0];
    assert!(line.ascent >= 24.0);
    assert!(line.descent >= 0.0);
    assert_eq!(line.baseline, line.ascent);
    let inline = layout
        .items
        .iter()
        .find_map(|item| match item {
            LayoutItem::Inline {
                source_range,
                size,
                baseline,
                origin,
                ..
            } => Some((*source_range, *size, *baseline, *origin)),
            LayoutItem::Text { .. } => None,
        })
        .expect("inline item");
    assert_eq!(inline.0, (6, 9));
    assert_eq!(inline.1.to_array(), [16.0, 24.0]);
    assert_eq!(inline.2, InlineBaseline::Baseline);
    assert_eq!(inline.3.y, 0.0);
}

#[test]
fn text_rich_inline_baseline_modes_project_expected_line_metrics() {
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let mut metrics_provider = DirectTextShapeRunProvider::default();
    let text_metrics = line_metrics_with_provider(&style, &mut metrics_provider)
        .into_result()
        .expect("measure text metrics");
    let text_ascent = text_metrics.baseline;
    let text_descent = text_metrics.line_height - text_ascent;
    let expected = [
        (InlineBaseline::Baseline, 20.0, text_descent),
        (
            InlineBaseline::Center,
            text_ascent.max(10.0),
            text_descent.max(10.0),
        ),
        (
            InlineBaseline::Top,
            text_ascent,
            text_descent.max(20.0 - text_ascent),
        ),
        (
            InlineBaseline::Bottom,
            text_ascent.max(20.0 - text_descent),
            text_descent,
        ),
    ];

    for (baseline, expected_ascent, expected_descent) in expected {
        let parsed = parse_rich_text(
            &format!(
                "<img src=\"res://icons/star.png\" width=\"16\" height=\"20\" baseline=\"{}\">",
                baseline_name(baseline)
            ),
            RichTextFormat::HtmlSubsetV1,
        );
        let mut provider = DirectTextShapeRunProvider::default();
        let layout = layout_rich_line_with_provider(&parsed, &style, &mut provider)
            .into_result()
            .expect("layout rich line");
        let line = &layout.lines[0];
        assert!((line.ascent - expected_ascent).abs() < 0.01);
        assert!((line.descent - expected_descent).abs() < 0.01);
        let origin_y = layout.items.iter().find_map(|item| match item {
            LayoutItem::Inline { origin, .. } => Some(origin.y),
            LayoutItem::Text { .. } => None,
        });
        assert!(origin_y.is_some_and(|origin_y| origin_y >= 0.0));
    }
}

#[test]
fn text_rich_run_style_overrides_participate_in_layout_metrics() {
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let plain = parse_rich_text("Wide", RichTextFormat::Plain);
    let large = parse_rich_text("[size=20]Wide[/size]", RichTextFormat::BbCodeV1);
    let mut plain_provider = DirectTextShapeRunProvider::default();
    let mut large_provider = DirectTextShapeRunProvider::default();

    let plain = layout_rich_line_with_provider(&plain, &style, &mut plain_provider)
        .into_result()
        .expect("layout plain rich line");
    let large = layout_rich_line_with_provider(&large, &style, &mut large_provider)
        .into_result()
        .expect("layout large rich line");

    assert!(large.size.x > plain.size.x);
    assert!(large.lines[0].ascent > plain.lines[0].ascent);
    assert!(large.size.y > plain.size.y);
}

#[test]
fn rich_run_style_projects_italic_and_open_type_features_to_shaping_style() {
    let base = TextStyle::default();
    let features = vec![
        OpenTypeFeature::new(*b"liga", 0),
        OpenTypeFeature::new(*b"ss01", 1),
    ];
    let resolved = resolve_rich_run_style(
        &base,
        &StyleOverride {
            italic: Some(true),
            features: Some(features.clone()),
            ..StyleOverride::default()
        },
    );

    assert!(resolved.italic);
    assert_eq!(resolved.features.as_ref(), features.as_slice());
    assert!(
        !resolve_rich_run_style(
            &TextStyle {
                italic: true,
                ..TextStyle::default()
            },
            &StyleOverride {
                italic: Some(false),
                ..StyleOverride::default()
            },
        )
        .italic
    );
}

#[test]
fn text_rich_forced_lines_preserve_inline_metrics_and_original_run_indices() {
    let parsed = parse_rich_text(
        "first\n<img src=\"res://icons/star.png\" width=\"16\" height=\"24\">second",
        RichTextFormat::HtmlSubsetV1,
    );
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let mut provider = DirectTextShapeRunProvider::default();

    let layout = layout_rich_text_with_provider(&parsed, &style, &mut provider)
        .into_result()
        .expect("layout rich text");

    assert_eq!(layout.lines.len(), 2);
    assert!(layout.lines[1].origin.y >= layout.lines[0].ascent + layout.lines[0].descent);
    assert!(layout.lines[1].ascent >= 24.0);
    assert_eq!(layout.lines[1].item_range, (1, 3));
    let inline = &layout.items[layout.lines[1].item_range.0 as usize];
    assert!(matches!(
        inline,
        LayoutItem::Inline {
            run_index: 1,
            source_range: (6, 9),
            advance,
            ..
        } if (*advance - 16.0).abs() < 0.01
    ));
}

#[test]
fn text_rich_forced_line_ranges_share_unicode_separator_semantics() {
    assert_eq!(
        super::rich_forced_line_ranges("a\r\nb\u{2028}c\u{0085}")
            .into_result()
            .expect("valid source ranges"),
        vec![(0, 1), (3, 4), (7, 8), (10, 10)]
    );
}

#[test]
fn text_rich_glyph_wrap_keeps_boundary_shaping_context_bounded() {
    let source = format!("A{}Z", "x".repeat(38));
    let parsed = parse_rich_text(&source, RichTextFormat::Plain);
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let mut provider = CountingShapeRunProvider::default();

    let layout = layout_rich_text_glyph_wrapped_with_provider(&parsed, &style, 12.0, &mut provider)
        .into_result()
        .expect("layout glyph-wrapped rich text");

    assert!(
        layout.lines.len() > 1,
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
            <= 2 + layout.lines.len()
                * (2 * super::super::line_break::BOUNDARY_SHAPING_CONTEXT_GRAPHEMES + 1),
        "boundary correction calls must remain linear in the produced line count"
    );
}

#[test]
fn text_rich_line_materialization_borrows_source_and_uses_a_run_cursor() {
    let source = include_str!("materialize.rs");

    assert!(!source.contains("parsed.text.clone()"));
    assert!(!source.contains("measure_text_source_range_width_with_provider"));
    assert!(
        !source.contains(
            ".runs\n            .iter()\n            .enumerate()\n            .filter_map"
        )
    );
    assert!(source.contains("run_cursor"));
}

fn baseline_name(baseline: InlineBaseline) -> &'static str {
    match baseline {
        InlineBaseline::Baseline => "baseline",
        InlineBaseline::Center => "center",
        InlineBaseline::Top => "top",
        InlineBaseline::Bottom => "bottom",
    }
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
