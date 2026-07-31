use super::*;

#[test]
fn cached_combinations_trust_query_state_data_membership() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let read_combo_text = read_source(&query_root.join("query_combinations_iter.rs"));
    let mut_combo_text = read_source(&query_root.join("query_combinations_mut_iter.rs"));
    let read_only_text = read_source(&query_root.join("query_state").join("read_only.rs"));
    let read_only_cached_text =
        read_source(&query_root.join("query_state").join("read_only_cached.rs"));
    let mutable_text = read_source(&query_root.join("query_state").join("mutable.rs"));

    let cached_read_constructor = read_combo_text
        .split("pub(crate) fn new_from_cached_entities")
        .nth(1)
        .and_then(|text| text.split("fn fetch_current").next())
        .expect("read cached combination constructor");
    let owned_read_constructor = read_combo_text
        .split("pub(crate) fn new(")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn new_from_cached_entities").next())
        .expect("read owned read-only combination constructor");
    let cached_mut_constructor = mut_combo_text
        .split("pub(crate) fn new_from_cached_entities")
        .nth(1)
        .and_then(|text| text.split("pub fn fetch_next").next())
        .expect("mutable cached combination constructor");

    for (label, constructor) in [
        ("read cached combinations", cached_read_constructor),
        ("mutable cached combinations", cached_mut_constructor),
    ] {
        assert!(
            constructor.contains("cached_query_component_locations(")
                && constructor.contains("F::matches_component_locations("),
            "{label} must keep dynamic filter checks on cached component-location slices"
        );
        assert!(
            !constructor.contains("F::matches(world, *entity, ticks)")
                && !constructor.contains("F::matches(world, entity, ticks)"),
            "{label} must not return cached dynamic filters to world-level lookups"
        );
        assert!(
            !constructor.contains("D::matches_data"),
            "{label} must not repeat QueryData structural checks already owned by QueryState cache rebuilds"
        );
    }

    assert!(
        read_only_cached_text.contains("QueryCombinationIter::new_from_cached_entities("),
        "read-only cached combinations must use the cache-aware constructor"
    );
    assert!(
        read_only_cached_text.contains("&self.cached_locations")
            && read_only_cached_text.contains("&self.cached_component_locations")
            && read_only_cached_text.contains("&self.cached_component_location_offsets"),
        "read-only cached combinations must pass stable and component-location caches into the iterator"
    );
    assert!(
        read_combo_text.contains("enum QueryCombinationCandidates<'state>")
            && read_combo_text.contains("cache_indices: Vec<usize>")
            && read_combo_text.contains("entities: &'state [EntityId]")
            && read_combo_text.contains("stable_locations: &'state [StableEntityLocation]")
            && read_combo_text.contains("component_locations: &'state [ComponentStorageLocation]"),
        "read-only cached combinations must borrow QueryState cache slices and store only matched cache slots"
    );
    assert!(
        cached_read_constructor.contains("let mut cache_indices = Vec::with_capacity(entities.len());")
            && cached_read_constructor.contains("cache_indices.push(index)")
            && !cached_read_constructor.contains("matched_entities")
            && !cached_read_constructor.contains("matched_stable_locations")
            && !cached_read_constructor.contains("matched_component_locations")
            && !cached_read_constructor.contains("extend_from_slice"),
        "read-only cached combinations must not copy matched entities, stable locations, or component-location buffers"
    );
    assert!(
        read_combo_text.contains("let cache_index = cache_indices[candidate_index];")
            && read_combo_text.contains("D::fetch_with_component_locations("),
        "read-only cached combinations must fetch from the borrowed cache slot"
    );
    assert!(
        owned_read_constructor.contains("entities: &[EntityId]")
            && owned_read_constructor.contains("if K > entities.len()")
            && owned_read_constructor.contains("return Self::empty(world, ticks);")
            && owned_read_constructor.contains("let mut matched_entities = Vec::new();")
            && owned_read_constructor.contains("for entity in entities.iter().copied()")
            && owned_read_constructor
                .contains("read_only_combination_candidate_matches::<D, F>(world, entity, ticks)")
            && owned_read_constructor.contains("matched_entities.push(entity)")
            && owned_read_constructor.contains("if matched_entities.len() < K")
            && !owned_read_constructor.contains("candidate_count")
            && !owned_read_constructor.contains(".collect::<Vec<_>>()")
            && !owned_read_constructor.contains("filter(|entity|"),
        "uncached read-only combinations must skip impossible group sizes and match each candidate once"
    );
    assert!(
        read_combo_text.contains("fn empty(world: &'world World, ticks: ChangeTickWindow) -> Self")
            && read_combo_text.contains("candidates: QueryCombinationCandidates::Owned(Vec::new())")
            && read_combo_text.contains("remaining: 0"),
        "read-only combinations must share an explicit empty iterator constructor for impossible group sizes"
    );
    let combination_count = read_combo_text
        .split("pub(crate) fn combination_count(")
        .nth(1)
        .expect("read combination count helper");
    assert!(
        combination_count.contains("let mut count = 1_usize;")
            && combination_count.contains("while denominator <= group_size")
            && combination_count.contains("count.checked_mul(numerator)")
            && combination_count.contains("return usize::MAX;")
            && !combination_count.contains(".zip(numerator)")
            && !combination_count.contains(".try_fold("),
        "combination count must use an explicit checked loop instead of iterator setup on every constructor path"
    );
    assert!(
        cached_read_constructor.contains("if K > entities.len()")
            && cached_read_constructor.contains("return Self::empty(world, ticks);")
            && cached_read_constructor.contains("if cache_indices.len() < K"),
        "cached read-only combinations must avoid building or enumerating impossible group sizes"
    );
    assert!(
        mut_combo_text.contains("struct QueryCombinationMutCandidates<'state>")
            && mut_combo_text.contains("cache_indices: Vec<usize>")
            && mut_combo_text.contains("entities: &'state [EntityId]"),
        "mutable cached combinations must borrow QueryState entity slices and store only matched cache slots"
    );
    assert!(
        !mut_combo_text.contains("pub(crate) fn new<EntityList>")
            && !mut_combo_text.contains("QueryCombinationMutCandidates::Owned")
            && !mut_combo_text.contains("D::matches_data(world, *entity)"),
        "mutable combinations must not keep the old uncached full-world constructor after QueryState cache refresh became the authoritative entry path"
    );
    assert!(
        cached_mut_constructor
            .contains("if K > entities.len()")
            && cached_mut_constructor.contains("return Self::empty(world, ticks);")
            && cached_mut_constructor.contains("let mut cache_indices = Vec::with_capacity(entities.len());")
            && cached_mut_constructor.contains("cache_indices.push(index)")
            && cached_mut_constructor.contains("if cache_indices.len() < K")
            && !cached_mut_constructor.contains("then_some(entity)")
            && !cached_mut_constructor.contains(".collect::<Vec<_>>()"),
        "mutable cached combinations must skip impossible group sizes and not copy matched entities into a temporary Vec"
    );
    assert!(
        mut_combo_text
            .contains("fn empty(world: &'world mut World, ticks: ChangeTickWindow) -> Self")
            && mut_combo_text.contains("entities: &[]")
            && mut_combo_text.contains("cache_indices: Vec::new()")
            && mut_combo_text.contains("remaining: 0"),
        "mutable combinations must share an explicit empty iterator constructor for impossible group sizes"
    );
    assert!(
        mut_combo_text.contains("self.candidates.entity(self.indices[index])")
            && mut_combo_text.contains("let entity_count = self.candidates.len();"),
        "mutable cached combinations must enumerate combinations over the compact cache-slot candidate list"
    );
    assert!(
        read_only_text
            .contains("QueryCombinationIter::new(world, world.entity_ids_for_query(), ticks)"),
        "uncached read-only combinations must keep full world-entity query validation"
    );
    assert!(
        mutable_text.contains("QueryCombinationMutIter::new_from_cached_entities("),
        "mutable cached combinations must use the cache-aware constructor"
    );
    assert!(
        mutable_text.contains("&self.cached_entities")
            && mutable_text.contains("&self.cached_component_locations")
            && mutable_text.contains("&self.cached_component_location_offsets"),
        "mutable cached combinations must pass entity and component-location caches into the iterator"
    );
}
