use std::sync::Arc;

use super::{UiAssetPaletteCatalog, UiAssetPaletteEntry, UiAssetPaletteEntryKind};
use crate::ui::asset_editor::source::source_buffer::{UiAssetSourceBuffer, UiAssetSourceSnapshot};

const PAGE_SIZE: usize = 4;

/// Query and page allocations retire outstanding row targets, including A -> B -> A.
#[derive(Debug)]
pub(super) struct UiAssetPaletteSearchState {
    query: String,
    matches: Arc<[usize]>,
    pub(super) visible_start: usize,
}

impl UiAssetPaletteSearchState {
    pub(super) fn new(entries: &[UiAssetPaletteEntry], query: &str, start: usize) -> Self {
        let terms = query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        let matches = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| terms.is_empty() || matches_terms(entry, &terms))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let last_page = matches.len().saturating_sub(1) / PAGE_SIZE * PAGE_SIZE;
        Self {
            query: query.to_string(),
            matches: matches.into(),
            visible_start: (start / PAGE_SIZE * PAGE_SIZE).min(last_page),
        }
    }
}

fn matches_terms(entry: &UiAssetPaletteEntry, terms: &[String]) -> bool {
    let label = entry.label.to_lowercase();
    let target = match &entry.kind {
        UiAssetPaletteEntryKind::Native { widget_type, .. } => widget_type,
        UiAssetPaletteEntryKind::Component { component } => component,
        UiAssetPaletteEntryKind::Reference { component_ref } => component_ref,
    }
    .to_lowercase();
    terms
        .iter()
        .all(|term| label.contains(term.as_str()) || target.contains(term.as_str()))
}

/// Immutable rows retain the real catalog, query/page and source allocations.
/// The host must additionally retain its document instance/input context.
#[derive(Clone, Debug)]
pub(crate) struct UiAssetPaletteSearchSnapshot {
    entries: Arc<[UiAssetPaletteEntry]>,
    search: Arc<UiAssetPaletteSearchState>,
    asset_id: String,
    source: UiAssetSourceSnapshot,
    source_revision: u64,
    selected_index: Option<usize>,
    can_select: bool,
}

impl PartialEq for UiAssetPaletteSearchSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.same_source(other)
            && Arc::ptr_eq(&self.entries, &other.entries)
            && Arc::ptr_eq(&self.search, &other.search)
            && self.selected_index == other.selected_index
            && self.can_select == other.can_select
    }
}

impl Eq for UiAssetPaletteSearchSnapshot {}

impl UiAssetPaletteSearchSnapshot {
    pub(crate) fn same_source(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id && self.source.same_identity(&other.source)
    }

    pub(crate) fn query(&self) -> &str {
        &self.search.query
    }

    pub(crate) fn asset_id(&self) -> &str {
        &self.asset_id
    }

    pub(crate) fn source_revision(&self) -> u64 {
        self.source_revision
    }

    pub(crate) fn result_count(&self) -> usize {
        self.search.matches.len()
    }

    pub(crate) fn total_count(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn visible_start(&self) -> usize {
        self.search.visible_start
    }

    pub(crate) fn has_previous(&self) -> bool {
        self.visible_start() != 0
    }

    pub(crate) fn has_next(&self) -> bool {
        self.visible_start().saturating_add(PAGE_SIZE) < self.result_count()
    }

    pub(crate) fn can_select(&self) -> bool {
        self.can_select && self.visible_start() < self.result_count()
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = (usize, &UiAssetPaletteEntry)> + '_ {
        self.search
            .matches
            .iter()
            .skip(self.visible_start())
            .take(PAGE_SIZE)
            .map(|index| (*index, &self.entries[*index]))
    }

    pub(crate) fn selected_row(&self) -> Option<usize> {
        self.rows()
            .position(|(index, _)| Some(index) == self.selected_index)
    }

    pub(crate) fn catalog_index_for_row(&self, row: usize) -> Option<usize> {
        if row >= PAGE_SIZE {
            return None;
        }
        self.search.matches.get(self.visible_start() + row).copied()
    }

    pub(crate) fn is_current_source(&self, asset_id: &str, source: &UiAssetSourceBuffer) -> bool {
        self.asset_id == asset_id && source.is_current_snapshot(&self.source)
    }
}

impl UiAssetPaletteCatalog {
    pub(crate) fn search_query(&self) -> &str {
        &self.search.query
    }

    pub(crate) fn set_search_query(&mut self, query: &str) -> bool {
        if self.search.query == query {
            return false;
        }
        self.search = Arc::new(UiAssetPaletteSearchState::new(&self.entries, query, 0));
        true
    }

    pub(super) fn set_search_visible_start(&mut self, index: usize) -> bool {
        let last_page = self.search.matches.len().saturating_sub(1) / PAGE_SIZE * PAGE_SIZE;
        let next = (index / PAGE_SIZE * PAGE_SIZE).min(last_page);
        if next == self.visible_start() {
            return false;
        }
        self.search = Arc::new(UiAssetPaletteSearchState {
            query: self.search.query.clone(),
            matches: Arc::clone(&self.search.matches),
            visible_start: next,
        });
        true
    }

    pub(crate) fn inherit_search_navigation(&mut self, previous: &Self) {
        self.set_search_query(previous.search_query());
        self.set_visible_start(previous.visible_start());
    }

    pub(crate) fn search_contains_entry(&self, index: usize) -> bool {
        self.search.matches.binary_search(&index).is_ok()
    }

    pub(crate) fn reveal_search_entry(&mut self, index: usize) {
        if let Ok(position) = self.search.matches.binary_search(&index) {
            self.set_visible_start(position);
        }
    }

    pub(crate) fn search_snapshot(
        &self,
        asset_id: &str,
        source: &UiAssetSourceBuffer,
        selected_index: Option<usize>,
        can_select: bool,
    ) -> UiAssetPaletteSearchSnapshot {
        UiAssetPaletteSearchSnapshot {
            entries: Arc::clone(&self.entries),
            search: Arc::clone(&self.search),
            asset_id: asset_id.to_string(),
            source: source.snapshot(),
            source_revision: source.revision(),
            selected_index,
            can_select,
        }
    }

    pub(crate) fn is_current_search_snapshot(
        &self,
        snapshot: &UiAssetPaletteSearchSnapshot,
    ) -> bool {
        Arc::ptr_eq(&self.entries, &snapshot.entries) && Arc::ptr_eq(&self.search, &snapshot.search)
    }
}
