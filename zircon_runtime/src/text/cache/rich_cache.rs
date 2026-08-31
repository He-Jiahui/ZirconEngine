use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::text::EphemeralCacheHash;
use crate::text::RichTextFormat;
use crate::text::rich::{
    CompiledRichText, RichTextContentTrust, RichTextParseError, RichTextParserGeneration,
};

use super::{IndexedTextCache, IndexedTextCacheEntry};

pub(crate) const DEFAULT_COMPILED_RICH_TEXT_CACHE_CAPACITY: usize = 256;
pub(crate) const DEFAULT_COMPILED_RICH_TEXT_CACHE_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledRichTextCacheReport {
    pub parser_identity: u64,
    pub decorator_generation: u64,
    pub emoji_generation: u64,
    pub compile_requests_in_flight: usize,
    pub single_flight_wait_count: u64,
    pub single_flight_wait_nanos: u64,
    pub single_flight_wait_max_nanos: u64,
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
    pub telemetry_saturated: bool,
}

#[derive(Clone, Copy)]
enum RichTextCacheCounter {
    Hit,
    Miss,
    Parse,
    Eviction,
    AdmissionBypass,
    CandidateProbe,
    SingleFlightWait,
    SingleFlightWaitNanos,
}

impl CompiledRichTextCacheReport {
    fn record(&mut self, counter: RichTextCacheCounter, amount: u64) {
        let value = match counter {
            RichTextCacheCounter::Hit => &mut self.hit_count,
            RichTextCacheCounter::Miss => &mut self.miss_count,
            RichTextCacheCounter::Parse => &mut self.parse_count,
            RichTextCacheCounter::Eviction => &mut self.eviction_count,
            RichTextCacheCounter::AdmissionBypass => &mut self.admission_bypass_count,
            RichTextCacheCounter::CandidateProbe => &mut self.candidate_probe_count,
            RichTextCacheCounter::SingleFlightWait => &mut self.single_flight_wait_count,
            RichTextCacheCounter::SingleFlightWaitNanos => &mut self.single_flight_wait_nanos,
        };
        let (next, saturated) = checked_interval_counter_add(*value, amount);
        *value = next;
        self.telemetry_saturated |= saturated;
    }

    fn reset_interval_counters(&mut self) {
        self.hit_count = 0;
        self.miss_count = 0;
        self.parse_count = 0;
        self.eviction_count = 0;
        self.admission_bypass_count = 0;
        self.candidate_probe_count = 0;
        self.single_flight_wait_count = 0;
        self.single_flight_wait_nanos = 0;
        self.single_flight_wait_max_nanos = 0;
        self.telemetry_saturated = false;
    }

    fn compile_request_started(&mut self) {
        let Some(next) = self.compile_requests_in_flight.checked_add(1) else {
            self.compile_requests_in_flight = usize::MAX;
            self.telemetry_saturated = true;
            return;
        };
        self.compile_requests_in_flight = next;
    }

    fn compile_request_finished(&mut self) {
        let Some(next) = self.compile_requests_in_flight.checked_sub(1) else {
            self.telemetry_saturated = true;
            return;
        };
        self.compile_requests_in_flight = next;
    }

    fn record_single_flight_wait(&mut self, elapsed_nanos: u128) {
        let elapsed_nanos = u64::try_from(elapsed_nanos).unwrap_or_else(|_| {
            self.telemetry_saturated = true;
            u64::MAX
        });
        self.record(RichTextCacheCounter::SingleFlightWait, 1);
        self.record(RichTextCacheCounter::SingleFlightWaitNanos, elapsed_nanos);
        self.single_flight_wait_max_nanos = self.single_flight_wait_max_nanos.max(elapsed_nanos);
    }

    pub(crate) fn with_generation(mut self, generation: RichTextParserGeneration) -> Self {
        self.parser_identity = generation.parser_identity;
        self.decorator_generation = generation.decorator_generation;
        self.emoji_generation = generation.emoji_generation;
        self
    }
}

fn checked_interval_counter_add(current: u64, amount: u64) -> (u64, bool) {
    current
        .checked_add(amount)
        .map_or((u64::MAX, true), |next| (next, false))
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RichTextArtifactKey {
    markup_hash: EphemeralCacheHash,
    markup_len: usize,
    format: RichTextFormat,
    content_trust: RichTextContentTrust,
    parser_identity: u64,
    decorator_generation: u64,
    emoji_generation: u64,
}

#[derive(Debug)]
struct RichTextArtifactCell {
    markup: Arc<str>,
    compiled: OnceLock<Result<Arc<CompiledRichText>, RichTextParseError>>,
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

    fn take_report(&mut self) -> CompiledRichTextCacheReport {
        let report = self.report;
        self.report.reset_interval_counters();
        report
    }

    fn key(
        &self,
        markup: &str,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
    ) -> RichTextArtifactKey {
        let mut hasher = self.hash_builder.build_hasher();
        markup.hash(&mut hasher);
        RichTextArtifactKey {
            markup_hash: EphemeralCacheHash::from_process_hash(hasher.finish()),
            markup_len: markup.len(),
            format,
            content_trust,
            parser_identity: generation.parser_identity,
            decorator_generation: generation.decorator_generation,
            emoji_generation: generation.emoji_generation,
        }
    }

    fn lookup_or_insert(
        &mut self,
        markup: &str,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
    ) -> Arc<RichTextArtifactCell> {
        let key = self.key(markup, format, content_trust, generation);
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.cell.markup.as_ref() == markup);
        self.report.record(
            RichTextCacheCounter::CandidateProbe,
            lookup.candidate_count as u64,
        );
        if let Some(slot) = lookup.slot {
            self.report.record(RichTextCacheCounter::Hit, 1);
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

        self.report.record(RichTextCacheCounter::Miss, 1);
        let cell = Arc::new(RichTextArtifactCell {
            markup: Arc::from(markup),
            compiled: OnceLock::new(),
        });
        let initial_bytes = cell.markup.len();
        if !self.reserve_for(1, initial_bytes) {
            self.report.record(RichTextCacheCounter::AdmissionBypass, 1);
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
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
    ) -> Option<Arc<CompiledRichText>> {
        let key = self.key(markup, format, content_trust, generation);
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.cell.markup.as_ref() == markup);
        self.report.record(
            RichTextCacheCounter::CandidateProbe,
            lookup.candidate_count as u64,
        );
        let Some(slot) = lookup.slot else {
            self.report.record(RichTextCacheCounter::Miss, 1);
            return None;
        };
        let entry = self.index.entry(slot).map(|entry| {
            (
                entry
                    .cell
                    .compiled
                    .get()
                    .and_then(|compiled| compiled.as_ref().ok())
                    .cloned(),
                entry.compiled_bytes_accounted,
            )
        });
        let compiled = entry.as_ref().and_then(|(compiled, _)| compiled.clone());
        if compiled.is_some() {
            self.report.record(RichTextCacheCounter::Hit, 1);
            if entry.is_some_and(|(_, compiled_bytes_accounted)| compiled_bytes_accounted) {
                self.index.touch(slot);
            }
        } else {
            self.report.record(RichTextCacheCounter::Miss, 1);
        }
        compiled
    }

    fn record_compiled(
        &mut self,
        cell: &Arc<RichTextArtifactCell>,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
        compiled_bytes: usize,
    ) {
        let key = self.key(&cell.markup, format, content_trust, generation);
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
            self.report.record(RichTextCacheCounter::AdmissionBypass, 1);
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
            self.report.record(RichTextCacheCounter::AdmissionBypass, 1);
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

    fn record_failed(
        &mut self,
        cell: &Arc<RichTextArtifactCell>,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
    ) {
        let key = self.key(&cell.markup, format, content_trust, generation);
        let Some(slot) = self
            .index
            .find_slot(&key, |entry| Arc::ptr_eq(&entry.cell, cell))
            .slot
        else {
            return;
        };
        if let Some(entry) = self.index.remove(slot) {
            self.report.resident_bytes = self
                .report
                .resident_bytes
                .saturating_sub(entry.resident_bytes);
        }
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
            self.report.record(RichTextCacheCounter::Eviction, 1);
            self.report.resident_entries = self.index.len();
        }
        self.report.resident_entries = self.index.len();
        true
    }
}

pub(crate) struct CompiledRichTextCacheOwner {
    cache: Mutex<CompiledRichTextCache>,
}

struct CompiledRichTextCompileRequest<'a> {
    owner: &'a CompiledRichTextCacheOwner,
}

impl Drop for CompiledRichTextCompileRequest<'_> {
    fn drop(&mut self) {
        self.owner.lock().report.compile_request_finished();
    }
}

impl Default for CompiledRichTextCacheOwner {
    fn default() -> Self {
        Self {
            cache: Mutex::new(CompiledRichTextCache::new()),
        }
    }
}

impl Debug for CompiledRichTextCacheOwner {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CompiledRichTextCacheOwner")
            .field("report", &self.report())
            .finish()
    }
}

impl CompiledRichTextCacheOwner {
    pub(crate) fn compile(
        &self,
        markup: &str,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
        compile: impl FnOnce(Arc<str>) -> Result<CompiledRichText, RichTextParseError>,
    ) -> Result<Arc<CompiledRichText>, RichTextParseError> {
        let cell = self
            .lock()
            .lookup_or_insert(markup, format, content_trust, generation);
        if let Some(compiled) = cell.compiled.get() {
            return compiled.clone();
        }
        let wait_started = Instant::now();
        let initialized_here = Cell::new(false);
        let _request = self.begin_compile_request();
        let compiled = cell
            .compiled
            .get_or_init(|| {
                initialized_here.set(true);
                {
                    let mut cache = self.lock();
                    cache.report.record(RichTextCacheCounter::Parse, 1);
                }
                match compile(Arc::clone(&cell.markup)) {
                    Ok(compiled) => {
                        let compiled = Arc::new(compiled);
                        self.lock().record_compiled(
                            &cell,
                            format,
                            content_trust,
                            generation,
                            compiled.estimated_bytes(),
                        );
                        Ok(compiled)
                    }
                    Err(error) => {
                        self.lock()
                            .record_failed(&cell, format, content_trust, generation);
                        Err(error)
                    }
                }
            })
            .clone();
        if !initialized_here.get() {
            self.lock()
                .report
                .record_single_flight_wait(wait_started.elapsed().as_nanos());
        }
        compiled
    }

    pub(crate) fn lookup(
        &self,
        markup: &str,
        format: RichTextFormat,
        content_trust: RichTextContentTrust,
        generation: RichTextParserGeneration,
    ) -> Option<Arc<CompiledRichText>> {
        self.lock()
            .lookup_compiled(markup, format, content_trust, generation)
    }

    pub(crate) fn report(&self) -> CompiledRichTextCacheReport {
        self.lock().report
    }

    pub(crate) fn take_report(&self) -> CompiledRichTextCacheReport {
        self.lock().take_report()
    }

    pub(crate) fn clear(&self) {
        *self.lock() = CompiledRichTextCache::new();
    }

    fn begin_compile_request(&self) -> CompiledRichTextCompileRequest<'_> {
        self.lock().report.compile_request_started();
        CompiledRichTextCompileRequest { owner: self }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, CompiledRichTextCache> {
        self.cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "rich_cache/tests.rs"]
mod tests;
