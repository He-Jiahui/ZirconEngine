use std::sync::Arc;

use crate::core::i18n::{EditorI18nService, EditorLocale};

use super::{search_document, EditorCommandPaletteEntry, EditorCommandPaletteSeed};

#[derive(Debug)]
pub(super) struct EditorCommandPaletteLocaleProjection {
    locale: EditorLocale,
    pub(super) entries: Arc<[EditorCommandPaletteEntry]>,
    pub(super) search_documents: Arc<[Box<str>]>,
    pub(super) search_postings: Box<[Box<[usize]>; 256]>,
}

impl EditorCommandPaletteLocaleProjection {
    pub(super) fn build(
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        seeds: &[EditorCommandPaletteSeed],
    ) -> Self {
        let mut entries = Vec::with_capacity(seeds.len());
        let mut search_documents = Vec::with_capacity(seeds.len());
        let mut posting_counts = [0usize; 256];
        for seed in seeds {
            let entry = seed.project(i18n, locale);
            let document = search_document(&entry);
            let mut indexed_bytes = [false; 256];
            for byte in document.bytes() {
                indexed_bytes[usize::from(byte)] = true;
            }
            for (byte, present) in indexed_bytes.into_iter().enumerate() {
                if present {
                    posting_counts[byte] += 1;
                }
            }
            entries.push(entry);
            search_documents.push(document);
        }
        let mut search_postings = posting_counts.map(Vec::with_capacity);
        for (index, document) in search_documents.iter().enumerate() {
            let mut indexed_bytes = [false; 256];
            for byte in document.bytes() {
                indexed_bytes[usize::from(byte)] = true;
            }
            for (byte, present) in indexed_bytes.into_iter().enumerate() {
                if present {
                    search_postings[byte].push(index);
                }
            }
        }
        Self {
            locale: locale.clone(),
            entries: entries.into(),
            search_documents: search_documents.into(),
            search_postings: Box::new(search_postings.map(Vec::into_boxed_slice)),
        }
    }

    pub(super) fn matches(&self, locale: &EditorLocale) -> bool {
        &self.locale == locale
    }
}

#[cfg(test)]
#[path = "tests/locale_projection.rs"]
mod tests;
