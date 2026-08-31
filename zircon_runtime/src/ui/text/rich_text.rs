use crate::core::framework::text::TextLayoutError;
use crate::text::layout::{RichTextLayoutRun, RichTextLayoutSource};
use crate::text::shaping::TextShapingOutcome;
use crate::text::{
    CompiledRichText, InlineObjectRef, LinkRef, ParagraphOverride, RichTable, RichTextFormat,
    RichTextParseError, SharedTextLayoutSession, StyleOverride, StyledRun,
    build_resolved_rich_text_glyph_artifact, build_resolved_text_glyph_artifact,
    build_resolved_text_presentation_glyph_artifact, register_compiled_rich_text_artifact,
    register_resolved_rich_text_artifact_with_layout_runs, register_resolved_text_glyph_artifact,
    resolve_compiled_rich_text_artifact, resolve_resolved_text_glyph_artifact,
    resolve_rich_text_virtual_line_sequences_for_layout,
    resolved_rich_text_artifact_matches_layout_snapshot,
    resolved_text_glyph_artifact_matches_layout_snapshot,
};
use std::{mem::size_of, sync::Arc};
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRichTextFormat, UiTextRange, UiTextRunKind,
};

mod link_hit;

pub(crate) use link_hit::link_at_layout_point;

use super::presentation::is_secure_text_presentation_artifact;

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
    pub(crate) fn from_compiled(rich: Arc<CompiledRichText>) -> Result<Self, TextLayoutError> {
        let text_length = rich.text().len();
        let run_indices: Vec<u32> = (0..rich.parsed().runs.len())
            .map(checked_projection_index)
            .collect::<Result<_, _>>()?;
        let paragraph_indices: Vec<u32> = (0..rich.parsed().paragraphs.len())
            .map(checked_projection_index)
            .collect::<Result<_, _>>()?;
        let table_indices: Vec<u32> = (0..rich.parsed().tables.len())
            .map(checked_projection_index)
            .collect::<Result<_, _>>()?;
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
    ) -> Result<Self, TextLayoutError> {
        if local_range.start > local_range.end
            || local_range.end > self.text().len()
            || !self.text().is_char_boundary(local_range.start)
            || !self.text().is_char_boundary(local_range.end)
        {
            return Err(TextLayoutError::LayoutFailed);
        }
        let start = local_range.start;
        let end = local_range.end;
        let absolute_range = UiTextRange {
            start: self
                .source_range
                .start
                .checked_add(start)
                .ok_or(TextLayoutError::LayoutFailed)?,
            end: self
                .source_range
                .start
                .checked_add(end)
                .ok_or(TextLayoutError::LayoutFailed)?,
        };
        if let Some(depth) = parent_table_depth {
            if let Some(indices) = self.rich.cell_projection_indices(
                depth,
                (
                    u32::try_from(absolute_range.start)
                        .map_err(|_| TextLayoutError::LayoutFailed)?,
                    u32::try_from(absolute_range.end).map_err(|_| TextLayoutError::LayoutFailed)?,
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
            .map(|run| checked_projection_index(run.rich_run_index))
            .collect::<Result<Vec<_>, _>>()?;
        let paragraph_indices = self
            .paragraphs
            .iter()
            .filter(|paragraph| {
                let range = self.rich.parsed().paragraphs[paragraph.rich_paragraph_index].0;
                ranges_intersect(u32_range(range), absolute_range)
            })
            .map(|paragraph| checked_projection_index(paragraph.rich_paragraph_index))
            .collect::<Result<Vec<_>, _>>()?;
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

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.rich
            .estimated_bytes()
            .saturating_add(
                self.runs
                    .capacity()
                    .saturating_mul(size_of::<UiTextSourceRun>()),
            )
            .saturating_add(
                self.paragraphs
                    .capacity()
                    .saturating_mul(size_of::<UiTextParagraphSource>()),
            )
            .saturating_add(
                self.table_indices
                    .capacity()
                    .saturating_mul(size_of::<u32>()),
            )
            .saturating_add(size_of::<Self>())
    }

    fn from_projection(
        rich: Arc<CompiledRichText>,
        source_range: UiTextRange,
        run_indices: &[u32],
        paragraph_indices: &[u32],
        table_indices: Vec<u32>,
        table_root_depth: u16,
    ) -> Result<Self, TextLayoutError> {
        let runs = run_indices
            .iter()
            .copied()
            .map(
                |index| -> Result<Option<UiTextSourceRun>, TextLayoutError> {
                    let rich_run_index =
                        usize::try_from(index).map_err(|_| TextLayoutError::LayoutFailed)?;
                    let run = rich
                        .parsed()
                        .runs
                        .get(rich_run_index)
                        .ok_or(TextLayoutError::LayoutFailed)?;
                    let Some(rich_source_range) =
                        intersect_range(u32_range(run.byte_range), source_range)
                    else {
                        return Ok(None);
                    };
                    let source_range = UiTextRange {
                        start: rich_source_range.start - source_range.start,
                        end: rich_source_range.end - source_range.start,
                    };
                    Ok(
                        (source_range.start < source_range.end).then(|| UiTextSourceRun {
                            kind: ui_run_kind(&run.style, run.link.as_ref()),
                            source_range,
                            rich_source_range,
                            rich: Arc::clone(&rich),
                            rich_run_index,
                        }),
                    )
                },
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let paragraphs = paragraph_indices
            .iter()
            .copied()
            .map(
                |index| -> Result<Option<UiTextParagraphSource>, TextLayoutError> {
                    let rich_paragraph_index =
                        usize::try_from(index).map_err(|_| TextLayoutError::LayoutFailed)?;
                    let (range, paragraph) = rich
                        .parsed()
                        .paragraphs
                        .get(rich_paragraph_index)
                        .ok_or(TextLayoutError::LayoutFailed)?;
                    let Some(rich_range) = intersect_range(u32_range(*range), source_range) else {
                        return Ok(None);
                    };
                    let local_range = UiTextRange {
                        start: rich_range.start - source_range.start,
                        end: rich_range.end - source_range.start,
                    };
                    let list_prefix = paragraph
                        .list_item
                        .as_ref()
                        .map(|item| item.marker_range)
                        .map(u32_range)
                        .and_then(|prefix| range_contains(source_range, prefix).then_some(prefix))
                        .map(|prefix| UiTextRange {
                            start: prefix.start - source_range.start,
                            end: prefix.end - source_range.start,
                        });
                    Ok(Some(UiTextParagraphSource {
                        source_range: local_range,
                        list_prefix,
                        rich_paragraph_index,
                    }))
                },
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        for index in &table_indices {
            let index = usize::try_from(*index).map_err(|_| TextLayoutError::LayoutFailed)?;
            rich.parsed()
                .tables
                .get(index)
                .ok_or(TextLayoutError::LayoutFailed)?;
        }
        Ok(Self {
            runs,
            rich,
            source_range,
            paragraphs,
            table_indices,
            table_root_depth,
        })
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
            source_index: u32::try_from(source.rich_run_index).ok()?,
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

fn checked_projection_index(index: usize) -> Result<u32, TextLayoutError> {
    u32::try_from(index).map_err(|_| TextLayoutError::LayoutFailed)
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

pub(crate) fn parse_source_text_with_provider(
    text: &str,
    format: RichTextFormat,
    provider: &SharedTextLayoutSession,
) -> Result<UiParsedText, TextLayoutError> {
    let rich = provider
        .compile_rich_text(text, format)
        .map_err(rich_parse_layout_error)?;
    UiParsedText::from_compiled(rich)
}

#[cfg(test)]
pub(crate) fn parse_source_text(
    text: &str,
    format: RichTextFormat,
) -> Result<UiParsedText, TextLayoutError> {
    let rich = crate::text::rich::parser_registry::compile_rich_text(text, format)
        .map_err(rich_parse_layout_error)?;
    UiParsedText::from_compiled(rich)
}

fn rich_parse_layout_error(error: RichTextParseError) -> TextLayoutError {
    match error {
        RichTextParseError::DecoratorPanicked { .. }
        | RichTextParseError::ParserIdentityExhausted
        | RichTextParseError::BidiControlNotAllowed { .. }
        | RichTextParseError::UnbalancedBidiControl { .. } => TextLayoutError::LayoutFailed,
        _ => TextLayoutError::RichTextBudgetExceeded,
    }
}

#[cfg(test)]
pub(crate) fn prepare_render_command_text_artifacts(commands: &mut [UiRenderCommand]) {
    let mut provider = SharedTextLayoutSession::new();
    prepare_render_command_text_artifacts_with_provider(commands, &mut provider);
}

pub(crate) fn prepare_render_command_text_artifacts_with_provider(
    commands: &mut [UiRenderCommand],
    provider: &mut SharedTextLayoutSession,
) {
    let font_revision = provider.font_collection_revision();
    for command in commands {
        if matches!(command.style.rich_text_format, UiRichTextFormat::Plain) {
            let mut style = command.style.clone();
            let Some(text) = command.text.clone() else {
                continue;
            };
            let Some(layout) = command.text_layout.as_mut() else {
                continue;
            };
            style.font_size = layout.font_size;
            style.line_height = layout.line_height;
            let is_secure_presentation = layout
                .rich_text_artifact
                .as_ref()
                .is_some_and(is_secure_text_presentation_artifact);
            let artifact_is_current = layout
                .rich_text_artifact
                .as_ref()
                .and_then(resolve_resolved_text_glyph_artifact)
                .is_some_and(|artifact| {
                    resolved_text_glyph_artifact_matches_layout_snapshot(
                        artifact.as_ref(),
                        text.as_str(),
                        &style,
                        layout,
                        font_revision,
                    )
                });
            if artifact_is_current {
                continue;
            }
            if is_secure_presentation {
                layout.rich_text_artifact = match build_resolved_text_presentation_glyph_artifact(
                    text.as_str(),
                    &style,
                    layout,
                    provider,
                ) {
                    TextShapingOutcome::Ready(Some(artifact)) => {
                        Some(register_resolved_text_glyph_artifact(Arc::new(artifact)))
                    }
                    TextShapingOutcome::Ready(None) => None,
                    TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
                        provider.record_layout_error(&error);
                        None
                    }
                };
            } else {
                layout.rich_text_artifact = match build_resolved_text_glyph_artifact(
                    text.as_str(),
                    &style,
                    layout,
                    provider,
                ) {
                    TextShapingOutcome::Ready(Some(artifact)) => {
                        Some(register_resolved_text_glyph_artifact(Arc::new(artifact)))
                    }
                    TextShapingOutcome::Ready(None) => None,
                    TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
                        provider.record_layout_error(&error);
                        None
                    }
                };
            }
            continue;
        }
        let Some(layout) = command.text_layout.as_mut() else {
            continue;
        };
        let mut style = command.style.clone();
        style.font_size = layout.font_size;
        style.line_height = layout.line_height;
        let existing_compiled = layout
            .rich_text_artifact
            .as_ref()
            .and_then(resolve_compiled_rich_text_artifact);
        let artifact_is_current = layout
            .rich_text_artifact
            .as_ref()
            .zip(existing_compiled.as_ref())
            .is_some_and(|(handle, compiled)| {
                resolved_rich_text_artifact_matches_layout_snapshot(
                    handle,
                    compiled.text(),
                    &style,
                    layout,
                    font_revision,
                )
            });
        if artifact_is_current {
            continue;
        }
        let compiled = match existing_compiled {
            Some(compiled) => compiled,
            None => {
                let Some(markup) = command.text.as_deref() else {
                    continue;
                };
                match provider.compile_rich_text(markup, command.style.rich_text_format.into()) {
                    Ok(compiled) => compiled,
                    Err(error) => {
                        provider.record_layout_error(&rich_parse_layout_error(error));
                        layout.rich_text_artifact = None;
                        continue;
                    }
                }
            }
        };
        let parsed = match UiParsedText::from_compiled(Arc::clone(&compiled)) {
            Ok(parsed) => parsed,
            Err(error) => {
                provider.record_layout_error(&error);
                layout.rich_text_artifact = None;
                continue;
            }
        };
        let retained_virtual_line_sequences =
            layout.rich_text_artifact.as_ref().and_then(|handle| {
                resolve_rich_text_virtual_line_sequences_for_layout(
                    handle,
                    compiled.text(),
                    &style,
                    layout,
                )
            });
        layout.rich_text_artifact = match build_resolved_rich_text_glyph_artifact(
            &parsed,
            compiled.shared_text(),
            &style,
            layout,
            retained_virtual_line_sequences.as_deref(),
            provider,
        ) {
            TextShapingOutcome::Ready(Some(artifact)) => {
                Some(register_resolved_rich_text_artifact_with_layout_runs(
                    compiled,
                    artifact.artifact,
                    Arc::from(layout.lines.clone()),
                    artifact.glyph_runs,
                ))
            }
            TextShapingOutcome::Ready(None) => Some(register_compiled_rich_text_artifact(compiled)),
            TextShapingOutcome::Deferred(error) | TextShapingOutcome::Failed(error) => {
                provider.record_layout_error(&error);
                Some(register_compiled_rich_text_artifact(compiled))
            }
        };
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
