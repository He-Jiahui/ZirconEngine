use crate::text::rich::parse_rich_text;
use crate::text::shaping::DirectTextShapeRunProvider;
use crate::text::TextStyle;
use crate::text::{InlineBaseline, LayoutItem, RichTextFormat};

use super::*;

#[test]
fn text_rich_inline_image_reserves_metric_in_layout() {
    let parsed = parse_rich_text(
            "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\" baseline=\"baseline\">after",
            RichTextFormat::Html,
        );
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let mut provider = DirectTextShapeRunProvider;

    let layout = layout_rich_line_with_provider(&parsed, &style, &mut provider);

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
    let mut metrics_provider = DirectTextShapeRunProvider;
    let text_metrics = line_metrics_with_provider(&style, &mut metrics_provider);
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
            RichTextFormat::Html,
        );
        let mut provider = DirectTextShapeRunProvider;
        let layout = layout_rich_line_with_provider(&parsed, &style, &mut provider);
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
    let large = parse_rich_text("[size=20]Wide[/size]", RichTextFormat::BbCode);
    let mut plain_provider = DirectTextShapeRunProvider;
    let mut large_provider = DirectTextShapeRunProvider;

    let plain = layout_rich_line_with_provider(&plain, &style, &mut plain_provider);
    let large = layout_rich_line_with_provider(&large, &style, &mut large_provider);

    assert!(large.size.x > plain.size.x);
    assert!(large.lines[0].ascent > plain.lines[0].ascent);
    assert!(large.size.y > plain.size.y);
}

#[test]
fn text_rich_forced_lines_preserve_inline_metrics_and_original_run_indices() {
    let parsed = parse_rich_text(
        "first\n<img src=\"res://icons/star.png\" width=\"16\" height=\"24\">second",
        RichTextFormat::Html,
    );
    let style = TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    };
    let mut provider = DirectTextShapeRunProvider;

    let layout = layout_rich_text_with_provider(&parsed, &style, &mut provider);

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

fn baseline_name(baseline: InlineBaseline) -> &'static str {
    match baseline {
        InlineBaseline::Baseline => "baseline",
        InlineBaseline::Center => "center",
        InlineBaseline::Top => "top",
        InlineBaseline::Bottom => "bottom",
    }
}
