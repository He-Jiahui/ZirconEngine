mod locale_projection;
#[cfg(test)]
mod localization_tests;
#[cfg(test)]
mod performance_tests;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{de, Deserialize, Deserializer, Serialize};
use zircon_runtime_interface::ui::component::UiValue;

use super::{CommandEvalCtx, EditorCommandDescriptor, WhenClause};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale};

use locale_projection::EditorCommandPaletteLocaleProjection;

pub const EDITOR_COMMAND_PALETTE_MRU_CAPACITY: usize = 32;
const EDITOR_COMMAND_PALETTE_LOCALE_CACHE_CAPACITY: usize = 4;

/// Bounded, most-recent-first command history stored only in the Session settings layer.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct EditorCommandPaletteMru(Vec<EditorOperationPath>);

impl EditorCommandPaletteMru {
    pub fn new(entries: impl IntoIterator<Item = EditorOperationPath>) -> Result<Self, String> {
        let mru = Self(entries.into_iter().collect());
        mru.validate()?;
        Ok(mru)
    }

    pub fn entries(&self) -> &[EditorOperationPath] {
        &self.0
    }

    /// Records one successful command dispatch and returns whether the order changed.
    pub fn record(&mut self, command: EditorOperationPath) -> bool {
        if self.0.first() == Some(&command) {
            return false;
        }
        self.0.retain(|entry| entry != &command);
        self.0.insert(0, command);
        self.0.truncate(EDITOR_COMMAND_PALETTE_MRU_CAPACITY);
        true
    }

    #[cfg(test)]
    fn contains_id(&self, command_id: &str) -> bool {
        self.0.iter().any(|entry| entry.as_str() == command_id)
    }

    fn rank_of(&self, command_id: &str) -> usize {
        self.0
            .iter()
            .position(|entry| entry.as_str() == command_id)
            .unwrap_or(usize::MAX)
    }

    fn validate(&self) -> Result<(), String> {
        if self.0.len() > EDITOR_COMMAND_PALETTE_MRU_CAPACITY {
            return Err(format!(
                "command palette MRU must contain at most {EDITOR_COMMAND_PALETTE_MRU_CAPACITY} entries"
            ));
        }
        let unique = self.0.iter().collect::<BTreeSet<_>>();
        if unique.len() != self.0.len() {
            return Err("command palette MRU cannot contain duplicate command ids".to_string());
        }
        Ok(())
    }
}

struct EditorCommandPaletteMruIndices {
    sorted: [usize; EDITOR_COMMAND_PALETTE_MRU_CAPACITY],
    len: usize,
}

impl EditorCommandPaletteMruIndices {
    fn new(mru: &EditorCommandPaletteMru, entry_indices: &BTreeMap<String, usize>) -> Self {
        let mut sorted = [usize::MAX; EDITOR_COMMAND_PALETTE_MRU_CAPACITY];
        let mut len = 0;
        for command_id in mru.entries().iter().take(sorted.len()) {
            let Some(index) = entry_indices.get(command_id.as_str()).copied() else {
                continue;
            };
            sorted[len] = index;
            len += 1;
        }
        sorted[..len].sort_unstable();
        Self { sorted, len }
    }

    fn contains(&self, index: usize) -> bool {
        self.sorted[..self.len].binary_search(&index).is_ok()
    }
}

impl<'de> Deserialize<'de> for EditorCommandPaletteMru {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = Vec::<EditorOperationPath>::deserialize(deserializer)?;
        Self::new(entries).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCommandPaletteEntry {
    pub id: String,
    pub label: String,
    pub source: String,
    pub shortcut: String,
    pub category: String,
    pub keywords: Vec<String>,
}

impl EditorCommandPaletteEntry {
    pub fn to_ui_value(&self) -> UiValue {
        let mut values = BTreeMap::new();
        values.insert("id".to_string(), UiValue::String(self.id.clone()));
        values.insert("label".to_string(), UiValue::String(self.label.clone()));
        values.insert("source".to_string(), UiValue::String(self.source.clone()));
        values.insert(
            "shortcut".to_string(),
            UiValue::String(self.shortcut.clone()),
        );
        values.insert(
            "category".to_string(),
            UiValue::String(self.category.clone()),
        );
        values.insert(
            "keywords".to_string(),
            UiValue::Array(self.keywords.iter().cloned().map(UiValue::String).collect()),
        );
        UiValue::Map(values)
    }
}

#[derive(Clone, Debug)]
struct EditorCommandPaletteSeed {
    id: String,
    presentation: super::EditorCommandPresentation,
    source: String,
    shortcut: String,
    category_key: &'static str,
    keywords: Vec<String>,
}

impl EditorCommandPaletteSeed {
    fn from_descriptor(descriptor: &EditorCommandDescriptor) -> Self {
        Self {
            id: descriptor.id().to_string(),
            presentation: descriptor.presentation().clone(),
            source: descriptor.category().source_tag().to_string(),
            shortcut: descriptor
                .default_chord()
                .map(ToString::to_string)
                .unwrap_or_default(),
            category_key: descriptor.category().localization_key(),
            keywords: descriptor.keywords().to_vec(),
        }
    }

    fn project(
        &self,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
    ) -> EditorCommandPaletteEntry {
        EditorCommandPaletteEntry {
            id: self.id.clone(),
            label: self
                .presentation
                .resolve_label(i18n, locale)
                .as_ref()
                .to_owned(),
            source: self.source.clone(),
            shortcut: self.shortcut.clone(),
            category: i18n
                .translate_for_locale(locale, self.category_key)
                .as_ref()
                .to_owned(),
            keywords: self.keywords.clone(),
        }
    }
}

/// Immutable command discovery data shared by every palette open in one registry generation.
#[derive(Debug)]
pub struct EditorCommandPaletteCatalog {
    generation: u64,
    seeds: Arc<[EditorCommandPaletteSeed]>,
    entry_indices: BTreeMap<String, usize>,
    enablement: Arc<[EditorCommandPaletteEnablement]>,
    locale_projections: Mutex<VecDeque<Arc<EditorCommandPaletteLocaleProjection>>>,
}

impl EditorCommandPaletteCatalog {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.seeds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seeds.is_empty()
    }

    pub(super) fn from_descriptors<'a>(
        generation: u64,
        descriptors: impl Iterator<Item = &'a EditorCommandDescriptor>,
    ) -> Self {
        let mut seeds = Vec::new();
        let mut entry_indices = BTreeMap::new();
        let mut enablement = Vec::new();
        for descriptor in descriptors {
            let seed = EditorCommandPaletteSeed::from_descriptor(descriptor);
            entry_indices.insert(seed.id.clone(), seeds.len());
            enablement.push(EditorCommandPaletteEnablement::from_descriptor(descriptor));
            seeds.push(seed);
        }
        Self {
            generation,
            seeds: seeds.into(),
            entry_indices,
            enablement: enablement.into(),
            locale_projections: Mutex::new(VecDeque::new()),
        }
    }

    fn locale_projection(
        &self,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
    ) -> Arc<EditorCommandPaletteLocaleProjection> {
        {
            let cache = self.lock_locale_projections();
            if let Some(projection) = cache.iter().find(|projection| projection.matches(locale)) {
                return Arc::clone(projection);
            }
        }

        let built = Arc::new(EditorCommandPaletteLocaleProjection::build(
            i18n,
            locale,
            &self.seeds,
        ));
        let mut cache = self.lock_locale_projections();
        if let Some(projection) = cache.iter().find(|projection| projection.matches(locale)) {
            return Arc::clone(projection);
        }
        cache.push_front(Arc::clone(&built));
        cache.truncate(EDITOR_COMMAND_PALETTE_LOCALE_CACHE_CAPACITY);
        built
    }

    fn lock_locale_projections(
        &self,
    ) -> MutexGuard<'_, VecDeque<Arc<EditorCommandPaletteLocaleProjection>>> {
        self.locale_projections
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    fn cached_locale_projection_count(&self) -> usize {
        self.lock_locale_projections().len()
    }

    pub(crate) fn query_window(
        self: &Arc<Self>,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        context: &CommandEvalCtx,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> EditorCommandPaletteQueryWindow {
        self.query_window_with_mru(
            i18n,
            locale,
            context,
            query,
            offset,
            limit,
            &EditorCommandPaletteMru::default(),
        )
    }

    pub(crate) fn query_window_with_mru(
        self: &Arc<Self>,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        context: &CommandEvalCtx,
        query: &str,
        offset: usize,
        limit: usize,
        mru: &EditorCommandPaletteMru,
    ) -> EditorCommandPaletteQueryWindow {
        let projection = self.locale_projection(i18n, locale);
        let query = EditorCommandPaletteCompiledQuery::new(query);
        if query.is_empty() {
            return self.unfiltered_window(projection, context, offset, limit, mru);
        }

        let mut metrics = EditorCommandPaletteQueryMetrics {
            owned_buffers: 3,
            ..EditorCommandPaletteQueryMetrics::default()
        };
        let candidate_limit = (limit > 0)
            .then(|| offset.saturating_add(limit).min(projection.entries.len()))
            .unwrap_or_default();
        let mut candidates = BinaryHeap::with_capacity(candidate_limit);
        let candidate_indices = query.rarest_posting(&projection.search_postings);
        for &index in candidate_indices {
            metrics.visited_entries += 1;
            metrics.enablement_evaluations += 1;
            if !self.enablement[index].is_enabled(context) {
                continue;
            }
            let document = &projection.search_documents[index];
            if let Some(score) = fuzzy_score(document, &query, &mut metrics) {
                metrics.total_matches += 1;
                retain_top_candidate(
                    &mut candidates,
                    candidate_limit,
                    EditorCommandPaletteCandidate {
                        index,
                        score,
                        mru_rank: mru.rank_of(projection.entries[index].id.as_str()),
                    },
                );
            }
        }

        let mut candidates = candidates.into_vec();
        candidates.sort_unstable_by(EditorCommandPaletteCandidate::rank_cmp);
        metrics.candidate_handles = candidates.len();
        let handles = candidates
            .into_iter()
            .enumerate()
            .skip(offset)
            .take(limit)
            .map(|(rank, candidate)| (rank, candidate.index))
            .collect::<Vec<_>>();
        metrics.owned_buffers += 1;
        metrics.retained_handles = handles.len();

        EditorCommandPaletteQueryWindow {
            catalog_generation: self.generation,
            projection,
            handles: handles.into_boxed_slice(),
            offset,
            metrics,
        }
    }

    fn unfiltered_window(
        self: &Arc<Self>,
        projection: Arc<EditorCommandPaletteLocaleProjection>,
        context: &CommandEvalCtx,
        offset: usize,
        limit: usize,
        mru: &EditorCommandPaletteMru,
    ) -> EditorCommandPaletteQueryWindow {
        let mut handles = Vec::with_capacity(limit);
        let mut total_matches = 0;
        let window_end = offset.saturating_add(limit);
        let mru_indices = EditorCommandPaletteMruIndices::new(mru, &self.entry_indices);
        for command_id in mru.entries() {
            let Some(index) = self.entry_indices.get(command_id.as_str()).copied() else {
                continue;
            };
            if !self.enablement[index].is_enabled(context) {
                continue;
            }
            if (offset..window_end).contains(&total_matches) {
                handles.push((total_matches, index));
            }
            total_matches += 1;
        }
        for index in 0..projection.entries.len() {
            if mru_indices.contains(index) {
                continue;
            }
            if !self.enablement[index].is_enabled(context) {
                continue;
            }
            if (offset..window_end).contains(&total_matches) {
                handles.push((total_matches, index));
            }
            total_matches += 1;
        }
        EditorCommandPaletteQueryWindow {
            catalog_generation: self.generation,
            projection,
            offset,
            metrics: EditorCommandPaletteQueryMetrics {
                visited_entries: self.seeds.len(),
                enablement_evaluations: self.seeds.len(),
                total_matches,
                retained_handles: handles.len(),
                owned_buffers: 1,
                ..EditorCommandPaletteQueryMetrics::default()
            },
            handles: handles.into_boxed_slice(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorCommandPaletteQueryMetrics {
    pub visited_entries: usize,
    pub enablement_evaluations: usize,
    /// Source document bytes visited by the single-pass fuzzy matcher.
    pub document_byte_visits: usize,
    pub text_comparisons: usize,
    pub total_matches: usize,
    pub candidate_handles: usize,
    pub retained_handles: usize,
    /// Owned variable-size buffers: normalized query, prefix table, bounded candidates, and result
    /// handles.
    pub owned_buffers: usize,
}

/// A ranked page of lightweight handles into one immutable catalog generation.
#[derive(Debug)]
pub struct EditorCommandPaletteQueryWindow {
    catalog_generation: u64,
    projection: Arc<EditorCommandPaletteLocaleProjection>,
    handles: Box<[(usize, usize)]>,
    offset: usize,
    metrics: EditorCommandPaletteQueryMetrics,
}

impl EditorCommandPaletteQueryWindow {
    pub fn catalog_generation(&self) -> u64 {
        self.catalog_generation
    }

    pub fn total_match_count(&self) -> usize {
        self.metrics.total_matches
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    pub fn metrics(&self) -> EditorCommandPaletteQueryMetrics {
        self.metrics
    }

    pub fn entries(
        &self,
    ) -> impl DoubleEndedIterator<Item = &EditorCommandPaletteEntry> + ExactSizeIterator {
        self.handles
            .iter()
            .map(|(_, catalog_index)| &self.projection.entries[*catalog_index])
    }

    pub fn to_ui_value(&self) -> UiValue {
        UiValue::Array(
            self.entries()
                .map(EditorCommandPaletteEntry::to_ui_value)
                .collect(),
        )
    }
}

#[derive(Clone, Debug)]
struct EditorCommandPaletteEnablement {
    when: WhenClause,
    required_capabilities: Box<[Box<str>]>,
    requires_writable_asset: bool,
}

impl EditorCommandPaletteEnablement {
    fn from_descriptor(descriptor: &EditorCommandDescriptor) -> Self {
        Self {
            when: descriptor.when().clone(),
            required_capabilities: descriptor
                .required_capabilities()
                .iter()
                .cloned()
                .map(String::into_boxed_str)
                .collect(),
            requires_writable_asset: descriptor.asset_write_target().is_some(),
        }
    }

    fn is_enabled(&self, context: &CommandEvalCtx) -> bool {
        self.when.eval(context)
            && self
                .required_capabilities
                .iter()
                .all(|capability| context.has_capability(capability))
            && (!self.requires_writable_asset || WhenClause::AssetWritable.eval(context))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EditorCommandPaletteCandidate {
    index: usize,
    score: u8,
    mru_rank: usize,
}

impl EditorCommandPaletteCandidate {
    fn rank_cmp(left: &Self, right: &Self) -> Ordering {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.mru_rank.cmp(&right.mru_rank))
            .then_with(|| left.index.cmp(&right.index))
    }
}

impl Ord for EditorCommandPaletteCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        Self::rank_cmp(self, other)
    }
}

impl PartialOrd for EditorCommandPaletteCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn retain_top_candidate(
    candidates: &mut BinaryHeap<EditorCommandPaletteCandidate>,
    candidate_limit: usize,
    candidate: EditorCommandPaletteCandidate,
) {
    if candidate_limit == 0 {
        return;
    }
    if candidates.len() < candidate_limit {
        candidates.push(candidate);
        return;
    }
    if candidates.peek().is_some_and(|worst| candidate < *worst) {
        let _ = candidates.pop();
        candidates.push(candidate);
    }
}

fn search_document(entry: &EditorCommandPaletteEntry) -> Box<str> {
    let keyword_bytes = entry.keywords.iter().map(String::len).sum::<usize>();
    let mut document = String::with_capacity(
        entry.id.len()
            + entry.label.len()
            + entry.source.len()
            + entry.category.len()
            + keyword_bytes
            + entry.keywords.len()
            + 4,
    );
    for value in [&entry.id, &entry.label, &entry.source, &entry.category] {
        document.push_str(value);
        document.push(' ');
    }
    for keyword in &entry.keywords {
        document.push_str(keyword);
        document.push(' ');
    }
    document.make_ascii_lowercase();
    document.into_boxed_str()
}

struct EditorCommandPaletteCompiledQuery {
    normalized: String,
    prefix_lengths: Box<[usize]>,
}

impl EditorCommandPaletteCompiledQuery {
    fn new(query: &str) -> Self {
        let normalized = query.trim().to_lowercase();
        let bytes = normalized.as_bytes();
        let mut prefix_lengths = vec![0; bytes.len()];
        let mut prefix_len = 0;
        for index in 1..bytes.len() {
            while prefix_len > 0 && bytes[index] != bytes[prefix_len] {
                prefix_len = prefix_lengths[prefix_len - 1];
            }
            if bytes[index] == bytes[prefix_len] {
                prefix_len += 1;
            }
            prefix_lengths[index] = prefix_len;
        }
        Self {
            normalized,
            prefix_lengths: prefix_lengths.into_boxed_slice(),
        }
    }

    fn is_empty(&self) -> bool {
        self.normalized.is_empty()
    }

    fn bytes(&self) -> &[u8] {
        self.normalized.as_bytes()
    }

    fn rarest_posting<'a>(&self, postings: &'a [Box<[usize]>; 256]) -> &'a [usize] {
        let mut visited = [false; 256];
        self.bytes()
            .iter()
            .copied()
            .filter(|byte| {
                let visited = &mut visited[usize::from(*byte)];
                let first = !*visited;
                *visited = true;
                first
            })
            .map(|byte| postings[usize::from(byte)].as_ref())
            .min_by_key(|posting| posting.len())
            .unwrap_or_default()
    }
}

fn fuzzy_score(
    document: &str,
    query: &EditorCommandPaletteCompiledQuery,
    metrics: &mut EditorCommandPaletteQueryMetrics,
) -> Option<u8> {
    let document = document.as_bytes();
    let query_bytes = query.bytes();

    let mut query_index = 0;
    let mut first_match = None;
    let mut last_match = 0;
    let mut gap_count = 0usize;
    let mut subsequence_score = None;
    let mut contiguous_len = 0;
    for (document_index, value) in document.iter().copied().enumerate() {
        metrics.document_byte_visits += 1;

        while contiguous_len > 0 {
            metrics.text_comparisons += 1;
            if query_bytes[contiguous_len] == value {
                break;
            }
            contiguous_len = query.prefix_lengths[contiguous_len - 1];
        }
        metrics.text_comparisons += 1;
        if query_bytes[contiguous_len] == value {
            contiguous_len += 1;
            if contiguous_len == query_bytes.len() {
                return Some(255);
            }
        }

        if subsequence_score.is_none() {
            metrics.text_comparisons += 1;
            if query_bytes.get(query_index).copied() != Some(value) {
                continue;
            }
            if let Some(previous) = first_match.map(|_| last_match) {
                gap_count += document_index.saturating_sub(previous + 1);
            } else {
                first_match = Some(document_index);
            }
            last_match = document_index;
            query_index += 1;
            if query_index == query_bytes.len() {
                let start_penalty = first_match.unwrap_or_default().min(48);
                let gap_penalty = gap_count.min(96);
                subsequence_score = Some(
                    224usize
                        .saturating_sub(start_penalty)
                        .saturating_sub(gap_penalty) as u8,
                );
            }
        }
    }
    subsequence_score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_pass_fuzzy_score_preserves_exact_and_subsequence_ranking() {
        let exact_query = EditorCommandPaletteCompiledQuery::new("ha");
        let mut exact_metrics = EditorCommandPaletteQueryMetrics::default();
        let exact = fuzzy_score("alpha beta", &exact_query, &mut exact_metrics);

        let subsequence_query = EditorCommandPaletteCompiledQuery::new("ab");
        let mut subsequence_metrics = EditorCommandPaletteQueryMetrics::default();
        let subsequence = fuzzy_score("alpha beta", &subsequence_query, &mut subsequence_metrics);

        assert_eq!(exact, Some(255));
        assert_eq!(subsequence, Some(219));
        assert!(exact_metrics.document_byte_visits <= "alpha beta".len());
        assert_eq!(subsequence_metrics.document_byte_visits, "alpha beta".len());
    }

    #[test]
    fn later_exact_match_still_overrides_an_earlier_subsequence() {
        let query = EditorCommandPaletteCompiledQuery::new("ab");
        let mut metrics = EditorCommandPaletteQueryMetrics::default();

        assert_eq!(
            fuzzy_score("a gap before ab", &query, &mut metrics),
            Some(255)
        );
        assert!(metrics.document_byte_visits < "a gap before ab".len());
    }

    #[test]
    fn rarest_posting_keeps_repeated_query_byte_candidates() {
        let query = EditorCommandPaletteCompiledQuery::new("letter");
        let mut postings: [Box<[usize]>; 256] =
            std::array::from_fn(|_| Vec::new().into_boxed_slice());
        postings[usize::from(b'e')] = vec![2, 7].into_boxed_slice();
        postings[usize::from(b'l')] = vec![2, 4, 7].into_boxed_slice();
        postings[usize::from(b't')] = vec![2, 5, 7].into_boxed_slice();
        postings[usize::from(b'r')] = vec![2, 6, 7].into_boxed_slice();

        assert_eq!(query.rarest_posting(&postings), &[2, 7]);
    }
}
