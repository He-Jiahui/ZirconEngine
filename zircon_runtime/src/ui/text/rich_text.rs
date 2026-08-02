use crate::text::font::shared_font_database_generation;
use crate::text::layout::{RichTextLayoutRun, RichTextLayoutSource};
use crate::text::{
    CompiledRichText, InlineObjectRef, LinkRef, ParagraphOverride, RichTable, RichTextFormat,
    SharedTextLayoutSession, StyleOverride, StyledRun, build_resolved_text_glyph_artifact,
    register_compiled_rich_text_artifact, register_resolved_text_glyph_artifact,
    resolve_compiled_rich_text_artifact, resolve_resolved_text_glyph_artifact,
    rich::compile_rich_text,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRichTextFormat, UiTextRange, UiTextRunKind,
};

mod link_hit;

pub(crate) use link_hit::link_at_layout_point;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiTextSourceRun {
    pub kind: UiTextRunKind,
    pub source_range: UiTextRange,
    rich_source_range: UiTextRange,
    rich: Arc<CompiledRichText>,
    rich_run_index: usize,
}

impl UiTextSourceRun {
    pub(crate) fn text(&self) -> &str {
        self.rich
            .text()
            .get(self.rich_source_range.start..self.rich_source_range.end)
            .unwrap_or_default()
    }

    pub(crate) fn style(&self) -> &StyleOverride {
        &self.rich_run().style
    }

    pub(crate) fn inline(&self) -> Option<&InlineObjectRef> {
        self.rich_run().inline.as_ref()
    }

    pub(crate) fn link(&self) -> Option<&LinkRef> {
        self.rich_run().link.as_ref()
    }

    pub(crate) fn subrange(&self, start: usize, end: usize) -> Option<Self> {
        let start = self.source_range.start.max(start);
        let end = self.source_range.end.min(end);
        (start < end).then(|| Self {
            kind: self.kind,
            source_range: UiTextRange { start, end },
            rich_source_range: UiTextRange {
                start: self.rich_source_range.start + start - self.source_range.start,
                end: self.rich_source_range.start + end - self.source_range.start,
            },
            rich: Arc::clone(&self.rich),
            rich_run_index: self.rich_run_index,
        })
    }

    fn rich_run(&self) -> &StyledRun {
        &self.rich.parsed().runs[self.rich_run_index]
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiTextParagraphSource {
    source_range: UiTextRange,
    list_prefix: Option<UiTextRange>,
    rich_paragraph_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiParsedText {
    pub runs: Vec<UiTextSourceRun>,
    pub rich: Arc<CompiledRichText>,
    source_range: UiTextRange,
    paragraphs: Vec<UiTextParagraphSource>,
    table_indices: Vec<u32>,
    table_root_depth: u16,
}

impl UiParsedText {
    pub(crate) fn from_compiled(rich: Arc<CompiledRichText>) -> Self {
        let text_length = rich.text().len();
        let run_indices = (0..rich.parsed().runs.len())
            .filter_map(|index| u32::try_from(index).ok())
            .collect();
        let paragraph_indices = (0..rich.parsed().paragraphs.len())
            .filter_map(|index| u32::try_from(index).ok())
            .collect();
        let table_indices = (0..rich.parsed().tables.len())
            .filter_map(|index| u32::try_from(index).ok())
            .collect();
        Self::from_projection(
            rich,
            UiTextRange {
                start: 0,
                end: text_length,
            },
            &run_indices,
            &paragraph_indices,
            table_indices,
            0,
        )
    }

    pub(crate) fn text(&self) -> &str {
        self.rich
            .text()
            .get(self.source_range.start..self.source_range.end)
            .unwrap_or_default()
    }

    pub(crate) fn source_offset(&self) -> usize {
        self.source_range.start
    }

    pub(crate) fn project_range(
        &self,
        local_range: std::ops::Range<usize>,
        parent_table_depth: Option<u16>,
    ) -> Self {
        let start = local_range.start.min(self.text().len());
        let end = local_range.end.min(self.text().len()).max(start);
        let absolute_range = UiTextRange {
            start: self.source_range.start + start,
            end: self.source_range.start + end,
        };
        if let Some(depth) = parent_table_depth {
            if let Some(indices) = self.rich.cell_projection_indices(
                depth,
                (
                    u32::try_from(absolute_range.start).unwrap_or(u32::MAX),
                    u32::try_from(absolute_range.end).unwrap_or(u32::MAX),
                ),
            ) {
                return Self::from_projection(
                    Arc::clone(&self.rich),
                    absolute_range,
                    indices.run_indices,
                    indices.paragraph_indices,
                    indices.nested_table_indices.to_vec(),
                    depth.saturating_add(1),
                );
            }
        }

        let run_indices = self
            .runs
            .iter()
            .filter(|run| ranges_intersect(run.rich_source_range, absolute_range))
            .filter_map(|run| u32::try_from(run.rich_run_index).ok())
            .collect::<Vec<_>>();
        let paragraph_indices = self
            .paragraphs
            .iter()
            .filter(|paragraph| {
                let range = self.rich.parsed().paragraphs[paragraph.rich_paragraph_index].0;
                ranges_intersect(u32_range(range), absolute_range)
            })
            .filter_map(|paragraph| u32::try_from(paragraph.rich_paragraph_index).ok())
            .collect::<Vec<_>>();
        let table_indices = self
            .table_indices
            .iter()
            .copied()
            .filter(|index| {
                self.rich
                    .parsed()
                    .tables
                    .get(*index as usize)
                    .is_some_and(|table| {
                        range_contains(absolute_range, u32_range(table.byte_range))
                            && parent_table_depth.is_none_or(|depth| table.depth > depth)
                    })
            })
            .collect();
        Self::from_projection(
            Arc::clone(&self.rich),
            absolute_range,
            &run_indices,
            &paragraph_indices,
            table_indices,
            parent_table_depth
                .map(|depth| depth.saturating_add(1))
                .unwrap_or(self.table_root_depth),
        )
    }

    pub(crate) fn paragraphs(
        &self,
    ) -> impl Iterator<Item = (UiTextRange, &ParagraphOverride, Option<UiTextRange>)> {
        self.paragraphs.iter().filter_map(|source| {
            let paragraph = self
                .rich
                .parsed()
                .paragraphs
                .get(source.rich_paragraph_index)?;
            Some((source.source_range, &paragraph.1, source.list_prefix))
        })
    }

    pub(crate) fn tables(&self) -> impl Iterator<Item = &RichTable> {
        self.table_indices
            .iter()
            .filter_map(|index| self.rich.parsed().tables.get(*index as usize))
    }

    pub(crate) fn table_root_depth(&self) -> u16 {
        self.table_root_depth
    }

    fn from_projection(
        rich: Arc<CompiledRichText>,
        source_range: UiTextRange,
        run_indices: &[u32],
        paragraph_indices: &[u32],
        mut table_indices: Vec<u32>,
        table_root_depth: u16,
    ) -> Self {
        let mut run_indices = run_indices.to_vec();
        run_indices.sort_unstable();
        run_indices.dedup();
        let runs = run_indices
            .into_iter()
            .filter_map(|index| {
                let rich_run_index = usize::try_from(index).ok()?;
                let run = rich.parsed().runs.get(rich_run_index)?;
                let rich_source_range = intersect_range(u32_range(run.byte_range), source_range)?;
                let source_range = UiTextRange {
                    start: rich_source_range.start - source_range.start,
                    end: rich_source_range.end - source_range.start,
                };
                (source_range.start < source_range.end).then(|| UiTextSourceRun {
                    kind: ui_run_kind(&run.style, run.link.as_ref()),
                    source_range,
                    rich_source_range,
                    rich: Arc::clone(&rich),
                    rich_run_index,
                })
            })
            .collect();
        let mut paragraph_indices = paragraph_indices.to_vec();
        paragraph_indices.sort_unstable();
        paragraph_indices.dedup();
        let paragraphs = paragraph_indices
            .into_iter()
            .filter_map(|index| {
                let rich_paragraph_index = usize::try_from(index).ok()?;
                let (range, paragraph) = rich.parsed().paragraphs.get(rich_paragraph_index)?;
                let rich_range = intersect_range(u32_range(*range), source_range)?;
                let local_range = UiTextRange {
                    start: rich_range.start - source_range.start,
                    end: rich_range.end - source_range.start,
                };
                let list_prefix = paragraph
                    .list_prefix
                    .map(u32_range)
                    .and_then(|prefix| range_contains(source_range, prefix).then_some(prefix))
                    .map(|prefix| UiTextRange {
                        start: prefix.start - source_range.start,
                        end: prefix.end - source_range.start,
                    });
                Some(UiTextParagraphSource {
                    source_range: local_range,
                    list_prefix,
                    rich_paragraph_index,
                })
            })
            .collect();
        table_indices.sort_unstable();
        table_indices.dedup();
        Self {
            runs,
            rich,
            source_range,
            paragraphs,
            table_indices,
            table_root_depth,
        }
    }
}

impl RichTextLayoutSource for UiParsedText {
    fn text(&self) -> &str {
        UiParsedText::text(self)
    }

    fn run_count(&self) -> usize {
        self.runs.len()
    }

    fn run(&self, index: usize) -> Option<RichTextLayoutRun<'_>> {
        let source = self.runs.get(index)?;
        let run = source.rich_run();
        Some(RichTextLayoutRun {
            source_index: u32::try_from(source.rich_run_index).unwrap_or(u32::MAX),
            byte_range: (
                u32::try_from(source.source_range.start).ok()?,
                u32::try_from(source.source_range.end).ok()?,
            ),
            style: &run.style,
            inline: run.inline.as_ref(),
        })
    }
}

fn u32_range(range: (u32, u32)) -> UiTextRange {
    UiTextRange {
        start: range.0 as usize,
        end: range.1 as usize,
    }
}

fn intersect_range(left: UiTextRange, right: UiTextRange) -> Option<UiTextRange> {
    let start = left.start.max(right.start);
    let end = left.end.min(right.end);
    (start < end).then_some(UiTextRange { start, end })
}

fn ranges_intersect(left: UiTextRange, right: UiTextRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn range_contains(outer: UiTextRange, inner: UiTextRange) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

pub(crate) fn parse_source_text(text: &str, format: RichTextFormat) -> UiParsedText {
    let rich = compile_rich_text(text, format);
    UiParsedText::from_compiled(rich)
}

pub(crate) fn prepare_render_command_text_artifacts(commands: &mut [UiRenderCommand]) {
    let mut provider = SharedTextLayoutSession::new();
    let font_generation = shared_font_database_generation();
    for command in commands {
        if matches!(command.style.rich_text_format, UiRichTextFormat::Plain) {
            let style = command.style.clone();
            let Some(text) = command.text.clone() else {
                continue;
            };
            let Some(layout) = command.text_layout.as_mut() else {
                continue;
            };
            let artifact_is_current = layout
                .rich_text_artifact
                .as_ref()
                .and_then(resolve_resolved_text_glyph_artifact)
                .is_some_and(|artifact| artifact.font_generation == font_generation);
            if artifact_is_current {
                continue;
            }
            layout.rich_text_artifact =
                build_resolved_text_glyph_artifact(text.as_str(), &style, layout, &mut provider)
                    .map(|artifact| register_resolved_text_glyph_artifact(Arc::new(artifact)));
            continue;
        }
        let Some(layout) = command.text_layout.as_mut() else {
            continue;
        };
        if layout
            .rich_text_artifact
            .as_ref()
            .is_some_and(|handle| resolve_compiled_rich_text_artifact(handle).is_some())
        {
            continue;
        }
        let Some(markup) = command.text.as_deref() else {
            continue;
        };
        let compiled = compile_rich_text(markup, command.style.rich_text_format.into());
        layout.rich_text_artifact = Some(register_compiled_rich_text_artifact(compiled));
    }
}

fn ui_run_kind(style: &StyleOverride, link: Option<&LinkRef>) -> UiTextRunKind {
    if link.is_some() {
        UiTextRunKind::Link
    } else if style.code == Some(true) {
        UiTextRunKind::Code
    } else if style.weight.is_some_and(|weight| weight >= 600) {
        UiTextRunKind::Strong
    } else if style.italic == Some(true) {
        UiTextRunKind::Emphasis
    } else {
        UiTextRunKind::Plain
    }
}

#[cfg(test)]
mod tests;
