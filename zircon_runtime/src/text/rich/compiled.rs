use std::mem::size_of;
use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::resource::ResourceId;
use crate::text::{InlineObjectRef, OpenTypeFeature, RichParseResult, RichTextFormat, StyledRun};

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
pub(crate) struct RichTableCellProjectionIndices<'a> {
    pub(crate) run_indices: &'a [u32],
    pub(crate) paragraph_indices: &'a [u32],
    pub(crate) nested_table_indices: &'a [u32],
}

/// Canonical, generation-owned result shared by every rich-text consumer.
#[derive(Debug, PartialEq)]
pub struct CompiledRichText {
    source_markup: Arc<str>,
    format: RichTextFormat,
    generation: RichTextParserGeneration,
    parsed: RichParseResult,
    cluster_ranges: Arc<[(u32, u32)]>,
    inline_run_indices: Arc<[u32]>,
    link_run_indices: Arc<[u32]>,
    resource_ids: Arc<[ResourceId]>,
    table_cell_projection_indices: Arc<[RichTableCellProjectionIndex]>,
    cell_run_indices: Arc<[u32]>,
    cell_paragraph_indices: Arc<[u32]>,
    cell_nested_table_indices: Arc<[u32]>,
    estimated_bytes: usize,
}

impl CompiledRichText {
    pub(crate) fn new(
        source_markup: Arc<str>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
        parsed: RichParseResult,
    ) -> Self {
        let cluster_ranges = parsed
            .text
            .grapheme_indices(true)
            .map(|(start, grapheme)| (to_u32(start), to_u32(start + grapheme.len())))
            .collect::<Vec<_>>();
        let inline_run_indices = parsed
            .runs
            .iter()
            .enumerate()
            .filter_map(|(index, run)| run.inline.is_some().then(|| to_u32(index)))
            .collect::<Vec<_>>();
        let link_run_indices = parsed
            .runs
            .iter()
            .enumerate()
            .filter_map(|(index, run)| run.link.is_some().then(|| to_u32(index)))
            .collect::<Vec<_>>();
        let mut resource_ids = parsed
            .runs
            .iter()
            .filter_map(|run| match run.inline.as_ref() {
                Some(InlineObjectRef::Image { texture, .. }) => Some(*texture),
                Some(InlineObjectRef::Icon { .. } | InlineObjectRef::Widget { .. }) | None => None,
            })
            .collect::<Vec<_>>();
        resource_ids.sort_unstable();
        resource_ids.dedup();
        let (
            table_cell_projection_indices,
            cell_run_indices,
            cell_paragraph_indices,
            cell_nested_table_indices,
        ) = table_cell_projection_indices(&parsed);
        let mut compiled = Self {
            source_markup,
            format,
            generation,
            parsed,
            cluster_ranges: Arc::from(cluster_ranges.into_boxed_slice()),
            inline_run_indices: Arc::from(inline_run_indices.into_boxed_slice()),
            link_run_indices: Arc::from(link_run_indices.into_boxed_slice()),
            resource_ids: Arc::from(resource_ids.into_boxed_slice()),
            table_cell_projection_indices: Arc::from(
                table_cell_projection_indices.into_boxed_slice(),
            ),
            cell_run_indices: Arc::from(cell_run_indices.into_boxed_slice()),
            cell_paragraph_indices: Arc::from(cell_paragraph_indices.into_boxed_slice()),
            cell_nested_table_indices: Arc::from(cell_nested_table_indices.into_boxed_slice()),
            estimated_bytes: 0,
        };
        compiled.estimated_bytes = compiled.calculate_estimated_bytes();
        compiled
    }

    pub(crate) fn from_projection(parsed: RichParseResult) -> Self {
        Self::new(
            Arc::from(""),
            RichTextFormat::Plain,
            RichTextParserGeneration::default(),
            parsed,
        )
    }

    pub fn source_markup(&self) -> &str {
        &self.source_markup
    }

    pub const fn format(&self) -> RichTextFormat {
        self.format
    }

    pub fn parsed(&self) -> &RichParseResult {
        &self.parsed
    }

    pub fn text(&self) -> &str {
        &self.parsed.text
    }

    pub fn shared_text(&self) -> Arc<str> {
        Arc::clone(&self.parsed.text)
    }

    pub fn cluster_ranges(&self) -> &[(u32, u32)] {
        &self.cluster_ranges
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

    pub fn resource_ids(&self) -> &[ResourceId] {
        &self.resource_ids
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
        let run_metadata_bytes = self
            .parsed
            .runs
            .iter()
            .map(|run| {
                run.style
                    .family
                    .as_ref()
                    .map_or(0, |family| family.as_str().len())
                    + run.style.features.as_ref().map_or(0, |features| {
                        features.capacity() * size_of::<OpenTypeFeature>()
                    })
                    + run.link.as_ref().map_or(0, |link| link.href.capacity())
            })
            .sum::<usize>();
        let table_bytes = self
            .parsed
            .tables
            .iter()
            .map(|table| {
                table.columns.capacity() * size_of::<crate::text::RichTableColumn>()
                    + table.cells.capacity() * size_of::<crate::text::RichTableCell>()
            })
            .sum::<usize>();
        size_of::<Self>()
            .saturating_add(self.source_markup.len())
            .saturating_add(self.parsed.text.len())
            .saturating_add(self.parsed.runs.capacity() * size_of::<crate::text::StyledRun>())
            .saturating_add(
                self.parsed.paragraphs.capacity()
                    * size_of::<((u32, u32), crate::text::ParagraphOverride)>(),
            )
            .saturating_add(self.parsed.tables.capacity() * size_of::<crate::text::RichTable>())
            .saturating_add(self.cluster_ranges.len() * size_of::<(u32, u32)>())
            .saturating_add(self.inline_run_indices.len() * size_of::<u32>())
            .saturating_add(self.link_run_indices.len() * size_of::<u32>())
            .saturating_add(self.resource_ids.len() * size_of::<ResourceId>())
            .saturating_add(
                self.table_cell_projection_indices.len()
                    * size_of::<RichTableCellProjectionIndex>(),
            )
            .saturating_add(self.cell_run_indices.len() * size_of::<u32>())
            .saturating_add(self.cell_paragraph_indices.len() * size_of::<u32>())
            .saturating_add(self.cell_nested_table_indices.len() * size_of::<u32>())
            .saturating_add(run_metadata_bytes)
            .saturating_add(table_bytes)
    }

    pub(crate) const fn generation(&self) -> RichTextParserGeneration {
        self.generation
    }
}

fn table_cell_projection_indices(
    parsed: &RichParseResult,
) -> (
    Vec<RichTableCellProjectionIndex>,
    Vec<u32>,
    Vec<u32>,
    Vec<u32>,
) {
    let mut projections = Vec::new();
    let mut run_indices = Vec::new();
    let mut paragraph_indices = Vec::new();
    let mut nested_table_indices = Vec::new();
    for table in &parsed.tables {
        for cell in &table.cells {
            let run_start = to_u32(run_indices.len());
            run_indices.extend(
                parsed
                    .runs
                    .iter()
                    .enumerate()
                    .filter(|(_, run)| ranges_intersect(run.byte_range, cell.byte_range))
                    .map(|(index, _)| to_u32(index)),
            );
            let paragraph_start = to_u32(paragraph_indices.len());
            paragraph_indices.extend(
                parsed
                    .paragraphs
                    .iter()
                    .enumerate()
                    .filter(|(_, (range, _))| ranges_intersect(*range, cell.byte_range))
                    .map(|(index, _)| to_u32(index)),
            );
            let nested_table_start = to_u32(nested_table_indices.len());
            nested_table_indices.extend(
                parsed
                    .tables
                    .iter()
                    .enumerate()
                    .filter(|(_, nested)| {
                        nested.depth > table.depth
                            && range_contains(cell.byte_range, nested.byte_range)
                    })
                    .map(|(index, _)| to_u32(index)),
            );
            projections.push(RichTableCellProjectionIndex {
                parent_table_depth: table.depth,
                byte_range: cell.byte_range,
                run_indices: (run_start, to_u32(run_indices.len())),
                paragraph_indices: (paragraph_start, to_u32(paragraph_indices.len())),
                nested_table_indices: (nested_table_start, to_u32(nested_table_indices.len())),
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
    (
        projections,
        run_indices,
        paragraph_indices,
        nested_table_indices,
    )
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

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::CompiledRichText;
    use crate::text::{
        ParagraphOverride, RichParseResult, RichTable, RichTableCell, RichTableColumn, StyledRun,
    };

    #[test]
    fn compiled_rich_text_indexes_each_table_cell_projection() {
        let rich = CompiledRichText::from_projection(RichParseResult {
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
}
