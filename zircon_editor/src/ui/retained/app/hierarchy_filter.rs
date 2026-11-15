use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeSet;

use zircon_runtime::scene::{NodeId, WorldInspectionHierarchyRow};

use super::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::workbench::snapshot::SceneEntries;

struct HierarchyFilterCache {
    source: SceneEntries,
    query: String,
    filtered: SceneEntries,
}

thread_local! {
    // One current query per UI thread; retaining the source prevents pointer-reuse cache hits.
    static FILTER_CACHE: RefCell<Option<HierarchyFilterCache>> = const { RefCell::new(None) };
}

impl SceneEntries {
    pub(crate) fn filtered_by_hierarchy_query(&self, query: &str) -> Self {
        let query = query.trim();
        if query.is_empty() {
            return self.clone();
        }
        let name_rows = FILTER_CACHE.with_borrow_mut(|cache| {
            if let Some(cached) = cache.as_ref() {
                if cached.source.shares_hierarchy_with(self)
                    && cached.source.inspection_generation() == self.inspection_generation()
                    && cached.query == query
                {
                    return cached.filtered.hierarchy_rows_arc();
                }
            }
            let filtered = self.with_hierarchy_rows(hierarchy_entries_matching_query(self, query));
            let rows = filtered.hierarchy_rows_arc();
            *cache = Some(HierarchyFilterCache {
                source: self.clone(),
                query: query.to_owned(),
                filtered,
            });
            rows
        });
        // Name matches stay cached without selection. Selected entities that miss the
        // query stay visible so the outliner and the viewport outline describe one selection.
        let visible = if rows_cover_selection(&name_rows, self.selected_entities()) {
            name_rows
        } else {
            hierarchy_entries_keeping_selection(self, &name_rows, self.selected_entities()).into()
        };
        self.with_hierarchy_rows(visible)
    }
}

impl RetainedEditorHost {
    pub(super) fn hierarchy_filter_query(&self) -> &str {
        &self.hierarchy_filter_query
    }

    pub(super) fn filtered_hierarchy_entries(
        &self,
        entries: &SceneEntries,
    ) -> Option<SceneEntries> {
        zircon_runtime::profile_scope!("editor", "hierarchy", "filter_projection");
        let query = self.hierarchy_filter_query.trim();
        if query.is_empty() {
            record_hierarchy_filter_metrics(entries.len(), entries.len(), 0, entries.len());
            return None;
        }
        Some(entries.filtered_by_hierarchy_query(query))
    }

    pub(super) fn set_hierarchy_filter_query(&mut self, query: &str) {
        if self.hierarchy_filter_query == query {
            return;
        }
        self.hierarchy_filter_query = query.to_string();
        self.workbench_window_bridge.set_scene_filter_query(query);
        self.invalidate_host(
            HostInvalidationMask::PRESENTATION_DATA.union(HostInvalidationMask::HIT_TEST),
        );
    }
}

fn rows_cover_selection(rows: &[WorldInspectionHierarchyRow], selected: &BTreeSet<NodeId>) -> bool {
    selected
        .iter()
        .all(|entity| rows.iter().any(|row| row.entity == *entity))
}

fn hierarchy_entries_keeping_selection(
    entries: &[WorldInspectionHierarchyRow],
    name_matches: &[WorldInspectionHierarchyRow],
    selected: &BTreeSet<NodeId>,
) -> Vec<WorldInspectionHierarchyRow> {
    let mut included = vec![false; entries.len()];
    let mut matched = 0usize;
    for (index, entry) in entries.iter().enumerate() {
        if selected.contains(&entry.entity)
            || name_matches.iter().any(|row| row.entity == entry.entity)
        {
            included[index] = true;
            matched += 1;
        }
    }
    if matched == 0 {
        return Vec::new();
    }
    let parent_indices = hierarchy_parent_indices(entries);
    for index in (0..entries.len()).rev() {
        if included[index] {
            if let Some(parent_index) = parent_indices[index] {
                included[parent_index] = true;
            }
        }
    }
    entries
        .iter()
        .zip(included)
        .filter_map(|(entry, included)| included.then(|| entry.clone()))
        .collect()
}

fn hierarchy_entries_matching_query(
    entries: &[WorldInspectionHierarchyRow],
    query: &str,
) -> Vec<WorldInspectionHierarchyRow> {
    let query = query.trim();
    if query.is_empty() {
        return entries.to_vec();
    }

    let normalized_query = if query.is_ascii() {
        Cow::Borrowed(query)
    } else {
        Cow::Owned(query.to_lowercase())
    };
    let mut included = None;
    let mut name_match_count = 0;

    for (index, entry) in entries.iter().enumerate() {
        let name_matches_query =
            hierarchy_name_matches_query(&entry.display_name, normalized_query.as_ref());
        if name_matches_query {
            included.get_or_insert_with(|| vec![false; entries.len()])[index] = true;
            name_match_count += 1;
        }
    }

    let Some(mut included) = included else {
        record_hierarchy_filter_metrics(entries.len(), 0, 0, 0);
        return Vec::new();
    };

    let parent_indices = hierarchy_parent_indices(entries);

    // A single reverse pass preserves every matching entry's ancestry in O(N).
    let mut ancestor_link_count = 0;
    for index in (0..entries.len()).rev() {
        if included[index] {
            if let Some(parent_index) = parent_indices[index] {
                if !included[parent_index] {
                    included[parent_index] = true;
                    ancestor_link_count += 1;
                }
            }
        }
    }

    let included_count = name_match_count + ancestor_link_count;
    let mut filtered_entries = Vec::with_capacity(included_count);
    for (entry, included) in entries.iter().zip(included) {
        if included {
            filtered_entries.push(entry.clone());
        }
    }

    record_hierarchy_filter_metrics(
        entries.len(),
        name_match_count,
        ancestor_link_count,
        filtered_entries.len(),
    );

    filtered_entries
}

fn record_hierarchy_filter_metrics(
    source_row_count: usize,
    name_match_count: usize,
    ancestor_link_count: usize,
    visible_row_count: usize,
) {
    // Aggregate once per projection so telemetry does not perturb the row traversal it measures.
    zircon_runtime::profile_counter!("editor", "hierarchy_filter_projection_invocation_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "hierarchy_filter_source_row_count",
        source_row_count
    );
    zircon_runtime::profile_counter!(
        "editor",
        "hierarchy_filter_name_match_count",
        name_match_count
    );
    zircon_runtime::profile_counter!(
        "editor",
        "hierarchy_filter_ancestor_link_count",
        ancestor_link_count
    );
    zircon_runtime::profile_counter!(
        "editor",
        "hierarchy_filter_visible_row_count",
        visible_row_count
    );
}

fn hierarchy_parent_indices(entries: &[WorldInspectionHierarchyRow]) -> Vec<Option<usize>> {
    let mut parent_indices = vec![None; entries.len()];
    let mut ancestor_indices = Vec::new();
    ancestor_indices.reserve(entries.len());

    for (index, entry) in entries.iter().enumerate() {
        while ancestor_indices
            .last()
            .is_some_and(|(_, depth)| *depth >= entry.depth)
        {
            ancestor_indices.pop();
        }
        parent_indices[index] = ancestor_indices.last().map(|(index, _)| *index);
        ancestor_indices.push((index, entry.depth));
    }

    parent_indices
}

fn hierarchy_name_matches_query(name: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    if query.is_ascii() {
        if name.is_ascii() {
            let query = query.as_bytes();
            return name
                .as_bytes()
                .windows(query.len())
                .any(|candidate| candidate.eq_ignore_ascii_case(query));
        }

        let lowered_name = name.to_lowercase();
        return lowered_name.char_indices().any(|(start, _)| {
            lowered_name
                .get(start..start + query.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(query))
        });
    }
    // Non-ASCII callers normalize the query once at the projection boundary.
    name.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "tests/hierarchy_filter.rs"]
mod tests;
