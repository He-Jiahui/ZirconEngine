use super::*;

#[test]
fn archetype_index_matching_reuses_sorted_component_index_without_per_query_resort() {
    let signature_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("archetype")
            .join("signature.rs"),
    );
    let archetype_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("archetype")
            .join("index.rs"),
    );
    let matching_archetypes = archetype_text
        .split("pub fn matching_archetypes(")
        .nth(1)
        .and_then(|text| text.split("fn shortest_required_archetype_ids").next())
        .expect("read matching_archetypes implementation");
    let signature_indexer = archetype_text
        .split("fn index_signature_components(")
        .nth(1)
        .and_then(|text| text.split("impl Default for ArchetypeIndex").next())
        .expect("read index_signature_components implementation");

    assert!(
        signature_indexer.contains("insert_archetype_id(ids, id);"),
        "ArchetypeIndex must centralize sorted per-component index insertion"
    );
    assert!(
        archetype_text
            .contains("fn insert_archetype_id(ids: &mut Vec<ArchetypeId>, id: ArchetypeId)")
            && archetype_text.contains("if let Err(index) = ids.binary_search(&id)")
            && archetype_text.contains("ids.insert(index, id);"),
        "per-component archetype index lists must stay sorted and unique as they are built"
    );
    assert!(
        !signature_indexer.contains("ids.contains(&id)")
            && !signature_indexer.contains("ids.sort_unstable();"),
        "signature indexing must not use linear contains plus full-list sort after each insertion"
    );
    assert!(
        !matching_archetypes.contains("candidates.sort_unstable();")
            && !matching_archetypes.contains("candidates.dedup();"),
        "matching_archetypes must rely on sorted unique index ownership instead of resorting each query"
    );
    assert!(
        matching_archetypes.contains("let mut matches = Vec::with_capacity(ids.len());")
            && matching_archetypes
                .contains("let mut matches = Vec::with_capacity(self.records.len());")
            && matching_archetypes.contains("for id in ids")
            && matching_archetypes.contains("for record in &self.records")
            && !matching_archetypes.contains("all_archetype_ids(&self.records)")
            && !matching_archetypes
                .contains("self.records.iter().map(ArchetypeRecord::id).collect()"),
        "matching_archetypes must size candidate result vectors directly instead of relying on collect growth"
    );
    assert!(
        !matching_archetypes.contains("candidates.retain(|id|")
            && !archetype_text.contains("fn all_archetype_ids"),
        "matching_archetypes should avoid a candidate clone/retain helper after direct projection"
    );
    assert!(
        signature_text.contains("fn normalize_components(mut components: Vec<ComponentId>)")
            && signature_text.contains("if components.len() > 1")
            && signature_text.contains("components.sort_unstable();")
            && signature_text.contains("components.dedup();"),
        "ArchetypeSignature normalization must skip sort/dedup for trivial component lists while preserving normalized multi-component signatures"
    );
}

#[test]
fn query_access_conflicts_with_uses_allocation_free_boolean_path() {
    let query_access_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("query")
            .join("query_access.rs"),
    );

    assert!(
        query_access_text.contains("pub fn conflicts_with(&self, other: &Self) -> bool"),
        "QueryAccess must keep a boolean conflict-check entry point"
    );
    assert!(
        query_access_text.contains("!self.has_disjoint_filter(other)")
            && query_access_text
                .contains("sorted_component_slices_intersect(&self.writes, &other.reads)")
            && query_access_text
                .contains("sorted_component_slices_intersect(&self.reads, &other.writes)"),
        "QueryAccess::conflicts_with must check sorted read/write intersections directly"
    );
    assert!(
        !query_access_text
            .contains("sorted_component_slices_intersect(&self.writes, &other.writes)"),
        "QueryAccess::conflicts_with must not repeat write/write checks already covered by write-implies-read"
    );
    assert!(
        !query_access_text.contains("!self.conflicting_components_with(other).is_empty()"),
        "QueryAccess::conflicts_with must not allocate the detailed conflict Vec for boolean checks"
    );
}
