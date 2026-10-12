use std::borrow::Cow;

use zircon_runtime::scene::WorldInspectionHierarchyRow;

use super::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::workbench::snapshot::SceneEntries;

impl SceneEntries {
    pub(crate) fn filtered_by_hierarchy_query(&self, query: &str) -> Self {
        let query = query.trim();
        if query.is_empty() {
            return self.clone();
        }
        self.with_hierarchy_rows(hierarchy_entries_matching_query(self, query))
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
