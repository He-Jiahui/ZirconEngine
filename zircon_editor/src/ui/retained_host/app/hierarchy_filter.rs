use zircon_runtime::scene::WorldInspectionHierarchyRow;

use super::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::workbench::snapshot::SceneEntries;

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
        Some(entries.with_hierarchy_rows(hierarchy_entries_matching_query(entries, query)))
    }

    pub(super) fn set_hierarchy_filter_query(&mut self, query: &str) {
        if self.hierarchy_filter_query == query {
            return;
        }
        self.hierarchy_filter_query = query.to_string();
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

    let query = query.to_lowercase();
    let mut included = vec![false; entries.len()];
    let parent_indices = hierarchy_parent_indices(entries);
    let mut name_match_count = 0;

    for (index, entry) in entries.iter().enumerate() {
        let name_matches_query = hierarchy_name_matches_query(&entry.display_name, &query);
        included[index] = name_matches_query;
        name_match_count += usize::from(name_matches_query);
    }

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

    let filtered_entries = entries
        .iter()
        .zip(included)
        .filter_map(|(entry, included)| included.then(|| entry.clone()))
        .collect::<Vec<_>>();

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

fn hierarchy_name_matches_query(name: &str, normalized_query: &str) -> bool {
    if normalized_query.is_empty() {
        return true;
    }
    if normalized_query.is_ascii() && name.is_ascii() {
        let query = normalized_query.as_bytes();
        return name
            .as_bytes()
            .windows(query.len())
            .any(|candidate| candidate.eq_ignore_ascii_case(query));
    }
    name.to_lowercase().contains(normalized_query)
}

#[cfg(test)]
mod tests {
    use super::{
        hierarchy_entries_matching_query, hierarchy_name_matches_query, hierarchy_parent_indices,
    };
    use zircon_runtime::scene::WorldInspectionHierarchyRow;

    use crate::ui::workbench::snapshot::{SceneEntries, SceneEntry};

    fn entry(id: u64, name: &str, depth: usize) -> WorldInspectionHierarchyRow {
        WorldInspectionHierarchyRow {
            entity: id,
            parent: None,
            depth: depth as u32,
            display_name: name.to_string(),
            kind: "Entity".to_string(),
            subtree_hash: 0,
            active_in_hierarchy: true,
            has_children: false,
        }
    }

    #[test]
    fn hierarchy_filter_keeps_matching_nodes_and_their_ancestor_path() {
        let entries = vec![
            entry(1, "Environment", 0),
            entry(2, "Camera", 1),
            entry(3, "Lighting", 0),
            entry(4, "Sun", 1),
            entry(5, "Bounce Light", 2),
        ];

        let filtered = hierarchy_entries_matching_query(&entries, "bounce");

        assert_eq!(
            filtered
                .into_iter()
                .map(|entry| entry.entity)
                .collect::<Vec<_>>(),
            vec![3, 4, 5]
        );
    }

    #[test]
    fn hierarchy_filter_is_case_insensitive_and_returns_all_entries_for_blank_queries() {
        let entries = vec![entry(1, "Camera", 0), entry(2, "Cube", 0)];

        assert_eq!(
            hierarchy_entries_matching_query(&entries, "CAM")[0].entity,
            1
        );
        assert_eq!(hierarchy_entries_matching_query(&entries, "   "), entries);
    }

    #[test]
    fn hierarchy_filter_matches_ascii_queries_without_normalizing_each_node_name() {
        assert!(hierarchy_name_matches_query("Main Camera", "camera"));
        assert!(hierarchy_name_matches_query("Directional LIGHT", "light"));
        assert!(!hierarchy_name_matches_query("Main Camera", "light"));
    }

    #[test]
    fn hierarchy_filter_keeps_unicode_case_insensitive_matching() {
        assert!(hierarchy_name_matches_query("Café Light", "café"));
    }

    #[test]
    fn hierarchy_filter_matches_ascii_queries_against_unicode_lowercase_mappings() {
        assert!(hierarchy_name_matches_query("\u{212A}", "k"));
        assert!(hierarchy_name_matches_query("\u{0130}", "i"));
    }

    #[test]
    fn hierarchy_filter_keeps_its_profile_scope_at_the_projection_boundary() {
        let source = include_str!("hierarchy_filter.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(
            production.contains("profile_scope!(\"editor\", \"hierarchy\", \"filter_projection\")")
        );
    }

    #[test]
    fn hierarchy_filter_profiles_the_work_needed_for_scale_attribution() {
        let source = include_str!("hierarchy_filter.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        for counter in [
            "hierarchy_filter_projection_invocation_count",
            "hierarchy_filter_source_row_count",
            "hierarchy_filter_name_match_count",
            "hierarchy_filter_ancestor_link_count",
            "hierarchy_filter_visible_row_count",
        ] {
            assert!(
                production.contains(&format!("\"{counter}\"")),
                "hierarchy filter profiling must emit {counter}"
            );
        }
    }

    #[test]
    fn hierarchy_filter_records_blank_query_as_full_visibility() {
        let source = include_str!("hierarchy_filter.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let wrapper = production
            .split("pub(super) fn filtered_hierarchy_entries")
            .nth(1)
            .and_then(|source| {
                source
                    .split("pub(super) fn set_hierarchy_filter_query")
                    .next()
            })
            .expect("filtered hierarchy entries wrapper");
        let projection = production
            .split("fn hierarchy_entries_matching_query")
            .nth(1)
            .and_then(|source| source.split("fn record_hierarchy_filter_metrics").next())
            .expect("hierarchy projection helper");

        assert!(
            wrapper.contains("profile_scope!(\"editor\", \"hierarchy\", \"filter_projection\")"),
            "the product wrapper must own the filter-projection span"
        );
        assert!(
            wrapper.contains(
                "record_hierarchy_filter_metrics(entries.len(), entries.len(), 0, entries.len());"
            ),
            "blank hierarchy queries must report full visibility instead of omitting trace counters"
        );
        assert!(
            wrapper.contains("return None;"),
            "blank hierarchy queries must retain the no-materialization result"
        );
        assert!(
            !projection
                .contains("profile_scope!(\"editor\", \"hierarchy\", \"filter_projection\")"),
            "the algorithm helper must not create a duplicate projection span"
        );
    }

    #[test]
    fn hierarchy_filter_handles_a_large_flat_projection() {
        let entries = (0..5_000)
            .map(|id| entry(id, &format!("Node {id}"), 0))
            .collect::<Vec<_>>();

        let filtered = hierarchy_entries_matching_query(&entries, "Node 4999");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].entity, 4_999);
    }

    #[test]
    fn hierarchy_filter_keeps_the_full_deep_ancestor_path() {
        let entries = (0..5_000)
            .map(|index| {
                entry(
                    index as u64,
                    if index == 4_999 { "Needle" } else { "Node" },
                    index,
                )
            })
            .collect::<Vec<_>>();

        let filtered = hierarchy_entries_matching_query(&entries, "needle");

        assert_eq!(filtered.len(), 5_000);
        assert_eq!(filtered.first().map(|entry| entry.entity), Some(0));
        assert_eq!(filtered.last().map(|entry| entry.entity), Some(4_999));
    }

    #[test]
    fn hierarchy_filter_indexes_each_deep_entry_once() {
        let entries = (0..5_000)
            .map(|index| entry(index as u64, "Node", index))
            .collect::<Vec<_>>();

        let parents = hierarchy_parent_indices(&entries);

        assert_eq!(parents.len(), entries.len());
        assert_eq!(parents.first().copied(), Some(None));
        assert_eq!(parents.get(1).copied(), Some(Some(0)));
        assert_eq!(parents.last().copied(), Some(Some(4_998)));
    }

    #[test]
    fn hierarchy_filter_keeps_all_matches_in_a_deep_projection() {
        let entries = (0..5_000)
            .map(|index| entry(index as u64, "Node", index))
            .collect::<Vec<_>>();

        let filtered = hierarchy_entries_matching_query(&entries, "node");

        assert_eq!(filtered.len(), entries.len());
        assert_eq!(filtered.first().map(|entry| entry.entity), Some(0));
        assert_eq!(filtered.last().map(|entry| entry.entity), Some(4_999));
    }

    #[test]
    fn hierarchy_filter_preserves_the_editor_selection_overlay() {
        let entries = SceneEntries::from_entries(
            [
                SceneEntry {
                    id: 1,
                    name: "Camera".to_string(),
                    depth: 0,
                },
                SceneEntry {
                    id: 2,
                    name: "Cube".to_string(),
                    depth: 0,
                },
            ],
            [2],
        );

        let filtered =
            entries.with_hierarchy_rows(hierarchy_entries_matching_query(&entries, "cube"));

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].entity, 2);
        assert!(filtered.is_selected(2));
    }
}
