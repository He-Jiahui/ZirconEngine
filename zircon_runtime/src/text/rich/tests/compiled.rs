use std::sync::Arc;

use super::{CompiledRichText, RichTextParserGeneration};
use crate::core::{math::Vec2, resource::ResourceId};
use crate::text::{
    InlineBaseline, InlineObjectRef, ParagraphOverride, RichIconAssetId, RichParseResult,
    RichTable, RichTableCell, RichTableColumn, RichTextFormat, RichTextParser, StyledRun,
};

#[test]
fn compiled_rich_text_estimate_counts_inline_icon_semantic_storage() {
    let alternative = "A".repeat(4 * 1024);
    let compiled_with_empty_alternative = compiled_icon_with_alternative("");
    let compiled_with_large_alternative = compiled_icon_with_alternative(&alternative);

    assert!(
        compiled_with_large_alternative.estimated_bytes()
            >= compiled_with_empty_alternative
                .estimated_bytes()
                .saturating_add(alternative.len())
    );
}

#[test]
fn compiled_rich_text_estimate_counts_image_semantic_storage() {
    let parser = RichTextParser::default();
    let empty = parser
        .compile(
            "<img src=\"res://icons/star.png\" alt=\"\">",
            RichTextFormat::HtmlSubsetV1,
        )
        .expect("empty alternative compiles");
    let alternative = "A".repeat(4 * 1024);
    let source = format!("<img src=\"res://icons/star.png\" alt=\"{alternative}\">");
    let populated = parser
        .compile(&source, RichTextFormat::HtmlSubsetV1)
        .expect("bounded alternative compiles");

    assert!(
        populated.estimated_bytes() >= empty.estimated_bytes().saturating_add(alternative.len())
    );
}

#[test]
fn compiled_rich_text_identity_excludes_residency_estimates() {
    let first = compiled_icon_with_alternative("Favorite");
    let mut same_semantics = compiled_icon_with_alternative("Favorite");
    same_semantics.estimated_bytes = same_semantics.estimated_bytes.saturating_add(1);

    assert_eq!(first, same_semantics);
}

fn compiled_icon_with_alternative(alternative_text: &str) -> CompiledRichText {
    compiled_from_parsed(RichParseResult {
        text: "\u{fffc}".into(),
        runs: vec![StyledRun {
            byte_range: (0, 3),
            inline: Some(InlineObjectRef::Icon {
                asset: RichIconAssetId::from_resource_id(ResourceId::from_stable_label(
                    "res://icons/favorite.png",
                )),
                size: Vec2::new(16.0, 16.0),
                baseline: InlineBaseline::Baseline,
                alternative_text: Some(alternative_text.to_owned()),
            }),
            ..StyledRun::default()
        }],
        ..RichParseResult::default()
    })
}

#[test]
fn compiled_rich_text_indexes_each_table_cell_projection() {
    let rich = compiled_from_parsed(RichParseResult {
        text: "outerinner".into(),
        runs: vec![
            StyledRun {
                byte_range: (0, 5),
                ..StyledRun::default()
            },
            StyledRun {
                byte_range: (5, 10),
                ..StyledRun::default()
            },
        ],
        paragraphs: vec![
            ((0, 5), ParagraphOverride::default()),
            ((5, 10), ParagraphOverride::default()),
        ],
        tables: vec![
            RichTable {
                byte_range: (0, 10),
                depth: 0,
                columns: vec![RichTableColumn::default()],
                cells: vec![RichTableCell {
                    byte_range: (0, 10),
                    ..RichTableCell::default()
                }],
            },
            RichTable {
                byte_range: (5, 10),
                depth: 1,
                columns: vec![RichTableColumn::default()],
                cells: vec![RichTableCell {
                    byte_range: (5, 10),
                    ..RichTableCell::default()
                }],
            },
        ],
        ..RichParseResult::default()
    });

    let outer = rich
        .cell_projection_indices(0, (0, 10))
        .expect("outer cell index");
    assert_eq!(outer.run_indices, &[0, 1]);
    assert_eq!(outer.paragraph_indices, &[0, 1]);
    assert_eq!(outer.nested_table_indices, &[1]);

    let nested = rich
        .cell_projection_indices(1, (5, 10))
        .expect("nested cell index");
    assert_eq!(nested.run_indices, &[1]);
    assert_eq!(nested.paragraph_indices, &[1]);
    assert!(nested.nested_table_indices.is_empty());
}

#[test]
fn rich_range_interval_index_rejects_touching_ranges_and_keeps_candidates_unique() {
    let index = super::RichRangeIntervalIndex::new(vec![
        super::RichRangeIntervalEntry {
            byte_range: (20, 30),
            source_index: 3,
        },
        super::RichRangeIntervalEntry {
            byte_range: (0, 5),
            source_index: 1,
        },
        super::RichRangeIntervalEntry {
            byte_range: (4, 8),
            source_index: 2,
        },
        super::RichRangeIntervalEntry {
            byte_range: (10, 12),
            source_index: 0,
        },
    ]);
    let mut candidates = Vec::new();
    index
        .collect_intersections((5, 11), 0, 8, &mut candidates)
        .expect("projection candidates fit the test budget");

    candidates.sort_unstable();
    assert_eq!(candidates, vec![0, 2]);
}

fn compiled_from_parsed(parsed: RichParseResult) -> CompiledRichText {
    let source_markup = Arc::clone(&parsed.text);
    CompiledRichText::new(
        source_markup,
        RichTextFormat::Plain,
        RichTextParserGeneration::default(),
        parsed,
    )
    .expect("test rich artifact fits indexed ranges")
}
