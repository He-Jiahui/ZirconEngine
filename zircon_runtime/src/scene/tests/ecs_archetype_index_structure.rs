#[test]
fn archetype_index_owns_complete_row_transitions_without_a_metadata_move_facade() {
    let index = include_str!("../ecs/archetype/index.rs");
    let record = include_str!("../ecs/archetype/record.rs");
    let archetype_mod = include_str!("../ecs/archetype/mod.rs");

    assert!(record.contains("table: ArchetypeTable"));
    assert!(index.contains("pub(crate) fn preflight_row("));
    assert!(index.contains("pub(crate) fn validate_transition("));
    assert!(index.contains("pub(crate) fn take_entity_row("));
    assert!(index.contains("pub(crate) fn append_preflighted_row("));
    assert!(index.contains("pub(crate) fn remove_entity_at("));
    assert!(!index.contains("pub fn move_entity("));
    assert!(!archetype_mod.contains("move_result"));
    assert!(!archetype_mod.contains("ArchetypeMove"));
}

#[test]
fn archetype_index_query_matching_uses_the_shortest_component_posting_list() {
    let source = include_str!("../ecs/archetype/index.rs");
    let matching = source
        .split("pub fn matching_archetypes(")
        .nth(1)
        .and_then(|body| body.split("fn shortest_required_archetype_ids").next())
        .expect("matching archetypes body");
    let shortest = source
        .split("fn shortest_required_archetype_ids(&self, required: &[ComponentId])")
        .nth(1)
        .and_then(|body| body.split("fn archetype_matches_required_without").next())
        .expect("shortest posting-list body");

    assert!(matching.contains("self.shortest_required_archetype_ids(required)"));
    assert!(matching.contains("Vec::with_capacity(ids.len())"));
    assert!(matching.contains("self.archetype_matches_required_without"));
    assert!(shortest.contains("for component_id in required"));
    assert!(shortest.contains("match self.by_component.get(component_id)"));
    assert!(shortest.contains("let candidate_len = ids.len();"));
    assert!(!matching.contains("ids.clone()"));
}

#[test]
fn archetype_index_dense_access_accepts_precompiled_column_slots() {
    let source = include_str!("../ecs/archetype/index.rs");

    for method in [
        "pub(crate) fn get_by_slot<T>(",
        "pub(crate) fn component_ticks_by_slot(",
        "pub(crate) fn get_mut_at_tick_by_slot<T>(",
        "pub(crate) fn get_mut_with_ticks_by_slot<T>(",
    ] {
        assert!(
            source.contains(method),
            "missing compiled-slot method `{method}`"
        );
    }
    assert!(source.contains("record.get_by_slot::<T>(column_slot, row)"));
    assert!(source.contains("record.get_mut_at_tick_by_slot::<T>(column_slot, row, tick)"));
}
