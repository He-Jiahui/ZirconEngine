use std::mem::size_of;

use crate::core::framework::text::TextLayoutError;
use crate::text::{RichTextFormat, SharedTextLayoutSession, TextDocumentKey};

use super::super::rich_text::{UiParsedText, parse_source_text_with_provider};

pub(super) const RETAINED_PLAIN_DOCUMENT_CAPACITY: usize = 16;
pub(super) const RETAINED_PLAIN_DOCUMENT_MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RetainedPlainTextDocumentCacheReport {
    pub(super) entry_count: usize,
    pub(super) estimated_bytes: usize,
    pub(super) hit_count: u64,
    pub(super) miss_count: u64,
    pub(super) stale_source_alias_count: u64,
    pub(super) source_exact_compare_count: u64,
    pub(super) evicted_count: u64,
    pub(super) oversized_bypass_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct RetainedPlainTextDocument {
    key: TextDocumentKey,
    parsed: UiParsedText,
    estimated_bytes: usize,
    last_access: u64,
}

/// Bounded parsed-document owner for viewport layouts.
///
/// The document key narrows lookup, while exact Plain source equality remains the correctness
/// authority until the surface owns a pointer-stable source snapshot beside its revision.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct RetainedPlainTextDocumentCache {
    documents: Vec<RetainedPlainTextDocument>,
    estimated_bytes: usize,
    next_access: u64,
    report: RetainedPlainTextDocumentCacheReport,
}

impl Default for RetainedPlainTextDocumentCache {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            estimated_bytes: 0,
            next_access: 0,
            report: RetainedPlainTextDocumentCacheReport::default(),
        }
    }
}

impl RetainedPlainTextDocumentCache {
    pub(super) fn clear(&mut self) {
        self.documents.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    #[cfg(test)]
    pub(super) fn report(&self) -> RetainedPlainTextDocumentCacheReport {
        let mut report = self.report;
        report.entry_count = self.documents.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    pub(super) fn resolve(
        &mut self,
        key: TextDocumentKey,
        source: &str,
        provider: &SharedTextLayoutSession,
    ) -> Result<UiParsedText, TextLayoutError> {
        let access = self.next_access();
        if let Some(index) = self
            .documents
            .iter()
            .position(|document| document.key == key)
        {
            self.report.source_exact_compare_count =
                self.report.source_exact_compare_count.saturating_add(1);
            if self.documents[index].parsed.text() == source {
                self.report.hit_count = self.report.hit_count.saturating_add(1);
                self.documents[index].last_access = access;
                return Ok(self.documents[index].parsed.clone());
            }

            let stale = self.documents.swap_remove(index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(stale.estimated_bytes);
            self.report.stale_source_alias_count =
                self.report.stale_source_alias_count.saturating_add(1);
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let parsed = parse_source_text_with_provider(source, RichTextFormat::Plain, provider)?;
        let estimated_bytes = parsed
            .estimated_bytes()
            .saturating_add(size_of::<RetainedPlainTextDocument>());
        if estimated_bytes > RETAINED_PLAIN_DOCUMENT_MAX_BYTES {
            self.report.oversized_bypass_count =
                self.report.oversized_bypass_count.saturating_add(1);
            self.refresh_report_size();
            return Ok(parsed);
        }

        self.evict_for(estimated_bytes);
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.documents.push(RetainedPlainTextDocument {
            key,
            parsed: parsed.clone(),
            estimated_bytes,
            last_access: access,
        });
        self.refresh_report_size();
        Ok(parsed)
    }

    fn evict_for(&mut self, incoming_bytes: usize) {
        while self.documents.len() >= RETAINED_PLAIN_DOCUMENT_CAPACITY
            || self.estimated_bytes.saturating_add(incoming_bytes)
                > RETAINED_PLAIN_DOCUMENT_MAX_BYTES
        {
            let Some((oldest_index, _)) = self
                .documents
                .iter()
                .enumerate()
                .min_by_key(|(_, document)| document.last_access)
            else {
                break;
            };
            let removed = self.documents.swap_remove(oldest_index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(removed.estimated_bytes);
            self.report.evicted_count = self.report.evicted_count.saturating_add(1);
        }
    }

    fn next_access(&mut self) -> u64 {
        self.next_access = self.next_access.saturating_add(1);
        self.next_access
    }

    fn refresh_report_size(&mut self) {
        self.report.entry_count = self.documents.len();
        self.report.estimated_bytes = self.estimated_bytes;
    }
}
