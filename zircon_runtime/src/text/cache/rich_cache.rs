use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

use crate::text::rich::{CompiledRichText, RichTextParserGeneration};
use crate::text::RichTextFormat;

use super::{IndexedTextCache, IndexedTextCacheEntry};

pub(crate) const DEFAULT_COMPILED_RICH_TEXT_CACHE_CAPACITY: usize = 256;
pub(crate) const DEFAULT_COMPILED_RICH_TEXT_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledRichTextCacheReport {
    pub hit_count: u64,
    pub miss_count: u64,
    pub parse_count: u64,
    pub eviction_count: u64,
    pub candidate_probe_count: u64,
    pub resident_entries: usize,
    pub resident_bytes: usize,
    pub max_entries: usize,
    pub max_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledRichTextCacheFrameSampler {
    last_hit_count: u64,
    last_miss_count: u64,
    last_parse_count: u64,
    last_eviction_count: u64,
    last_candidate_probe_count: u64,
}

impl CompiledRichTextCacheFrameSampler {
    pub fn from_shared_cache() -> Self {
        Self::from_report(shared_compiled_rich_text_cache_report())
    }

    pub fn sample(&mut self) -> CompiledRichTextCacheReport {
        self.sample_report(shared_compiled_rich_text_cache_report())
    }

    fn from_report(report: CompiledRichTextCacheReport) -> Self {
        Self {
            last_hit_count: report.hit_count,
            last_miss_count: report.miss_count,
            last_parse_count: report.parse_count,
            last_eviction_count: report.eviction_count,
            last_candidate_probe_count: report.candidate_probe_count,
        }
    }

    fn sample_report(
        &mut self,
        cumulative: CompiledRichTextCacheReport,
    ) -> CompiledRichTextCacheReport {
        let frame = CompiledRichTextCacheReport {
            hit_count: cumulative.hit_count.saturating_sub(self.last_hit_count),
            miss_count: cumulative.miss_count.saturating_sub(self.last_miss_count),
            parse_count: cumulative.parse_count.saturating_sub(self.last_parse_count),
            eviction_count: cumulative
                .eviction_count
                .saturating_sub(self.last_eviction_count),
            candidate_probe_count: cumulative
                .candidate_probe_count
                .saturating_sub(self.last_candidate_probe_count),
            resident_entries: cumulative.resident_entries,
            resident_bytes: cumulative.resident_bytes,
            max_entries: cumulative.max_entries,
            max_bytes: cumulative.max_bytes,
        };
        *self = Self::from_report(cumulative);
        frame
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichTextArtifactKey {
    markup_hash: u64,
    markup_len: usize,
    format: u8,
    parser_identity: u64,
    decorator_generation: u64,
    emoji_generation: u64,
}

#[derive(Debug)]
struct RichTextArtifactCell {
    markup: Arc<str>,
    compiled: OnceLock<Arc<CompiledRichText>>,
}

#[derive(Debug)]
struct RichTextArtifactEntry {
    key: RichTextArtifactKey,
    cell: Arc<RichTextArtifactCell>,
    resident_bytes: usize,
}

impl IndexedTextCacheEntry<RichTextArtifactKey> for RichTextArtifactEntry {
    fn cache_key(&self) -> &RichTextArtifactKey {
        &self.key
    }
}

struct CompiledRichTextCache {
    index: IndexedTextCache<RichTextArtifactKey, RichTextArtifactEntry>,
    hash_builder: RandomState,
    report: CompiledRichTextCacheReport,
}

impl CompiledRichTextCache {
    fn new() -> Self {
        Self {
            index: IndexedTextCache::new(),
            hash_builder: RandomState::new(),
            report: CompiledRichTextCacheReport {
                max_entries: DEFAULT_COMPILED_RICH_TEXT_CACHE_CAPACITY,
                max_bytes: DEFAULT_COMPILED_RICH_TEXT_CACHE_MAX_BYTES,
                ..CompiledRichTextCacheReport::default()
            },
        }
    }

    fn key(
        &self,
        markup: &str,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
    ) -> RichTextArtifactKey {
        let mut hasher = self.hash_builder.build_hasher();
        markup.hash(&mut hasher);
        RichTextArtifactKey {
            markup_hash: hasher.finish(),
            markup_len: markup.len(),
            format: rich_text_format_id(format),
            parser_identity: generation.parser_identity,
            decorator_generation: generation.decorator_generation,
            emoji_generation: generation.emoji_generation,
        }
    }

    fn lookup_or_insert(
        &mut self,
        markup: &str,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
    ) -> Arc<RichTextArtifactCell> {
        let key = self.key(markup, format, generation);
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.cell.markup.as_ref() == markup);
        self.report.candidate_probe_count = self
            .report
            .candidate_probe_count
            .saturating_add(lookup.candidate_count as u64);
        if let Some(slot) = lookup.slot {
            self.report.hit_count = self.report.hit_count.saturating_add(1);
            let cell = self.index.entry(slot).map(|entry| Arc::clone(&entry.cell));
            if cell
                .as_ref()
                .is_some_and(|cell| cell.compiled.get().is_some())
            {
                self.index.touch(slot);
            }
            if let Some(cell) = cell {
                return cell;
            }
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let cell = Arc::new(RichTextArtifactCell {
            markup: Arc::from(markup),
            compiled: OnceLock::new(),
        });
        let initial_bytes = cell.markup.len();
        self.index.update_or_insert_with(
            None,
            RichTextArtifactEntry {
                key,
                cell: Arc::clone(&cell),
                resident_bytes: initial_bytes,
            },
            false,
            |_, _| unreachable!("a fresh rich-text cell cannot update an existing slot"),
            |entry| entry,
        );
        self.report.resident_bytes = self.report.resident_bytes.saturating_add(initial_bytes);
        self.report.resident_entries = self.index.len();
        cell
    }

    fn lookup_compiled(
        &mut self,
        markup: &str,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
    ) -> Option<Arc<CompiledRichText>> {
        let key = self.key(markup, format, generation);
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.cell.markup.as_ref() == markup);
        self.report.candidate_probe_count = self
            .report
            .candidate_probe_count
            .saturating_add(lookup.candidate_count as u64);
        let Some(slot) = lookup.slot else {
            self.report.miss_count = self.report.miss_count.saturating_add(1);
            return None;
        };
        let compiled = self
            .index
            .entry(slot)
            .and_then(|entry| entry.cell.compiled.get())
            .cloned();
        if compiled.is_some() {
            self.report.hit_count = self.report.hit_count.saturating_add(1);
            self.index.touch(slot);
        } else {
            self.report.miss_count = self.report.miss_count.saturating_add(1);
        }
        compiled
    }

    fn record_compiled(&mut self, cell: &Arc<RichTextArtifactCell>, compiled_bytes: usize) {
        let generation = cell
            .compiled
            .get()
            .map(|compiled| compiled.generation())
            .unwrap_or_default();
        let format = cell
            .compiled
            .get()
            .map(|compiled| compiled.format())
            .unwrap_or(RichTextFormat::Plain);
        let key = self.key(&cell.markup, format, generation);
        let Some(slot) = self
            .index
            .find_slot(&key, |entry| Arc::ptr_eq(&entry.cell, cell))
            .slot
        else {
            return;
        };
        let Some(entry) = self.index.entry_mut(slot) else {
            return;
        };
        self.report.resident_bytes = self
            .report
            .resident_bytes
            .saturating_sub(entry.resident_bytes)
            .saturating_add(compiled_bytes);
        entry.resident_bytes = compiled_bytes;
        self.index.touch(slot);
        self.enforce_budget();
    }

    fn enforce_budget(&mut self) {
        while self.index.len() > self.report.max_entries
            || self.report.resident_bytes > self.report.max_bytes
        {
            let Some(entry) = self.index.pop_oldest() else {
                break;
            };
            self.report.resident_bytes = self
                .report
                .resident_bytes
                .saturating_sub(entry.resident_bytes);
            self.report.eviction_count = self.report.eviction_count.saturating_add(1);
        }
        self.report.resident_entries = self.index.len();
    }
}

pub(crate) fn cached_compiled_rich_text(
    markup: &str,
    format: RichTextFormat,
    generation: RichTextParserGeneration,
    compile: impl FnOnce(Arc<str>) -> CompiledRichText,
) -> Arc<CompiledRichText> {
    let cell = {
        let mut cache = lock_cache();
        cache.lookup_or_insert(markup, format, generation)
    };
    let mut compiled_here = false;
    let compiled = Arc::clone(cell.compiled.get_or_init(|| {
        compiled_here = true;
        {
            let mut cache = lock_cache();
            cache.report.parse_count = cache.report.parse_count.saturating_add(1);
        }
        Arc::new(compile(Arc::clone(&cell.markup)))
    }));
    if compiled_here {
        let mut cache = lock_cache();
        cache.record_compiled(&cell, compiled.estimated_bytes());
    }
    compiled
}

pub(crate) fn lookup_cached_compiled_rich_text(
    markup: &str,
    format: RichTextFormat,
    generation: RichTextParserGeneration,
) -> Option<Arc<CompiledRichText>> {
    lock_cache().lookup_compiled(markup, format, generation)
}

pub fn shared_compiled_rich_text_cache_report() -> CompiledRichTextCacheReport {
    lock_cache().report
}

fn lock_cache() -> std::sync::MutexGuard<'static, CompiledRichTextCache> {
    shared_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn shared_cache() -> &'static Mutex<CompiledRichTextCache> {
    static CACHE: OnceLock<Mutex<CompiledRichTextCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(CompiledRichTextCache::new()))
}

const fn rich_text_format_id(format: RichTextFormat) -> u8 {
    match format {
        RichTextFormat::Plain => 0,
        RichTextFormat::Markdown => 1,
        RichTextFormat::BbCode => 2,
        RichTextFormat::Html => 3,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::text::rich::{CompiledRichText, RichTextParserGeneration};
    use crate::text::{RichParseResult, RichTextFormat};

    use super::{
        CompiledRichTextCache, CompiledRichTextCacheFrameSampler, CompiledRichTextCacheReport,
    };

    #[test]
    fn compiled_rich_cache_reuses_exact_artifact_and_bounds_residency() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_entries = 2;
        let generation = RichTextParserGeneration {
            parser_identity: 7,
            decorator_generation: 1,
            emoji_generation: 1,
        };
        let first = cache.lookup_or_insert("[b]one[/b]", RichTextFormat::BbCode, generation);
        let repeated = cache.lookup_or_insert("[b]one[/b]", RichTextFormat::BbCode, generation);
        let second = cache.lookup_or_insert("[b]two[/b]", RichTextFormat::BbCode, generation);
        let third = cache.lookup_or_insert("[b]three[/b]", RichTextFormat::BbCode, generation);

        assert!(Arc::ptr_eq(&first, &repeated));
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(!Arc::ptr_eq(&second, &third));
        assert_eq!(cache.report.hit_count, 1);
        assert_eq!(cache.report.miss_count, 3);
        assert_eq!(cache.report.eviction_count, 1);
        assert_eq!(cache.report.resident_entries, 2);
    }

    #[test]
    fn compiled_rich_cache_key_includes_format_and_registry_generations() {
        let mut cache = CompiledRichTextCache::new();
        let base = RichTextParserGeneration {
            parser_identity: 9,
            decorator_generation: 1,
            emoji_generation: 1,
        };
        let bbcode = cache.lookup_or_insert("same", RichTextFormat::BbCode, base);
        let html = cache.lookup_or_insert("same", RichTextFormat::Html, base);
        let decorated = cache.lookup_or_insert(
            "same",
            RichTextFormat::BbCode,
            RichTextParserGeneration {
                decorator_generation: 2,
                ..base
            },
        );

        assert!(!Arc::ptr_eq(&bbcode, &html));
        assert!(!Arc::ptr_eq(&bbcode, &decorated));
    }

    #[test]
    fn compiled_rich_cache_does_not_evict_an_in_flight_single_flight_cell() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_entries = 1;
        let generation = RichTextParserGeneration::default();

        let first = cache.lookup_or_insert("first", RichTextFormat::Plain, generation);
        let _second = cache.lookup_or_insert("second", RichTextFormat::Plain, generation);
        let repeated = cache.lookup_or_insert("first", RichTextFormat::Plain, generation);

        assert!(Arc::ptr_eq(&first, &repeated));
    }

    #[test]
    fn compiled_artifact_accounts_for_source_and_visible_text() {
        let compiled = CompiledRichText::new(
            Arc::from("[b]text[/b]"),
            RichTextFormat::BbCode,
            RichTextParserGeneration::default(),
            RichParseResult {
                text: "text".into(),
                ..RichParseResult::default()
            },
        );

        assert!(compiled.estimated_bytes() >= "[b]text[/b]".len() + "text".len());
    }

    #[test]
    fn compiled_rich_cache_frame_sampler_reports_deltas_without_global_reset() {
        let baseline = CompiledRichTextCacheReport {
            hit_count: 10,
            miss_count: 4,
            parse_count: 4,
            eviction_count: 1,
            candidate_probe_count: 12,
            resident_entries: 3,
            resident_bytes: 128,
            max_entries: 8,
            max_bytes: 1024,
        };
        let mut sampler = CompiledRichTextCacheFrameSampler::from_report(baseline);
        let frame = sampler.sample_report(CompiledRichTextCacheReport {
            hit_count: 13,
            miss_count: 5,
            parse_count: 5,
            eviction_count: 2,
            candidate_probe_count: 17,
            resident_entries: 3,
            resident_bytes: 160,
            ..baseline
        });

        assert_eq!(frame.hit_count, 3);
        assert_eq!(frame.miss_count, 1);
        assert_eq!(frame.parse_count, 1);
        assert_eq!(frame.eviction_count, 1);
        assert_eq!(frame.candidate_probe_count, 5);
        assert_eq!(frame.resident_entries, 3);
        assert_eq!(frame.resident_bytes, 160);
        assert_eq!(sampler.sample_report(baseline).hit_count, 0);
    }
}
