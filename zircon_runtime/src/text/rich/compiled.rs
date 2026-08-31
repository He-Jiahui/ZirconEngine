use std::sync::Arc;

use crate::text::{RichParseResult, RichTextFormat, StyledRun};

use super::admission::{
    DEFAULT_RICH_TEXT_PROJECTION_INDICES, DEFAULT_RICH_TEXT_SEMANTIC_TEXT_BYTES,
    RichTextContentTrust, RichTextParseError, checked_artifact_index,
};

#[path = "compiled/dependency.rs"]
mod dependency;

#[path = "compiled/memory.rs"]
mod memory;

#[path = "compiled/semantic_text.rs"]
mod semantic_text;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RichTextParserGeneration {
    pub(crate) parser_identity: u64,
    pub(crate) decorator_generation: u64,
    pub(crate) emoji_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RichTableCellProjectionIndex {
    parent_table_depth: u16,
    byte_range: (u32, u32),
    run_indices: (u32, u32),
    paragraph_indices: (u32, u32),
    nested_table_indices: (u32, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RichRangeIntervalEntry {
    byte_range: (u32, u32),
    source_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RichRangeIntervalNode {
    entry: RichRangeIntervalEntry,
    max_end: u32,
    left: Option<usize>,
    right: Option<usize>,
}

/// Request-local interval owner used to project only ranges that can intersect one table cell.
///
/// The index is deliberately not retained by `CompiledRichText`; it is a construction-time search
/// structure over already checked canonical ranges.
struct RichRangeIntervalIndex {
    nodes: Vec<RichRangeIntervalNode>,
    root: Option<usize>,
}

impl RichRangeIntervalIndex {
    fn new(mut entries: Vec<RichRangeIntervalEntry>) -> Self {
        if entries
            .windows(2)
            .any(|pair| interval_entry_key(pair[0]) > interval_entry_key(pair[1]))
        {
            entries.sort_unstable_by_key(|entry| interval_entry_key(*entry));
        }
        let mut nodes = Vec::with_capacity(entries.len());
        let root = build_interval_tree(&entries, &mut nodes, 0..entries.len());
        Self { nodes, root }
    }

    fn collect_intersections(
        &self,
        byte_range: (u32, u32),
        consumed_results: usize,
        max_results: usize,
        output: &mut Vec<u32>,
    ) -> Result<(), RichTextParseError> {
        if let Some(root) = self.root {
            self.collect_intersections_from(
                root,
                byte_range,
                consumed_results,
                max_results,
                output,
            )?;
        }
        Ok(())
    }

    fn collect_intersections_from(
        &self,
        node_index: usize,
        byte_range: (u32, u32),
        consumed_results: usize,
        max_results: usize,
        output: &mut Vec<u32>,
    ) -> Result<(), RichTextParseError> {
        let node = self.nodes[node_index];
        if node.max_end <= byte_range.0 {
            return Ok(());
        }
        if let Some(left) = node.left {
            self.collect_intersections_from(
                left,
                byte_range,
                consumed_results,
                max_results,
                output,
            )?;
        }
        if ranges_intersect(node.entry.byte_range, byte_range) {
            let attempted_indices = consumed_results
                .saturating_add(output.len())
                .saturating_add(1);
            if attempted_indices > max_results {
                return Err(RichTextParseError::ProjectionIndexBudgetExceeded {
                    attempted_indices,
                    max_indices: max_results,
                });
            }
            output.push(node.entry.source_index);
        }
        if node.entry.byte_range.0 < byte_range.1 {
            if let Some(right) = node.right {
                self.collect_intersections_from(
                    right,
                    byte_range,
                    consumed_results,
                    max_results,
                    output,
                )?;
            }
        }
        Ok(())
    }
}

const fn interval_entry_key(entry: RichRangeIntervalEntry) -> (u32, u32, u32) {
    (entry.byte_range.0, entry.byte_range.1, entry.source_index)
}

fn build_interval_tree(
    entries: &[RichRangeIntervalEntry],
    nodes: &mut Vec<RichRangeIntervalNode>,
    range: std::ops::Range<usize>,
) -> Option<usize> {
    if range.is_empty() {
        return None;
    }
    let middle = range.start + (range.end - range.start) / 2;
    let entry = entries[middle];
    let node_index = nodes.len();
    nodes.push(RichRangeIntervalNode {
        entry,
        max_end: entry.byte_range.1,
        left: None,
        right: None,
    });
    let left = build_interval_tree(entries, nodes, range.start..middle);
    let right = build_interval_tree(entries, nodes, middle + 1..range.end);
    let max_end = left
        .map(|index| nodes[index].max_end)
        .into_iter()
        .chain(right.map(|index| nodes[index].max_end))
        .fold(entry.byte_range.1, u32::max);
    nodes[node_index] = RichRangeIntervalNode {
        entry,
        max_end,
        left,
        right,
    };
    Some(node_index)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RichTableCellProjectionIndices<'a> {
    pub(crate) run_indices: &'a [u32],
    pub(crate) paragraph_indices: &'a [u32],
    pub(crate) nested_table_indices: &'a [u32],
}

pub use dependency::RichTextDependency;

/// Canonical, generation-owned result shared by every rich-text consumer.
#[derive(Debug)]
pub struct CompiledRichText {
    source_markup: Arc<str>,
    format: RichTextFormat,
    content_trust: RichTextContentTrust,
    generation: RichTextParserGeneration,
    parsed: RichParseResult,
    semantic_text: Arc<str>,
    inline_run_indices: Arc<[u32]>,
    link_run_indices: Arc<[u32]>,
    dependencies: Arc<[RichTextDependency]>,
    table_cell_projection_indices: Arc<[RichTableCellProjectionIndex]>,
    cell_run_indices: Arc<[u32]>,
    cell_paragraph_indices: Arc<[u32]>,
    cell_nested_table_indices: Arc<[u32]>,
    estimated_bytes: usize,
}

impl PartialEq for CompiledRichText {
    fn eq(&self, other: &Self) -> bool {
        self.source_markup == other.source_markup
            && self.format == other.format
            && self.content_trust == other.content_trust
            && self.generation == other.generation
            && self.parsed == other.parsed
            && self.semantic_text == other.semantic_text
            && self.inline_run_indices == other.inline_run_indices
            && self.link_run_indices == other.link_run_indices
            && self.dependencies == other.dependencies
            && self.table_cell_projection_indices == other.table_cell_projection_indices
            && self.cell_run_indices == other.cell_run_indices
            && self.cell_paragraph_indices == other.cell_paragraph_indices
            && self.cell_nested_table_indices == other.cell_nested_table_indices
    }
}

impl CompiledRichText {
    pub(crate) fn new(
        source_markup: Arc<str>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
        parsed: RichParseResult,
    ) -> Result<Self, RichTextParseError> {
        Self::new_with_projection_budget(
            source_markup,
            format,
            generation,
            parsed,
            DEFAULT_RICH_TEXT_PROJECTION_INDICES,
            DEFAULT_RICH_TEXT_SEMANTIC_TEXT_BYTES,
        )
    }

    pub(crate) fn new_with_projection_budget(
        source_markup: Arc<str>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
        parsed: RichParseResult,
        max_projection_indices: usize,
        max_semantic_text_bytes: usize,
    ) -> Result<Self, RichTextParseError> {
        Self::new_with_content_trust_and_projection_budget(
            source_markup,
            format,
            RichTextContentTrust::Untrusted,
            generation,
            parsed,
            max_projection_indices,
            max_semantic_text_bytes,
        )
    }

    pub(crate) fn new_with_content_trust_and_projection_budget(
        source_markup: Arc<str>,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
        parsed: RichParseResult,
        max_projection_indices: usize,
        max_semantic_text_bytes: usize,
    ) -> Result<Self, RichTextParseError> {
        checked_artifact_index("visible byte length", parsed.text.len())?;
        checked_artifact_index("run count", parsed.runs.len())?;
        checked_artifact_index("paragraph count", parsed.paragraphs.len())?;
        checked_artifact_index("table count", parsed.tables.len())?;
        let inline_run_indices = parsed
            .runs
            .iter()
            .enumerate()
            .filter(|(_, run)| run.inline.is_some())
            .map(|(index, _)| checked_artifact_index("inline run", index))
            .collect::<Result<Vec<_>, _>>()?;
        let link_run_indices = parsed
            .runs
            .iter()
            .enumerate()
            .filter(|(_, run)| run.link.is_some())
            .map(|(index, _)| checked_artifact_index("link run", index))
            .collect::<Result<Vec<_>, _>>()?;
        let semantic_text = semantic_text::semantic_text_for_inline_runs(
            &parsed,
            &inline_run_indices,
            max_semantic_text_bytes,
        )?;
        let dependencies = dependency::collect(&parsed);
        let (
            table_cell_projection_indices,
            cell_run_indices,
            cell_paragraph_indices,
            cell_nested_table_indices,
        ) = table_cell_projection_indices(&parsed, max_projection_indices)?;
        let mut compiled = Self {
            source_markup,
            format,
            content_trust,
            generation,
            parsed,
            semantic_text,
            inline_run_indices: Arc::from(inline_run_indices.into_boxed_slice()),
            link_run_indices: Arc::from(link_run_indices.into_boxed_slice()),
            dependencies: Arc::from(dependencies.into_boxed_slice()),
            table_cell_projection_indices: Arc::from(
                table_cell_projection_indices.into_boxed_slice(),
            ),
            cell_run_indices: Arc::from(cell_run_indices.into_boxed_slice()),
            cell_paragraph_indices: Arc::from(cell_paragraph_indices.into_boxed_slice()),
            cell_nested_table_indices: Arc::from(cell_nested_table_indices.into_boxed_slice()),
            estimated_bytes: 0,
        };
        compiled.estimated_bytes = compiled.calculate_estimated_bytes();
        Ok(compiled)
    }

    pub fn source_markup(&self) -> &str {
        &self.source_markup
    }

    pub const fn format(&self) -> RichTextFormat {
        self.format
    }

    pub const fn content_trust(&self) -> RichTextContentTrust {
        self.content_trust
    }

    pub fn parsed(&self) -> &RichParseResult {
        &self.parsed
    }

    pub fn text(&self) -> &str {
        &self.parsed.text
    }

    pub(crate) fn semantic_text(&self) -> &str {
        &self.semantic_text
    }

    pub fn shared_text(&self) -> Arc<str> {
        Arc::clone(&self.parsed.text)
    }

    pub fn run_for_range(&self, start: usize, end: usize) -> Option<&StyledRun> {
        if start >= end {
            return None;
        }
        let start = u32::try_from(start).ok()?;
        let end = u32::try_from(end).ok()?;
        let index = self
            .parsed
            .runs
            .partition_point(|run| run.byte_range.1 <= start);
        self.parsed
            .runs
            .get(index)
            .filter(|run| run.byte_range.0 <= start && end <= run.byte_range.1)
    }

    pub fn inline_runs(&self) -> impl Iterator<Item = &StyledRun> {
        self.inline_run_indices
            .iter()
            .filter_map(|index| self.parsed.runs.get(*index as usize))
    }

    pub fn link_runs(&self) -> impl Iterator<Item = &StyledRun> {
        self.link_run_indices
            .iter()
            .filter_map(|index| self.parsed.runs.get(*index as usize))
    }

    pub fn dependencies(&self) -> &[RichTextDependency] {
        &self.dependencies
    }

    pub(crate) fn cell_projection_indices(
        &self,
        parent_table_depth: u16,
        byte_range: (u32, u32),
    ) -> Option<RichTableCellProjectionIndices<'_>> {
        let key = (parent_table_depth, byte_range.0, byte_range.1);
        let index = self
            .table_cell_projection_indices
            .binary_search_by_key(&key, |index| {
                (
                    index.parent_table_depth,
                    index.byte_range.0,
                    index.byte_range.1,
                )
            })
            .ok()?;
        let index = self.table_cell_projection_indices.get(index)?;
        Some(RichTableCellProjectionIndices {
            run_indices: indexed_slice(&self.cell_run_indices, index.run_indices),
            paragraph_indices: indexed_slice(&self.cell_paragraph_indices, index.paragraph_indices),
            nested_table_indices: indexed_slice(
                &self.cell_nested_table_indices,
                index.nested_table_indices,
            ),
        })
    }

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    fn calculate_estimated_bytes(&self) -> usize {
        memory::calculate_estimated_bytes(self)
    }

    pub(crate) const fn generation(&self) -> RichTextParserGeneration {
        self.generation
    }
}

fn table_cell_projection_indices(
    parsed: &RichParseResult,
    max_projection_indices: usize,
) -> Result<
    (
        Vec<RichTableCellProjectionIndex>,
        Vec<u32>,
        Vec<u32>,
        Vec<u32>,
    ),
    RichTextParseError,
> {
    let run_index = RichRangeIntervalIndex::new(
        parsed
            .runs
            .iter()
            .enumerate()
            .map(|(index, run)| {
                Ok(RichRangeIntervalEntry {
                    byte_range: run.byte_range,
                    source_index: checked_artifact_index("run", index)?,
                })
            })
            .collect::<Result<Vec<_>, RichTextParseError>>()?,
    );
    let paragraph_index = RichRangeIntervalIndex::new(
        parsed
            .paragraphs
            .iter()
            .enumerate()
            .map(|(index, (byte_range, _))| {
                Ok(RichRangeIntervalEntry {
                    byte_range: *byte_range,
                    source_index: checked_artifact_index("paragraph", index)?,
                })
            })
            .collect::<Result<Vec<_>, RichTextParseError>>()?,
    );
    let table_index = RichRangeIntervalIndex::new(
        parsed
            .tables
            .iter()
            .enumerate()
            .map(|(index, table)| {
                Ok(RichRangeIntervalEntry {
                    byte_range: table.byte_range,
                    source_index: checked_artifact_index("table", index)?,
                })
            })
            .collect::<Result<Vec<_>, RichTextParseError>>()?,
    );
    let mut projections = Vec::new();
    let mut run_indices = Vec::new();
    let mut paragraph_indices = Vec::new();
    let mut nested_table_indices = Vec::new();
    for table in &parsed.tables {
        for cell in &table.cells {
            let run_start = checked_artifact_index("cell run projection", run_indices.len())?;
            let mut projected_runs = Vec::new();
            run_index.collect_intersections(
                cell.byte_range,
                run_indices
                    .len()
                    .saturating_add(paragraph_indices.len())
                    .saturating_add(nested_table_indices.len()),
                max_projection_indices,
                &mut projected_runs,
            )?;
            projected_runs.sort_unstable();
            run_indices.extend(projected_runs);
            let paragraph_start =
                checked_artifact_index("cell paragraph projection", paragraph_indices.len())?;
            let mut projected_paragraphs = Vec::new();
            paragraph_index.collect_intersections(
                cell.byte_range,
                run_indices
                    .len()
                    .saturating_add(paragraph_indices.len())
                    .saturating_add(nested_table_indices.len()),
                max_projection_indices,
                &mut projected_paragraphs,
            )?;
            projected_paragraphs.sort_unstable();
            paragraph_indices.extend(projected_paragraphs);
            let nested_table_start =
                checked_artifact_index("cell nested-table projection", nested_table_indices.len())?;
            let mut projected_tables = Vec::new();
            table_index.collect_intersections(
                cell.byte_range,
                0,
                parsed.tables.len(),
                &mut projected_tables,
            )?;
            projected_tables.retain(|index| {
                parsed.tables.get(*index as usize).is_some_and(|nested| {
                    nested.depth > table.depth && range_contains(cell.byte_range, nested.byte_range)
                })
            });
            projected_tables.sort_unstable();
            admit_projection_indices(
                run_indices
                    .len()
                    .saturating_add(paragraph_indices.len())
                    .saturating_add(nested_table_indices.len()),
                projected_tables.len(),
                max_projection_indices,
            )?;
            nested_table_indices.extend(projected_tables);
            projections.push(RichTableCellProjectionIndex {
                parent_table_depth: table.depth,
                byte_range: cell.byte_range,
                run_indices: (
                    run_start,
                    checked_artifact_index("cell run projection", run_indices.len())?,
                ),
                paragraph_indices: (
                    paragraph_start,
                    checked_artifact_index("cell paragraph projection", paragraph_indices.len())?,
                ),
                nested_table_indices: (
                    nested_table_start,
                    checked_artifact_index(
                        "cell nested-table projection",
                        nested_table_indices.len(),
                    )?,
                ),
            });
        }
    }
    projections.sort_unstable_by_key(|index| {
        (
            index.parent_table_depth,
            index.byte_range.0,
            index.byte_range.1,
        )
    });
    Ok((
        projections,
        run_indices,
        paragraph_indices,
        nested_table_indices,
    ))
}

fn admit_projection_indices(
    consumed_indices: usize,
    additional_indices: usize,
    max_indices: usize,
) -> Result<(), RichTextParseError> {
    let attempted_indices = consumed_indices
        .checked_add(additional_indices)
        .unwrap_or(usize::MAX);
    if attempted_indices > max_indices {
        return Err(RichTextParseError::ProjectionIndexBudgetExceeded {
            attempted_indices,
            max_indices,
        });
    }
    Ok(())
}

fn ranges_intersect(left: (u32, u32), right: (u32, u32)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

fn range_contains(outer: (u32, u32), inner: (u32, u32)) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}

fn indexed_slice(values: &[u32], range: (u32, u32)) -> &[u32] {
    let start = range.0 as usize;
    let end = range.1 as usize;
    values.get(start..end).unwrap_or_default()
}

#[cfg(test)]
mod tests {
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
            populated.estimated_bytes()
                >= empty.estimated_bytes().saturating_add(alternative.len())
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
}
