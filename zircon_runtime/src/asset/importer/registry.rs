use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

use thiserror::Error;

use super::{
    AssetImporterCapabilityReport, AssetImporterCapabilityStatus, AssetImporterDescriptor,
    AssetImporterHandler, normalize_extension, normalize_full_suffix,
};
use crate::asset::AssetImportError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AssetImporterRegistryError {
    #[error("asset importer {0} already registered")]
    DuplicateImporterId(String),
    #[error("duplicate importer matcher {matcher} at priority {priority}")]
    DuplicateMatcher { matcher: String, priority: i32 },
    #[error(
        "asset importer {importer_id} cannot register deprecated UI document suffix {suffix}; UI documents must use .zui"
    )]
    DeprecatedUiDocumentSuffixImporter { importer_id: String, suffix: String },
    #[error("asset importer {0} must declare at least one source extension or full suffix")]
    MissingMatcher(String),
}

#[derive(Clone, Default)]
pub struct AssetImporterRegistry {
    // Clones are immutable published views; mutations publish a COW successor so
    // project/plugin readers never observe a partially-updated matcher index.
    generation: Arc<AssetImporterRegistryGeneration>,
}

impl AssetImporterRegistry {
    pub fn register(
        &mut self,
        importer: impl AssetImporterHandler + 'static,
    ) -> Result<(), AssetImporterRegistryError> {
        self.register_arc(Arc::new(importer))
    }

    pub fn register_arc(
        &mut self,
        importer: Arc<dyn AssetImporterHandler>,
    ) -> Result<(), AssetImporterRegistryError> {
        let prepared = PreparedImporter::new(importer)?;
        let generation = self.generation.as_ref();
        if generation.id_to_slot.contains_key(&prepared.id) {
            return Err(AssetImporterRegistryError::DuplicateImporterId(prepared.id));
        }
        for extension in &prepared.extensions {
            if generation.has_matcher_at_priority(
                &generation.extension_to_slots,
                extension,
                prepared.priority,
            ) {
                return Err(AssetImporterRegistryError::DuplicateMatcher {
                    matcher: format!("ext:{extension}"),
                    priority: prepared.priority,
                });
            }
        }
        for suffix in &prepared.full_suffixes {
            if generation.has_matcher_at_priority(
                &generation.full_suffix_to_slots,
                suffix,
                prepared.priority,
            ) {
                return Err(AssetImporterRegistryError::DuplicateMatcher {
                    matcher: format!("suffix:{suffix}"),
                    priority: prepared.priority,
                });
            }
        }

        Arc::make_mut(&mut self.generation).insert(prepared);
        Ok(())
    }

    pub fn select(
        &self,
        source_path: &Path,
    ) -> Result<Arc<dyn AssetImporterHandler>, AssetImportError> {
        self.select_slot(source_path)
            .map(|slot| slot.importer.clone())
    }

    fn select_slot(&self, source_path: &Path) -> Result<&AssetImporterSlot, AssetImportError> {
        if let Some(importer) = self.best_full_suffix_match(source_path) {
            return Ok(importer);
        }
        if let Some(suffix) = unknown_typed_toml_suffix(source_path) {
            return Err(AssetImportError::UnsupportedFormat(format!(
                "typed toml asset suffix `{suffix}.toml` has no registered importer"
            )));
        }
        if let Some(importer) = self.best_extension_match(source_path) {
            return Ok(importer);
        }
        Err(AssetImportError::UnsupportedFormat(format!(
            "no asset importer registered for {}",
            source_path.display()
        )))
    }

    pub fn descriptor_for_source(
        &self,
        source_path: &Path,
    ) -> Result<AssetImporterDescriptor, AssetImportError> {
        self.select_slot(source_path)
            .map(|slot| slot.importer.descriptor().clone())
    }

    pub fn capability_report_for_source(
        &self,
        source_path: &Path,
    ) -> Result<AssetImporterCapabilityReport, AssetImportError> {
        self.select_slot(source_path)
            .map(|slot| AssetImporterCapabilityReport {
                descriptor: slot.importer.descriptor().clone(),
                status: slot.availability.status.clone(),
            })
    }

    pub fn capability_reports(&self) -> Vec<AssetImporterCapabilityReport> {
        self.generation
            .slots()
            .map(|slot| AssetImporterCapabilityReport {
                descriptor: slot.importer.descriptor().clone(),
                status: slot.availability.status.clone(),
            })
            .collect()
    }

    pub fn descriptors(&self) -> Vec<AssetImporterDescriptor> {
        self.generation
            .slots()
            .map(|slot| slot.importer.descriptor().clone())
            .collect()
    }

    pub fn descriptors_for_plugin(&self, plugin_id: &str) -> Vec<AssetImporterDescriptor> {
        self.generation
            .plugin_to_slots
            .get(plugin_id)
            .into_iter()
            .flatten()
            .filter_map(|slot| self.generation.slot(*slot))
            .map(|slot| slot.importer.descriptor().clone())
            .collect()
    }

    pub fn importers(&self) -> Vec<Arc<dyn AssetImporterHandler>> {
        self.generation
            .slots()
            .map(|slot| slot.importer.clone())
            .collect()
    }

    pub fn remove_by_plugin_id(&mut self, plugin_id: &str) -> Vec<AssetImporterDescriptor> {
        Arc::make_mut(&mut self.generation).remove_by_plugin_id(plugin_id)
    }

    pub fn is_empty(&self) -> bool {
        self.generation.active_slots == 0
    }

    fn best_full_suffix_match(&self, source_path: &Path) -> Option<&AssetImporterSlot> {
        let name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let mut best = None;
        for (offset, _) in name.match_indices('.') {
            let suffix = &name[offset..];
            self.generation.consider_candidates(
                &self.generation.full_suffix_to_slots,
                suffix,
                suffix.len(),
                &mut best,
            );
        }
        best.and_then(|candidate| self.generation.slot(candidate.slot))
    }

    fn best_extension_match(&self, source_path: &Path) -> Option<&AssetImporterSlot> {
        let extension = source_path
            .extension()
            .and_then(|extension| extension.to_str())?;
        let mut best = None;
        self.generation.consider_candidates(
            &self.generation.extension_to_slots,
            extension,
            0,
            &mut best,
        );
        best.and_then(|candidate| self.generation.slot(candidate.slot))
    }
}

#[derive(Clone, Default)]
struct AssetImporterRegistryGeneration {
    slots: Vec<Option<AssetImporterSlot>>,
    extension_to_slots: MatcherIndex,
    full_suffix_to_slots: MatcherIndex,
    id_to_slot: HashMap<String, usize>,
    plugin_to_slots: HashMap<String, Vec<usize>>,
    active_slots: usize,
}

impl AssetImporterRegistryGeneration {
    fn slots(&self) -> impl Iterator<Item = &AssetImporterSlot> {
        self.slots.iter().filter_map(Option::as_ref)
    }

    fn slot(&self, slot: usize) -> Option<&AssetImporterSlot> {
        self.slots.get(slot).and_then(Option::as_ref)
    }

    fn has_matcher_at_priority(&self, index: &MatcherIndex, matcher: &str, priority: i32) -> bool {
        index.slots(matcher).is_some_and(|slots| {
            slots.iter().any(|slot| {
                self.slot(*slot)
                    .is_some_and(|existing| existing.importer.descriptor().priority == priority)
            })
        })
    }

    fn insert(&mut self, prepared: PreparedImporter) {
        let slot = self.slots.len();
        self.id_to_slot.insert(prepared.id.clone(), slot);
        self.plugin_to_slots
            .entry(prepared.plugin_id.clone())
            .or_default()
            .push(slot);
        for extension in &prepared.extensions {
            self.extension_to_slots.insert(extension, slot);
        }
        for suffix in &prepared.full_suffixes {
            self.full_suffix_to_slots.insert(suffix, slot);
        }
        self.slots.push(Some(AssetImporterSlot {
            importer: prepared.importer,
            availability: prepared.availability,
            extensions: prepared.extensions,
            full_suffixes: prepared.full_suffixes,
        }));
        self.active_slots += 1;
    }

    fn remove_by_plugin_id(&mut self, plugin_id: &str) -> Vec<AssetImporterDescriptor> {
        let Some(slots) = self.plugin_to_slots.remove(plugin_id) else {
            return Vec::new();
        };
        let mut removed = Vec::with_capacity(slots.len());
        for slot in slots {
            let Some(existing) = self.slots.get_mut(slot).and_then(Option::take) else {
                continue;
            };
            let descriptor = existing.importer.descriptor().clone();
            for extension in &existing.extensions {
                self.extension_to_slots.remove_slot(extension, slot);
            }
            for suffix in &existing.full_suffixes {
                self.full_suffix_to_slots.remove_slot(suffix, slot);
            }
            self.id_to_slot.remove(&descriptor.id);
            self.active_slots -= 1;
            removed.push(descriptor);
        }
        removed
    }

    fn consider_candidates(
        &self,
        index: &MatcherIndex,
        matcher: &str,
        suffix_len: usize,
        best: &mut Option<AssetImporterSelection>,
    ) {
        let Some(candidates) = index.slots(matcher) else {
            return;
        };
        for slot in candidates {
            let Some(candidate) = self.slot(*slot) else {
                continue;
            };
            let selection = AssetImporterSelection {
                slot: *slot,
                availability_rank: candidate.availability.rank,
                priority: candidate.importer.descriptor().priority,
                suffix_len,
            };
            if best.is_none_or(|current| selection.is_at_least_as_good_as(current)) {
                *best = Some(selection);
            }
        }
    }
}

#[derive(Clone)]
struct AssetImporterSlot {
    importer: Arc<dyn AssetImporterHandler>,
    availability: Arc<AssetImporterAvailability>,
    extensions: Vec<Box<str>>,
    full_suffixes: Vec<Box<str>>,
}

#[derive(Clone)]
struct AssetImporterAvailability {
    status: AssetImporterCapabilityStatus,
    rank: u8,
}

impl AssetImporterAvailability {
    fn new(status: AssetImporterCapabilityStatus) -> Self {
        let rank = u8::from(status.is_available());
        Self { status, rank }
    }
}

struct PreparedImporter {
    importer: Arc<dyn AssetImporterHandler>,
    id: String,
    plugin_id: String,
    priority: i32,
    availability: Arc<AssetImporterAvailability>,
    extensions: Vec<Box<str>>,
    full_suffixes: Vec<Box<str>>,
}

impl PreparedImporter {
    fn new(importer: Arc<dyn AssetImporterHandler>) -> Result<Self, AssetImporterRegistryError> {
        let descriptor = importer.descriptor();
        let extensions = normalized_matchers(&descriptor.source_extensions, normalize_extension);
        let full_suffixes = normalized_matchers(&descriptor.full_suffixes, normalize_full_suffix);
        if extensions.is_empty() && full_suffixes.is_empty() {
            return Err(AssetImporterRegistryError::MissingMatcher(
                descriptor.id.clone(),
            ));
        }
        if let Some(suffix) = full_suffixes
            .iter()
            .map(|suffix| suffix.as_ref())
            .find(|suffix| matches!(*suffix, ".ui.toml" | ".v2.ui.toml"))
        {
            return Err(
                AssetImporterRegistryError::DeprecatedUiDocumentSuffixImporter {
                    importer_id: descriptor.id.clone(),
                    suffix: suffix.to_owned(),
                },
            );
        }
        Ok(Self {
            importer: importer.clone(),
            id: descriptor.id.clone(),
            plugin_id: descriptor.plugin_id.clone(),
            priority: descriptor.priority,
            availability: Arc::new(AssetImporterAvailability::new(importer.capability_status())),
            extensions,
            full_suffixes,
        })
    }
}

fn normalized_matchers(matchers: &[String], normalize: impl Fn(&str) -> String) -> Vec<Box<str>> {
    let mut normalized = Vec::with_capacity(matchers.len());
    for matcher in matchers {
        let matcher = normalize(matcher);
        if !normalized
            .iter()
            .any(|existing: &Box<str>| existing.as_ref() == matcher)
        {
            normalized.push(matcher.into_boxed_str());
        }
    }
    normalized
}

#[derive(Clone, Default)]
struct MatcherIndex {
    // The folded hash avoids query-time matcher allocation. Hash collisions stay
    // correct because every bucket still checks ASCII-insensitive equality.
    buckets: HashMap<u64, Vec<MatcherIndexEntry>>,
}

impl MatcherIndex {
    fn insert(&mut self, matcher: &str, slot: usize) {
        let entries = self
            .buckets
            .entry(ascii_case_fold_hash(matcher))
            .or_default();
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.matcher.eq_ignore_ascii_case(matcher))
        {
            entry.slots.push(slot);
        } else {
            entries.push(MatcherIndexEntry {
                matcher: matcher.into(),
                slots: vec![slot],
            });
        }
    }

    fn slots(&self, matcher: &str) -> Option<&[usize]> {
        self.buckets
            .get(&ascii_case_fold_hash(matcher))?
            .iter()
            .find(|entry| entry.matcher.eq_ignore_ascii_case(matcher))
            .map(|entry| entry.slots.as_slice())
    }

    fn remove_slot(&mut self, matcher: &str, slot: usize) {
        let hash = ascii_case_fold_hash(matcher);
        let mut remove_bucket = false;
        if let Some(entries) = self.buckets.get_mut(&hash) {
            if let Some(position) = entries
                .iter()
                .position(|entry| entry.matcher.eq_ignore_ascii_case(matcher))
            {
                entries[position]
                    .slots
                    .retain(|candidate| *candidate != slot);
                if entries[position].slots.is_empty() {
                    entries.swap_remove(position);
                }
            }
            remove_bucket = entries.is_empty();
        }
        if remove_bucket {
            self.buckets.remove(&hash);
        }
    }
}

#[derive(Clone)]
struct MatcherIndexEntry {
    matcher: Box<str>,
    slots: Vec<usize>,
}

#[derive(Clone, Copy)]
struct AssetImporterSelection {
    slot: usize,
    availability_rank: u8,
    priority: i32,
    suffix_len: usize,
}

impl AssetImporterSelection {
    fn is_at_least_as_good_as(self, other: Self) -> bool {
        (
            self.availability_rank,
            self.priority,
            self.suffix_len,
            self.slot,
        ) >= (
            other.availability_rank,
            other.priority,
            other.suffix_len,
            other.slot,
        )
    }
}

fn ascii_case_fold_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte.to_ascii_lowercase())).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

impl fmt::Debug for AssetImporterRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssetImporterRegistry")
            .field("descriptors", &self.descriptors())
            .finish()
    }
}

fn unknown_typed_toml_suffix(source_path: &Path) -> Option<&str> {
    let name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let toml_start = name.len().checked_sub(".toml".len())?;
    if !name[toml_start..].eq_ignore_ascii_case(".toml") {
        return None;
    }
    let stem = &name[..toml_start];
    let typed_suffix_start = stem.rfind('.')?;
    let suffix = &name[typed_suffix_start..toml_start];
    if suffix.is_empty() {
        return None;
    }
    Some(suffix)
}

#[cfg(test)]
mod plugins07_descriptor_selection_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::asset::{AssetImportContext, AssetImportOutcome, AssetKind, FunctionAssetImporter};

    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 20_000;

    #[test]
    fn registry_label_hotpath_contract_descriptor_selection_borrows_slot() {
        let registry = benchmark_registry();
        let path = Path::new("fixture.plugins07");

        let selected = registry.descriptor_for_source(path).unwrap();

        assert_eq!(selected.id, "plugins07.registry.descriptor");
        assert_eq!(selected.plugin_id, "plugins07.registry");
        assert_eq!(selected.source_extensions, ["plugins07"]);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn registry_label_hotpath_performance_release_descriptor_selection() {
        let registry = benchmark_registry();
        let path = Path::new("fixture.plugins07");
        for _ in 0..4 {
            black_box(measure_legacy(&registry, path));
            black_box(measure_borrowed(&registry, path));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (
                    measure_legacy(&registry, path),
                    measure_borrowed(&registry, path),
                )
            } else {
                let optimized_ns = measure_borrowed(&registry, path);
                (measure_legacy(&registry, path), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_registry_borrowed_descriptor_selection sample_pairs={SAMPLE_PAIRS} lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=3 legacy_importer_arc_clones_per_sample={LOOKUPS_PER_SAMPLE} optimized_importer_arc_clones_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 3,
            "borrowed descriptor selection must improve P95 by at least 3%"
        );
    }

    fn benchmark_registry() -> AssetImporterRegistry {
        let mut registry = AssetImporterRegistry::default();
        registry
            .register(FunctionAssetImporter::new(
                AssetImporterDescriptor::new(
                    "plugins07.registry.descriptor",
                    "plugins07.registry",
                    AssetKind::Data,
                    1,
                )
                .with_source_extensions(["plugins07"]),
                unreachable_import,
            ))
            .unwrap();
        registry
    }

    fn unreachable_import(
        _context: &AssetImportContext,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        unreachable!("descriptor selection does not invoke the importer")
    }

    fn measure_legacy(registry: &AssetImporterRegistry, path: &Path) -> u128 {
        let started = Instant::now();
        for _ in 0..LOOKUPS_PER_SAMPLE {
            let descriptor = black_box(registry)
                .select(black_box(path))
                .unwrap()
                .descriptor()
                .clone();
            black_box(descriptor);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed(registry: &AssetImporterRegistry, path: &Path) -> u128 {
        let started = Instant::now();
        for _ in 0..LOOKUPS_PER_SAMPLE {
            let descriptor = black_box(registry)
                .descriptor_for_source(black_box(path))
                .unwrap();
            black_box(descriptor);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
