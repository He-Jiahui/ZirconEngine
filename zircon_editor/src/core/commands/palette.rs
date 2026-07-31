use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::sync::Arc;

use serde::{de, Deserialize, Deserializer, Serialize};
use zircon_runtime_interface::ui::component::UiValue;

use super::{CommandEvalCtx, EditorCommandDescriptor, WhenClause};
use crate::core::editor_operation::EditorOperationPath;

pub const EDITOR_COMMAND_PALETTE_MRU_CAPACITY: usize = 32;

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
    pub fn from_descriptor(descriptor: &EditorCommandDescriptor) -> Self {
        Self {
            id: descriptor.id().to_string(),
            label: descriptor.display_name().to_string(),
            source: descriptor.category().source_tag().to_string(),
            shortcut: descriptor
                .default_chord()
                .map(ToString::to_string)
                .unwrap_or_default(),
            category: descriptor.category().as_str().to_string(),
            keywords: descriptor.keywords().to_vec(),
        }
    }

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

/// Immutable command discovery data shared by every palette open in one registry generation.
#[derive(Debug)]
pub struct EditorCommandPaletteCatalog {
    generation: u64,
    entries: Arc<[EditorCommandPaletteEntry]>,
    entry_indices: BTreeMap<String, usize>,
    search_documents: Arc<[Box<str>]>,
    enablement: Arc<[EditorCommandPaletteEnablement]>,
}

impl EditorCommandPaletteCatalog {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[EditorCommandPaletteEntry] {
        &self.entries
    }

    pub(super) fn from_descriptors<'a>(
        generation: u64,
        descriptors: impl Iterator<Item = &'a EditorCommandDescriptor>,
    ) -> Self {
        let mut entries = Vec::new();
        let mut entry_indices = BTreeMap::new();
        let mut search_documents = Vec::new();
        let mut enablement = Vec::new();
        for descriptor in descriptors {
            let entry = EditorCommandPaletteEntry::from_descriptor(descriptor);
            entry_indices.insert(entry.id.clone(), entries.len());
            search_documents.push(search_document(&entry));
            enablement.push(EditorCommandPaletteEnablement::from_descriptor(descriptor));
            entries.push(entry);
        }
        Self {
            generation,
            entries: entries.into(),
            entry_indices,
            search_documents: search_documents.into(),
            enablement: enablement.into(),
        }
    }

    pub(super) fn query_window(
        self: &Arc<Self>,
        context: &CommandEvalCtx,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> EditorCommandPaletteQueryWindow {
        self.query_window_with_mru(
            context,
            query,
            offset,
            limit,
            &EditorCommandPaletteMru::default(),
        )
    }

    pub(super) fn query_window_with_mru(
        self: &Arc<Self>,
        context: &CommandEvalCtx,
        query: &str,
        offset: usize,
        limit: usize,
        mru: &EditorCommandPaletteMru,
    ) -> EditorCommandPaletteQueryWindow {
        let normalized_query = query.trim().to_lowercase();
        if normalized_query.is_empty() {
            return self.unfiltered_window(context, offset, limit, mru);
        }

        let mut metrics = EditorCommandPaletteQueryMetrics {
            owned_buffers: 2,
            ..EditorCommandPaletteQueryMetrics::default()
        };
        let candidate_limit = (limit > 0)
            .then(|| offset.saturating_add(limit).min(self.entries.len()))
            .unwrap_or_default();
        let mut candidates = BinaryHeap::with_capacity(candidate_limit);
        for ((index, document), enablement) in self
            .search_documents
            .iter()
            .enumerate()
            .zip(self.enablement.iter())
        {
            metrics.visited_entries += 1;
            metrics.enablement_evaluations += 1;
            if !enablement.is_enabled(context) {
                continue;
            }
            if let Some(score) = fuzzy_score(document, &normalized_query, &mut metrics) {
                metrics.total_matches += 1;
                retain_top_candidate(
                    &mut candidates,
                    candidate_limit,
                    EditorCommandPaletteCandidate {
                        index,
                        score,
                        mru_rank: mru.rank_of(self.entries[index].id.as_str()),
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
            catalog: Arc::clone(self),
            handles: handles.into_boxed_slice(),
            offset,
            metrics,
        }
    }

    fn unfiltered_window(
        self: &Arc<Self>,
        context: &CommandEvalCtx,
        offset: usize,
        limit: usize,
        mru: &EditorCommandPaletteMru,
    ) -> EditorCommandPaletteQueryWindow {
        let mut handles = Vec::with_capacity(limit);
        let mut total_matches = 0;
        let window_end = offset.saturating_add(limit);
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
        for index in 0..self.entries.len() {
            if mru.contains_id(self.entries[index].id.as_str()) {
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
            catalog: Arc::clone(self),
            offset,
            metrics: EditorCommandPaletteQueryMetrics {
                visited_entries: self.entries.len(),
                enablement_evaluations: self.entries.len(),
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
    pub text_comparisons: usize,
    pub total_matches: usize,
    pub candidate_handles: usize,
    pub retained_handles: usize,
    /// Owned variable-size buffers: normalized query, bounded candidates, and result handles.
    pub owned_buffers: usize,
}

/// A ranked page of lightweight handles into one immutable catalog generation.
#[derive(Debug)]
pub struct EditorCommandPaletteQueryWindow {
    catalog: Arc<EditorCommandPaletteCatalog>,
    handles: Box<[(usize, usize)]>,
    offset: usize,
    metrics: EditorCommandPaletteQueryMetrics,
}

impl EditorCommandPaletteQueryWindow {
    pub fn catalog_generation(&self) -> u64 {
        self.catalog.generation()
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
            .map(|(_, catalog_index)| &self.catalog.entries[*catalog_index])
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

fn fuzzy_score(
    document: &str,
    query: &str,
    metrics: &mut EditorCommandPaletteQueryMetrics,
) -> Option<u8> {
    let document = document.as_bytes();
    let query = query.as_bytes();
    metrics.text_comparisons += document.len().saturating_sub(query.len()).saturating_add(1);
    if document.windows(query.len()).any(|window| window == query) {
        return Some(255);
    }

    let mut query_index = 0;
    let mut first_match = None;
    let mut last_match = 0;
    let mut gap_count = 0usize;
    for (document_index, value) in document.iter().copied().enumerate() {
        metrics.text_comparisons += 1;
        if query.get(query_index).copied() != Some(value) {
            continue;
        }
        if let Some(previous) = first_match.map(|_| last_match) {
            gap_count += document_index.saturating_sub(previous + 1);
        } else {
            first_match = Some(document_index);
        }
        last_match = document_index;
        query_index += 1;
        if query_index == query.len() {
            let start_penalty = first_match.unwrap_or_default().min(48);
            let gap_penalty = gap_count.min(96);
            return Some(
                224usize
                    .saturating_sub(start_penalty)
                    .saturating_sub(gap_penalty) as u8,
            );
        }
    }
    None
}
