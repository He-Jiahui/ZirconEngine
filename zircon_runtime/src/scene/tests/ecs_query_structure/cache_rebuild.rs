use super::*;

#[test]
fn query_state_cache_rebuild_uses_access_reads_without_per_rebuild_merge() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let root_text = read_source(&query_root.join("query_state").join("mod.rs"));
    let cache_text = read_source(&query_root.join("query_state").join("cache.rs"));
    let access_text = read_source(&query_root.join("query_access.rs"));
    let data_text = read_source(&query_root.join("query_data.rs"));
    let filter_text = read_source(&query_root.join("query_filter.rs"));
    let cached_iter_text = read_source(&query_root.join("cached_query_iter.rs"));
    let world_query_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("query.rs"),
    );

    assert!(
        access_text.contains("insert_id(&mut self.reads, component_id);")
            && access_text.contains("insert_id(&mut self.writes, component_id);"),
        "QueryAccess::add_write must keep written component IDs in reads so cache rebuilds can use one access list"
    );
    assert!(
        access_text.contains("if let Err(index) = ids.binary_search(&component_id)")
            && access_text.contains("ids.insert(index, component_id);"),
        "QueryAccess::insert_id must preserve sorted access IDs by binary-position insertion"
    );
    assert!(
        !access_text.contains("ids.sort_unstable();"),
        "QueryAccess::insert_id must not push then re-sort each access insertion"
    );
    assert!(
        cache_text.contains(
            "component_storage_locations_for_internal("
        ),
        "QueryState cache rebuilds must reuse access.reads() for cached component storage locations"
    );
    assert!(
        cache_text.contains("self.access.reads(),")
            && cache_text.contains("&mut component_locations"),
        "QueryState cache rebuilds must fill a reusable component-location scratch Vec from access.reads()"
    );
    assert!(
        cache_text.contains("let matched_archetypes = world.matching_query_archetypes(&self.access);")
            && cache_text.contains(
                "let candidate_count = world.matching_query_archetype_entity_count(&matched_archetypes);",
            )
            && cache_text.contains(
                "world.visit_entity_locations_matching_archetypes(&matched_archetypes, |location| {",
            )
            && !cache_text.contains("entity_locations_matching_query_archetypes")
            && !cache_text.contains("candidate_locations"),
        "QueryState cache rebuilds must visit matching world locations directly instead of receiving a temporary candidate-location Vec"
    );
    let world_archetype_lookup = world_query_text
        .split("pub(crate) fn matching_query_archetypes")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn matching_query_archetype_entity_count")
                .next()
        })
        .expect("read world query archetype lookup");
    assert!(
        world_archetype_lookup.contains(".matching_archetypes(access.with(), access.without())"),
        "World query cache rebuilds must keep matched-archetype lookup owned by the world query layer"
    );
    let world_candidate_count = world_query_text
        .split("pub(crate) fn matching_query_archetype_entity_count")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn visit_entity_locations_matching_archetypes")
                .next()
        })
        .expect("read world query archetype candidate count");
    assert!(
        world_candidate_count.contains("let mut count = 0;")
            && world_candidate_count.contains("for archetype in archetypes")
            && world_candidate_count.contains("self.archetype_index.entities(*archetype)")
            && world_candidate_count.contains("count += entities.len();")
            && world_candidate_count.contains("count"),
        "World query cache candidate count must derive an exact reserve bound from matched archetype entity lists"
    );
    let world_candidate_visit = world_query_text
        .split("pub(crate) fn visit_entity_locations_matching_archetypes")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn component_storage_locations_for_internal")
                .next()
        })
        .expect("read world query archetype candidate visitor");
    assert!(
        world_candidate_visit.contains("if archetypes.is_empty()")
            && world_candidate_visit.contains("return;")
            && world_candidate_visit.contains("for entity in self.entities.iter().copied()")
            && world_candidate_visit.contains("self.internal_entity_location(entity)")
            && world_candidate_visit.contains(".binary_search(&location.location.archetype_id)")
            && world_candidate_visit.contains("visitor(location);")
            && !world_candidate_visit.contains("Vec::with_capacity")
            && !world_candidate_visit.contains("locations.push")
            && !world_candidate_visit.contains("return (archetypes, Vec::new())")
            && !world_candidate_visit.contains(".filter_map(")
            && !world_candidate_visit.contains(".collect()"),
        "World query cache candidate visitor must preserve stable entity-order traversal without constructing a temporary candidate-location Vec"
    );
    let world_component_locations = world_query_text
        .split("pub(crate) fn component_storage_locations_for_internal")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn component_ref_with_ticks_at_location")
                .next()
        })
        .expect("read world query component-location scratch fill");
    assert!(
        world_component_locations.contains("output.clear();")
            && world_component_locations.contains("let component_count = component_ids.len();")
            && world_component_locations.contains("if component_count == 0")
            && world_component_locations.contains("output.reserve(component_count);")
            && world_component_locations.contains("for component_id in component_ids")
            && world_component_locations
                .contains("self.component_storage.location(*component_id, internal)")
            && world_component_locations.contains("output.push(location);")
            && !world_component_locations.contains(".filter_map(")
            && !world_component_locations.contains("output.extend("),
        "World component-location scratch fill must clear, reserve from access-read count, and push storage locations directly without iterator filter_map/extend growth"
    );
    assert!(
        cache_text.contains("let candidate_count = world.matching_query_archetype_entity_count(&matched_archetypes);")
            && cache_text.contains("let component_count = self.access.reads().len();")
            && cache_text.contains("self.cached_entities.reserve(candidate_count);")
            && cache_text.contains("self.cached_entity_indices.reserve(candidate_count);")
            && cache_text.contains("self.cached_locations.reserve(candidate_count);")
            && cache_text.contains(
                "self.cached_component_location_offsets\n            .reserve(candidate_count);",
            )
            && cache_text.contains(
                "self.cached_component_locations\n            .reserve(candidate_count.saturating_mul(component_count));",
            ),
        "QueryState cache rebuilds must reserve candidate-sized entity/location caches before repopulating them"
    );
    assert!(
        !cache_text.contains("cached_component_ids"),
        "QueryState::update_cache must not rebuild a temporary cached_component_ids Vec per cache revision"
    );
    assert!(
        root_text.contains("cached_component_locations: Vec<ComponentStorageLocation>")
            && root_text.contains("cached_component_location_offsets: Vec<usize>")
            && !root_text.contains("Vec<Vec<ComponentStorageLocation>>"),
        "QueryState must keep component-location cache storage flat instead of retaining one Vec per entity"
    );
    assert!(
        !cache_text.contains("self.access.writes().iter()"),
        "QueryState::update_cache must not rescan writes when QueryAccess already mirrors writes into reads"
    );

    for (label, text) in [
        ("query_data.rs", data_text.as_str()),
        ("query_filter.rs", filter_text.as_str()),
        ("cached_query_iter.rs", cached_iter_text.as_str()),
    ] {
        assert!(
            text.contains(".binary_search_by_key(&component_id, |location| location.component_id)"),
            "{label} must use binary component-location lookup over QueryAccess-sorted cached locations"
        );
        assert!(
            !text.contains(".find(|location| location.component_id == component_id)"),
            "{label} must not return to linear component-location scans on cached query hot paths"
        );
    }
}
