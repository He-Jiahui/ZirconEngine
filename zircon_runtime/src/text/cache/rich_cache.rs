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
    pub admission_bypass_count: u64,
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
    last_admission_bypass_count: u64,
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
            last_admission_bypass_count: report.admission_bypass_count,
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
            admission_bypass_count: cumulative
                .admission_bypass_count
                .saturating_sub(self.last_admission_bypass_count),
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
    compiled_bytes_accounted: bool,
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
    completed_resident_entries: usize,
    completed_resident_bytes: usize,
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
            completed_resident_entries: 0,
            completed_resident_bytes: 0,
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
            let entry = self
                .index
                .entry(slot)
                .map(|entry| (Arc::clone(&entry.cell), entry.compiled_bytes_accounted));
            if entry
                .as_ref()
                .is_some_and(|(_, compiled_bytes_accounted)| *compiled_bytes_accounted)
            {
                self.index.touch(slot);
            }
            if let Some((cell, _)) = entry {
                return cell;
            }
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let cell = Arc::new(RichTextArtifactCell {
            markup: Arc::from(markup),
            compiled: OnceLock::new(),
        });
        let initial_bytes = cell.markup.len();
        if !self.reserve_for(1, initial_bytes) {
            self.report.admission_bypass_count =
                self.report.admission_bypass_count.saturating_add(1);
            return cell;
        }
        self.index.insert_untracked(RichTextArtifactEntry {
            key,
            cell: Arc::clone(&cell),
            resident_bytes: initial_bytes,
            compiled_bytes_accounted: false,
        });
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
        let entry = self.index.entry(slot).map(|entry| {
            (
                entry.cell.compiled.get().cloned(),
                entry.compiled_bytes_accounted,
            )
        });
        let compiled = entry.as_ref().and_then(|(compiled, _)| compiled.clone());
        if compiled.is_some() {
            self.report.hit_count = self.report.hit_count.saturating_add(1);
            if entry.is_some_and(|(_, compiled_bytes_accounted)| compiled_bytes_accounted) {
                self.index.touch(slot);
            }
        } else {
            self.report.miss_count = self.report.miss_count.saturating_add(1);
        }
        compiled
    }

    fn record_compiled(
        &mut self,
        cell: &Arc<RichTextArtifactCell>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
        compiled_bytes: usize,
    ) {
        let key = self.key(&cell.markup, format, generation);
        let Some(slot) = self
            .index
            .find_slot(&key, |entry| Arc::ptr_eq(&entry.cell, cell))
            .slot
        else {
            return;
        };
        let Some((resident_bytes, compiled_bytes_accounted)) = self
            .index
            .entry(slot)
            .map(|entry| (entry.resident_bytes, entry.compiled_bytes_accounted))
        else {
            return;
        };
        if compiled_bytes_accounted {
            self.index.touch(slot);
            return;
        }
        if compiled_bytes > self.report.max_bytes {
            if let Some(entry) = self.index.remove(slot) {
                self.report.resident_bytes = self
                    .report
                    .resident_bytes
                    .saturating_sub(entry.resident_bytes);
            }
            self.report.resident_entries = self.index.len();
            self.report.admission_bypass_count =
                self.report.admission_bypass_count.saturating_add(1);
            return;
        }
        let additional_bytes = compiled_bytes.saturating_sub(resident_bytes);
        if !self.reserve_for(0, additional_bytes) {
            if let Some(entry) = self.index.remove(slot) {
                self.report.resident_bytes = self
                    .report
                    .resident_bytes
                    .saturating_sub(entry.resident_bytes);
            }
            self.report.resident_entries = self.index.len();
            self.report.admission_bypass_count =
                self.report.admission_bypass_count.saturating_add(1);
            return;
        }
        let Some(entry) = self.index.entry_mut(slot) else {
            return;
        };
        self.report.resident_bytes = self
            .report
            .resident_bytes
            .saturating_sub(entry.resident_bytes)
            .saturating_add(compiled_bytes);
        entry.resident_bytes = compiled_bytes;
        entry.compiled_bytes_accounted = true;
        self.completed_resident_entries = self.completed_resident_entries.saturating_add(1);
        self.completed_resident_bytes =
            self.completed_resident_bytes.saturating_add(compiled_bytes);
        self.index.touch(slot);
        self.report.resident_entries = self.index.len();
    }

    fn reserve_for(&mut self, additional_entries: usize, additional_bytes: usize) -> bool {
        if additional_entries > self.report.max_entries || additional_bytes > self.report.max_bytes
        {
            return false;
        }
        let required_entry_reclamation = self
            .index
            .len()
            .saturating_add(additional_entries)
            .saturating_sub(self.report.max_entries);
        let required_byte_reclamation = self
            .report
            .resident_bytes
            .saturating_add(additional_bytes)
            .saturating_sub(self.report.max_bytes);
        if required_entry_reclamation > self.completed_resident_entries
            || required_byte_reclamation > self.completed_resident_bytes
        {
            return false;
        }
        while self.index.len().saturating_add(additional_entries) > self.report.max_entries
            || self.report.resident_bytes.saturating_add(additional_bytes) > self.report.max_bytes
        {
            let Some(entry) = self.index.pop_oldest() else {
                self.report.resident_entries = self.index.len();
                return false;
            };
            if entry.compiled_bytes_accounted {
                self.completed_resident_entries = self.completed_resident_entries.saturating_sub(1);
                self.completed_resident_bytes = self
                    .completed_resident_bytes
                    .saturating_sub(entry.resident_bytes);
            }
            self.report.resident_bytes = self
                .report
                .resident_bytes
                .saturating_sub(entry.resident_bytes);
            self.report.eviction_count = self.report.eviction_count.saturating_add(1);
            self.report.resident_entries = self.index.len();
        }
        self.report.resident_entries = self.index.len();
        true
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
    Arc::clone(cell.compiled.get_or_init(|| {
        {
            let mut cache = lock_cache();
            cache.report.parse_count = cache.report.parse_count.saturating_add(1);
        }
        let compiled = Arc::new(compile(Arc::clone(&cell.markup)));
        lock_cache().record_compiled(&cell, format, generation, compiled.estimated_bytes());
        compiled
    }))
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
        record_compiled_cell(&mut cache, &first, RichTextFormat::BbCode, generation);
        let repeated = cache.lookup_or_insert("[b]one[/b]", RichTextFormat::BbCode, generation);
        let second = cache.lookup_or_insert("[b]two[/b]", RichTextFormat::BbCode, generation);
        record_compiled_cell(&mut cache, &second, RichTextFormat::BbCode, generation);
        let third = cache.lookup_or_insert("[b]three[/b]", RichTextFormat::BbCode, generation);
        record_compiled_cell(&mut cache, &third, RichTextFormat::BbCode, generation);

        assert!(Arc::ptr_eq(&first, &repeated));
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(!Arc::ptr_eq(&second, &third));
        assert_eq!(cache.report.hit_count, 1);
        assert_eq!(cache.report.miss_count, 3);
        assert_eq!(cache.report.eviction_count, 1);
        assert_eq!(cache.report.resident_entries, 2);
    }

    fn record_compiled_cell(
        cache: &mut CompiledRichTextCache,
        cell: &Arc<super::RichTextArtifactCell>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
    ) {
        let compiled = compiled_cell_artifact(cell, format, generation);
        cache.record_compiled(cell, format, generation, compiled.estimated_bytes());
        assert!(cell.compiled.set(compiled).is_ok());
    }

    fn compiled_cell_artifact(
        cell: &Arc<super::RichTextArtifactCell>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
    ) -> Arc<CompiledRichText> {
        compiled_cell_artifact_with_text(cell, format, generation, cell.markup.to_string())
    }

    fn compiled_cell_artifact_with_text(
        cell: &Arc<super::RichTextArtifactCell>,
        format: RichTextFormat,
        generation: RichTextParserGeneration,
        text: String,
    ) -> Arc<CompiledRichText> {
        Arc::new(CompiledRichText::new(
            Arc::clone(&cell.markup),
            format,
            generation,
            RichParseResult {
                text: text.into(),
                ..RichParseResult::default()
            },
        ))
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
    fn compiled_rich_cache_bypasses_a_second_in_flight_cell_when_budget_is_full() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_entries = 1;
        let generation = RichTextParserGeneration::default();

        let first = cache.lookup_or_insert("first", RichTextFormat::Plain, generation);
        let second = cache.lookup_or_insert("second", RichTextFormat::Plain, generation);
        let repeated = cache.lookup_or_insert("first", RichTextFormat::Plain, generation);

        assert!(Arc::ptr_eq(&first, &repeated));
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(cache.report.resident_entries, 1);
        assert_eq!(cache.report.resident_bytes, first.markup.len());
        assert_eq!(cache.report.admission_bypass_count, 1);
    }

    #[test]
    fn compiled_rich_cache_discards_a_completed_cell_that_exceeds_its_byte_budget() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_bytes = 1;
        let generation = RichTextParserGeneration::default();

        let cell = cache.lookup_or_insert("x", RichTextFormat::Plain, generation);
        let compiled = compiled_cell_artifact(&cell, RichTextFormat::Plain, generation);
        cache.record_compiled(
            &cell,
            RichTextFormat::Plain,
            generation,
            compiled.estimated_bytes(),
        );
        assert!(cell.compiled.get().is_none());
        assert_eq!(cache.report.resident_entries, 0);
        assert_eq!(cache.report.resident_bytes, 0);
        assert!(cell.compiled.set(compiled).is_ok());

        assert!(cell.compiled.get().is_some());
        assert_eq!(cache.report.eviction_count, 0);
        assert_eq!(cache.report.admission_bypass_count, 1);
    }

    #[test]
    fn compiled_rich_cache_bypasses_markup_that_exceeds_its_byte_budget() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_bytes = 3;

        let cell = cache.lookup_or_insert(
            "four",
            RichTextFormat::Plain,
            RichTextParserGeneration::default(),
        );

        assert_eq!(cell.markup.as_ref(), "four");
        assert_eq!(cache.report.resident_entries, 0);
        assert_eq!(cache.report.resident_bytes, 0);
        assert_eq!(cache.report.admission_bypass_count, 1);
    }

    #[test]
    fn oversized_completion_does_not_evict_a_healthy_compiled_entry() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_entries = 2;
        let generation = RichTextParserGeneration::default();
        let retained = cache.lookup_or_insert("retained", RichTextFormat::Plain, generation);
        record_compiled_cell(&mut cache, &retained, RichTextFormat::Plain, generation);
        let retained_bytes = cache.report.resident_bytes;
        cache.report.max_bytes = retained_bytes.saturating_add(1);

        let oversized = cache.lookup_or_insert("x", RichTextFormat::Plain, generation);
        let compiled = compiled_cell_artifact_with_text(
            &oversized,
            RichTextFormat::Plain,
            generation,
            "expanded".repeat(retained_bytes.saturating_add(1)),
        );
        let eviction_count = cache.report.eviction_count;
        cache.record_compiled(
            &oversized,
            RichTextFormat::Plain,
            generation,
            compiled.estimated_bytes(),
        );
        assert!(oversized.compiled.set(compiled).is_ok());

        let repeated = cache.lookup_or_insert("retained", RichTextFormat::Plain, generation);
        assert!(Arc::ptr_eq(&retained, &repeated));
        assert_eq!(cache.report.eviction_count, eviction_count);
        assert_eq!(cache.report.resident_entries, 1);
        assert_eq!(cache.report.resident_bytes, retained_bytes);
        assert_eq!(cache.report.admission_bypass_count, 1);
    }

    #[test]
    fn failed_pending_admission_does_not_partially_evict_completed_entries() {
        let mut cache = CompiledRichTextCache::new();
        cache.report.max_entries = 3;
        let generation = RichTextParserGeneration::default();
        let retained = cache.lookup_or_insert("retained", RichTextFormat::Plain, generation);
        record_compiled_cell(&mut cache, &retained, RichTextFormat::Plain, generation);
        let retained_bytes = cache.report.resident_bytes;
        cache.report.max_bytes = retained_bytes.saturating_add(8);

        let pending = cache.lookup_or_insert("12345678", RichTextFormat::Plain, generation);
        let eviction_count = cache.report.eviction_count;
        let oversized_markup = "x".repeat(cache.report.max_bytes);
        let bypassed = cache.lookup_or_insert(&oversized_markup, RichTextFormat::Plain, generation);

        assert!(!Arc::ptr_eq(&pending, &bypassed));
        assert_eq!(cache.report.eviction_count, eviction_count);
        assert_eq!(cache.report.resident_entries, 2);
        assert_eq!(cache.report.resident_bytes, cache.report.max_bytes);
        assert_eq!(cache.report.admission_bypass_count, 1);
        let repeated = cache.lookup_or_insert("retained", RichTextFormat::Plain, generation);
        assert!(Arc::ptr_eq(&retained, &repeated));
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
            admission_bypass_count: 2,
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
            admission_bypass_count: 5,
            candidate_probe_count: 17,
            resident_entries: 3,
            resident_bytes: 160,
            ..baseline
        });

        assert_eq!(frame.hit_count, 3);
        assert_eq!(frame.miss_count, 1);
        assert_eq!(frame.parse_count, 1);
        assert_eq!(frame.eviction_count, 1);
        assert_eq!(frame.admission_bypass_count, 3);
        assert_eq!(frame.candidate_probe_count, 5);
        assert_eq!(frame.resident_entries, 3);
        assert_eq!(frame.resident_bytes, 160);
        assert_eq!(sampler.sample_report(baseline).hit_count, 0);
    }
}
