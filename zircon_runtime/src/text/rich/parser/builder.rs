use crate::text::{
    ParagraphOverride, RichParseResult, RichTable, RichTextAuthoringDiagnostic, StyledRun,
};

use super::super::admission::{RichParseBudget, RichTextContentTrust, RichTextParseError};
use super::super::decorator::retained_metadata_bytes;
use super::bidi_diagnostics::{BidiControlAdmission, BidiControlKind};

pub(super) struct RichParseBuilder {
    pub(super) text: String,
    pub(super) runs: Vec<StyledRun>,
    pub(super) paragraphs: Vec<((u32, u32), ParagraphOverride)>,
    pub(super) tables: Vec<RichTable>,
    pub(super) budget: RichParseBudget,
    pub(super) error: Option<RichTextParseError>,
    indexed_bytes: u32,
    consumed_tokens: usize,
    consumed_run_metadata_bytes: usize,
    bidi_control_admission: BidiControlAdmission,
    authoring_diagnostics: Vec<RichTextAuthoringDiagnostic>,
    authoring_diagnostics_truncated: bool,
}

impl RichParseBuilder {
    pub(super) fn new(budget: RichParseBudget, content_trust: RichTextContentTrust) -> Self {
        Self {
            text: String::new(),
            runs: Vec::new(),
            paragraphs: Vec::new(),
            tables: Vec::new(),
            budget,
            error: None,
            indexed_bytes: 0,
            consumed_tokens: 0,
            consumed_run_metadata_bytes: 0,
            bidi_control_admission: BidiControlAdmission::new(
                content_trust,
                budget.max_bidi_control_depth,
            ),
            authoring_diagnostics: Vec::new(),
            authoring_diagnostics_truncated: false,
        }
    }

    pub(super) fn finish(mut self) -> Result<RichParseResult, RichTextParseError> {
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        self.finish_bidi_control_admission()?;
        Ok(RichParseResult {
            text: self.text.into(),
            runs: self.runs,
            paragraphs: self.paragraphs,
            tables: self.tables,
            authoring_diagnostics: self.authoring_diagnostics,
            authoring_diagnostics_truncated: self.authoring_diagnostics_truncated,
        })
    }

    pub(super) fn push_authoring_diagnostic(&mut self, diagnostic: RichTextAuthoringDiagnostic) {
        if self.authoring_diagnostics.len() >= self.budget.max_authoring_diagnostics {
            self.authoring_diagnostics_truncated = true;
            return;
        }
        self.authoring_diagnostics.push(diagnostic);
    }

    pub(super) fn admit_bidi_control(
        &mut self,
        kind: BidiControlKind,
        source_range: (u32, u32),
    ) -> Result<(), RichTextParseError> {
        self.bidi_control_admission.observe(kind, source_range)
    }

    fn finish_bidi_control_admission(&self) -> Result<(), RichTextParseError> {
        self.bidi_control_admission.finish()
    }

    pub(super) fn admit_append(&mut self, appended_bytes: usize) -> Option<(u32, u32)> {
        if self.error.is_some() {
            return None;
        }
        let attempted_bytes = self
            .text
            .len()
            .checked_add(appended_bytes)
            .unwrap_or(usize::MAX);
        let max_bytes = self.budget.admitted_output_bytes();
        if attempted_bytes > max_bytes {
            self.error = Some(RichTextParseError::OutputByteBudgetExceeded {
                attempted_bytes,
                max_bytes,
            });
            return None;
        }
        let start = self.indexed_bytes;
        let end = attempted_bytes as u32;
        self.indexed_bytes = end;
        Some((start, end))
    }

    pub(super) fn admit_tokens(&mut self, count: usize) -> Result<(), RichTextParseError> {
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        let attempted_tokens = self
            .consumed_tokens
            .checked_add(count)
            .unwrap_or(usize::MAX);
        if attempted_tokens > self.budget.max_tokens {
            return Err(RichTextParseError::TokenBudgetExceeded {
                attempted_tokens,
                max_tokens: self.budget.max_tokens,
            });
        }
        self.consumed_tokens = attempted_tokens;
        Ok(())
    }

    pub(super) fn admit_active_paragraph_depth(&mut self, attempted_depth: usize) -> bool {
        if self.error.is_some() {
            return false;
        }
        if attempted_depth > self.budget.max_block_depth {
            self.error = Some(RichTextParseError::BlockDepthBudgetExceeded {
                attempted_depth,
                max_depth: self.budget.max_block_depth,
            });
            return false;
        }
        true
    }

    pub(super) fn admit_run(&mut self, run: &StyledRun) -> bool {
        if self.error.is_some() {
            return false;
        }
        let merges = self.runs.last().is_some_and(|previous| {
            previous.byte_range.1 == run.byte_range.0
                && previous.style == run.style
                && previous.inline == run.inline
                && previous.link == run.link
        });
        if !merges && self.runs.len() >= self.budget.max_runs {
            self.error = Some(RichTextParseError::RunCountBudgetExceeded {
                attempted_runs: self.runs.len().saturating_add(1),
                max_runs: self.budget.max_runs,
            });
            return false;
        }
        if !merges {
            let attempted_bytes = self
                .consumed_run_metadata_bytes
                .checked_add(retained_metadata_bytes(
                    &run.style,
                    run.inline.as_ref(),
                    run.link.as_ref(),
                ))
                .unwrap_or(usize::MAX);
            if attempted_bytes > self.budget.max_retained_run_metadata_bytes {
                self.error = Some(RichTextParseError::RunMetadataBudgetExceeded {
                    attempted_bytes,
                    max_bytes: self.budget.max_retained_run_metadata_bytes,
                });
                return false;
            }
            self.consumed_run_metadata_bytes = attempted_bytes;
        }
        true
    }

    pub(super) fn push_run(&mut self, run: StyledRun) {
        if let Some(previous) = self.runs.last_mut() {
            if previous.byte_range.1 == run.byte_range.0
                && previous.style == run.style
                && previous.inline == run.inline
                && previous.link == run.link
            {
                previous.byte_range.1 = run.byte_range.1;
                return;
            }
        }
        self.runs.push(run);
    }

    pub(super) fn push_paragraph(&mut self, range: (u32, u32), paragraph: ParagraphOverride) {
        if self.error.is_some() {
            return;
        }
        if self.paragraphs.len() >= self.budget.max_paragraphs {
            self.error = Some(RichTextParseError::ParagraphCountBudgetExceeded {
                attempted_paragraphs: self.paragraphs.len().saturating_add(1),
                max_paragraphs: self.budget.max_paragraphs,
            });
            return;
        }
        self.paragraphs.push((range, paragraph));
    }

    pub(super) fn push_table(&mut self, table: RichTable) {
        if self.error.is_some() {
            return;
        }
        if self.tables.len() >= self.budget.max_tables {
            self.error = Some(RichTextParseError::TableCountBudgetExceeded {
                attempted_tables: self.tables.len().saturating_add(1),
                max_tables: self.budget.max_tables,
            });
            return;
        }
        self.tables.push(table);
    }

    pub(super) const fn current_offset(&self) -> u32 {
        self.indexed_bytes
    }
}
