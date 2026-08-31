use std::ops::Range;

use super::{TextDocument, TextDocumentEditError};

/// Stable identity for one source hard line within a retained document authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TextDocumentHardLineId {
    creation_revision: u64,
    creation_ordinal: usize,
}

/// Separator-aware source model retained independently from wrapped visual lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextDocumentHardLineModel {
    id: TextDocumentHardLineId,
    content_len: usize,
    separator_len: usize,
}

impl TextDocumentHardLineModel {
    pub(crate) const fn id(&self) -> TextDocumentHardLineId {
        self.id
    }

    pub(crate) const fn content_len(&self) -> usize {
        self.content_len
    }

    pub(crate) const fn separator_len(&self) -> usize {
        self.separator_len
    }

    const fn source_len(&self) -> usize {
        self.content_len + self.separator_len
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextDocumentHardLineSpan {
    pub(crate) old: Range<usize>,
    pub(crate) new: Range<usize>,
}

#[derive(Debug)]
pub(super) struct TextDocumentHardLineModels {
    revision: u64,
    lines: Vec<TextDocumentHardLineModel>,
}

pub(super) struct PreparedHardLineEdit {
    kind: PreparedHardLineEditKind,
    receipt: TextDocumentHardLineSpan,
    revision: u64,
}

enum PreparedHardLineEditKind {
    LocalContent {
        model_index: usize,
        content_len: usize,
    },
    Reparse {
        model_range: Range<usize>,
        replacement: Vec<TextDocumentHardLineModel>,
    },
}

impl PreparedHardLineEdit {
    pub(super) fn receipt(&self) -> &TextDocumentHardLineSpan {
        &self.receipt
    }
}

impl TextDocumentHardLineModels {
    pub(super) fn new(revision: u64, source: &str) -> Self {
        Self {
            revision,
            lines: parsed_models(source, true, revision, 0),
        }
    }

    pub(super) fn lines(&self) -> &[TextDocumentHardLineModel] {
        &self.lines
    }

    pub(super) fn estimated_heap_bytes(&self) -> usize {
        self.lines
            .capacity()
            .saturating_mul(std::mem::size_of::<TextDocumentHardLineModel>())
    }

    fn prepare_edit(
        &self,
        document_len: usize,
        source_range: Range<usize>,
        old_source: &str,
        new_source: &str,
        model_range: Range<usize>,
        next_revision: u64,
    ) -> Result<PreparedHardLineEdit, TextDocumentEditError> {
        if self.revision.checked_add(1) != Some(next_revision)
            || model_range.end > self.lines.len()
            || source_range.len() != old_source.len()
        {
            return Err(TextDocumentEditError::HardLineInvariant);
        }

        let reaches_document_end = source_range.end == document_len;
        let old_lines = parsed_line_ranges(old_source, reaches_document_end);
        let new_lines = parsed_line_ranges(new_source, reaches_document_end);
        let old_models = &self.lines[model_range.clone()];
        if old_lines.len() != old_models.len() {
            return Err(TextDocumentEditError::HardLineInvariant);
        }

        let mut replacement = parsed_models(
            new_source,
            reaches_document_end,
            next_revision,
            model_range.start,
        );
        reconcile_stable_ids(
            old_source,
            &old_lines,
            old_models,
            new_source,
            &new_lines,
            &mut replacement,
        );
        let new_range = model_range.start..model_range.start + replacement.len();
        Ok(PreparedHardLineEdit {
            receipt: TextDocumentHardLineSpan {
                old: model_range.clone(),
                new: new_range,
            },
            kind: PreparedHardLineEditKind::Reparse {
                model_range,
                replacement,
            },
            revision: next_revision,
        })
    }

    pub(super) fn apply(&mut self, edit: PreparedHardLineEdit) {
        match edit.kind {
            PreparedHardLineEditKind::LocalContent {
                model_index,
                content_len,
            } => {
                self.lines[model_index].content_len = content_len;
            }
            PreparedHardLineEditKind::Reparse {
                model_range,
                replacement,
            } => {
                self.lines.splice(model_range, replacement);
            }
        }
        self.revision = edit.revision;
    }

    fn prepare_local_content_edit(
        &self,
        dirty: &Range<usize>,
        replacement: &str,
        next_revision: u64,
    ) -> Result<Option<PreparedHardLineEdit>, TextDocumentEditError> {
        if self.revision.checked_add(1) != Some(next_revision)
            || replacement.chars().any(crate::text::is_hard_line_separator)
        {
            return Ok(None);
        }
        let model_index = self.line_index_at_offset(dirty.start);
        let Some(model) = self.lines.get(model_index) else {
            return Err(TextDocumentEditError::HardLineInvariant);
        };
        let content_start = self.source_offset_for_line(model_index);
        let content_end = content_start
            .checked_add(model.content_len)
            .ok_or(TextDocumentEditError::LengthOverflow)?;
        if dirty.start < content_start || dirty.end > content_end {
            return Ok(None);
        }
        let content_len = model
            .content_len
            .checked_sub(dirty.len())
            .and_then(|retained| retained.checked_add(replacement.len()))
            .ok_or(TextDocumentEditError::LengthOverflow)?;
        let model_range = model_index..model_index + 1;
        Ok(Some(PreparedHardLineEdit {
            kind: PreparedHardLineEditKind::LocalContent {
                model_index,
                content_len,
            },
            receipt: TextDocumentHardLineSpan {
                old: model_range.clone(),
                new: model_range,
            },
            revision: next_revision,
        }))
    }

    fn reanalysis_envelope(&self, dirty: &Range<usize>) -> (Range<usize>, Range<usize>) {
        let first = self.line_index_at_offset(dirty.start).saturating_sub(1);
        let last_dirty_offset = if dirty.start == dirty.end {
            dirty.end
        } else {
            dirty.end - 1
        };
        let last = self
            .line_index_at_offset(last_dirty_offset)
            .saturating_add(2)
            .min(self.lines.len());
        let source_start = self.source_offset_for_line(first);
        let source_end = self.source_offset_for_line(last);
        (first..last, source_start..source_end)
    }

    fn line_index_at_offset(&self, offset: usize) -> usize {
        let mut source_end = 0usize;
        for (index, line) in self.lines.iter().enumerate() {
            source_end += line.source_len();
            if offset < source_end || index + 1 == self.lines.len() {
                return index;
            }
        }
        0
    }

    fn source_offset_for_line(&self, line_index: usize) -> usize {
        self.lines
            .iter()
            .take(line_index)
            .fold(0usize, |offset, line| offset + line.source_len())
    }
}

impl TextDocument {
    pub(crate) fn hard_line_models(&self) -> &[TextDocumentHardLineModel] {
        self.hard_line_models.lines()
    }

    pub(super) fn prepare_hard_line_edit(
        &self,
        dirty: Range<usize>,
        replacement: &str,
        next_revision: u64,
    ) -> Result<PreparedHardLineEdit, TextDocumentEditError> {
        if let Some(local) =
            self.hard_line_models
                .prepare_local_content_edit(&dirty, replacement, next_revision)?
        {
            return Ok(local);
        }
        let (model_range, source_range) = self.hard_line_models.reanalysis_envelope(&dirty);
        let old_source = self.snapshot_range_unchecked(source_range.clone());
        let relative_start = dirty
            .start
            .checked_sub(source_range.start)
            .ok_or(TextDocumentEditError::HardLineInvariant)?;
        let relative_end = dirty
            .end
            .checked_sub(source_range.start)
            .ok_or(TextDocumentEditError::HardLineInvariant)?;
        let removed_len = relative_end
            .checked_sub(relative_start)
            .ok_or(TextDocumentEditError::HardLineInvariant)?;
        let retained_len = old_source
            .len()
            .checked_sub(removed_len)
            .ok_or(TextDocumentEditError::HardLineInvariant)?;
        let new_len = retained_len
            .checked_add(replacement.len())
            .ok_or(TextDocumentEditError::LengthOverflow)?;
        let mut new_source = String::with_capacity(new_len);
        new_source.push_str(&old_source[..relative_start]);
        new_source.push_str(replacement);
        new_source.push_str(&old_source[relative_end..]);

        self.hard_line_models.prepare_edit(
            self.byte_len,
            source_range,
            &old_source,
            &new_source,
            model_range,
            next_revision,
        )
    }
}

fn parsed_models(
    source: &str,
    reaches_document_end: bool,
    creation_revision: u64,
    ordinal_start: usize,
) -> Vec<TextDocumentHardLineModel> {
    parsed_line_ranges(source, reaches_document_end)
        .into_iter()
        .enumerate()
        .map(|(index, line)| TextDocumentHardLineModel {
            id: TextDocumentHardLineId {
                creation_revision,
                creation_ordinal: ordinal_start + index,
            },
            content_len: line.content.len(),
            separator_len: line.separator.len(),
        })
        .collect()
}

fn parsed_line_ranges(source: &str, reaches_document_end: bool) -> Vec<crate::text::HardLine> {
    let mut lines = crate::text::hard_lines(source);
    if !reaches_document_end
        && lines
            .last()
            .is_some_and(|line| line.content.is_empty() && line.separator.is_empty())
    {
        lines.pop();
    }
    lines
}

fn reconcile_stable_ids(
    old_source: &str,
    old_lines: &[crate::text::HardLine],
    old_models: &[TextDocumentHardLineModel],
    new_source: &str,
    new_lines: &[crate::text::HardLine],
    new_models: &mut [TextDocumentHardLineModel],
) {
    let common_len = old_lines.len().min(new_lines.len());
    let mut prefix = 0usize;
    while prefix < common_len
        && line_source(old_source, &old_lines[prefix])
            == line_source(new_source, &new_lines[prefix])
    {
        new_models[prefix].id = old_models[prefix].id;
        prefix += 1;
    }

    let mut old_suffix = old_lines.len();
    let mut new_suffix = new_lines.len();
    while old_suffix > prefix
        && new_suffix > prefix
        && line_source(old_source, &old_lines[old_suffix - 1])
            == line_source(new_source, &new_lines[new_suffix - 1])
    {
        old_suffix -= 1;
        new_suffix -= 1;
        new_models[new_suffix].id = old_models[old_suffix].id;
    }

    if prefix < old_suffix && prefix < new_suffix {
        new_models[prefix].id = old_models[prefix].id;
    }
}

fn line_source<'a>(source: &'a str, line: &crate::text::HardLine) -> &'a str {
    &source[line.source_range()]
}
