use super::*;

#[test]
fn cached_iterators_borrow_archetype_plans_and_keep_only_current_row_scratch() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    let iter = read_source(&query_root.join("query_iter.rs"));
    let many = read_source(&query_root.join("query_many_iter.rs"));
    let direct = read_source(&query_root.join("cached_query_iter.rs"));
    let state = read_source(&query_root.join("query_state/read_only_cached.rs"));

    assert!(iter.contains("Cached(StableQueryLocationIter<'world>)"));
    assert!(iter.contains("plans: &'state [CachedArchetypePlan]"));
    assert!(iter.contains("component_locations: Vec<ComponentStorageLocation>"));
    assert!(iter.contains("plan.write_component_locations("));
    assert!(state.contains("QueryIter::new_cached_plans("));
    assert!(state.contains("&self.cached_archetype_plans"));

    assert!(many.contains("plans: &'state [CachedArchetypePlan]"));
    assert!(many.contains("entities: I"));
    assert!(many.contains("project_entity_from_plans("));
    assert!(direct.contains("plans: &'state [CachedArchetypePlan]"));
    assert!(direct.contains("component_locations: Vec<ComponentStorageLocation>"));
    for source in [&iter, &many, &direct, &state] {
        for forbidden in [
            "cached_entity_indices",
            "cached_locations",
            "cached_component_location_offsets",
        ] {
            assert!(
                !source.contains(forbidden),
                "cached iterator retained `{forbidden}`"
            );
        }
    }
}

#[test]
fn system_query_cache_refresh_requires_an_exclusive_run_query_borrow() {
    let query = read_source(&manifest_dir().join("src/scene/ecs/system/query.rs"));

    for signature in [
        "pub fn iter(",
        "pub fn iter_cached(",
        "pub fn iter_many_cached<",
        "pub fn iter_many_unique_cached<",
        "pub fn iter_combinations<",
        "pub fn iter_combinations_cached<",
        "pub fn single(",
        "pub fn single_cached(",
        "pub fn get_cached(",
        "pub fn get_many_cached<",
        "pub fn get_many_unique_cached<",
        "pub fn is_empty(",
        "pub fn count(",
        "pub fn is_empty_cached(",
        "pub fn count_cached(",
        "pub fn contains_cached(",
        "pub fn iter_cached_direct(",
        "pub fn single_cached_direct(",
        "pub fn iter_many_cached_direct<",
        "pub fn iter_many_unique_cached_direct<",
        "pub fn get_cached_direct(",
        "pub fn get_many_cached_direct<",
        "pub fn get_many_unique_cached_direct<",
        "pub fn is_empty_cached_direct(",
        "pub fn count_cached_direct(",
        "pub fn contains_cached_direct(",
        "pub fn get_mut(",
        "pub fn single_mut(",
        "pub fn iter_mut(",
        "pub fn get_many_mut<",
        "pub fn get_many_unique_mut<",
        "pub fn iter_many_mut<",
        "pub fn iter_many_unique_mut<",
        "pub fn iter_combinations_mut<",
        "pub fn for_each_mut(",
    ] {
        let method = query
            .split(signature)
            .nth(1)
            .and_then(|source| source.split("\n    pub fn ").next())
            .expect("system query API declaration must exist");
        assert!(
            method.contains("&mut self"),
            "cache or mutable system query entry must require `&mut self`: {signature}"
        );
    }

    for signature in [
        "pub fn iter_many<",
        "pub fn iter_many_unique<",
        "pub fn get(",
        "pub fn get_many<",
        "pub fn get_many_unique<",
        "pub fn contains(",
    ] {
        let method = query
            .split(signature)
            .nth(1)
            .and_then(|source| source.split("\n    pub fn ").next())
            .expect("uncached system query API declaration must exist");
        assert!(
            method.contains("&self"),
            "uncached system query entry must remain shared: {signature}"
        );
    }

    let iter_many = query
        .split("pub fn iter_many<")
        .nth(1)
        .and_then(|source| source.split("\n    pub fn ").next())
        .expect("uncached many iterator must exist");
    assert!(iter_many.contains("state.iter_many_with_ticks(world, entities, self.ticks)"));
}

#[test]
fn cached_point_queries_resolve_entity_location_then_apply_the_compiled_plan() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    let plan = read_source(&query_root.join("query_state/archetype_plan.rs"));
    let cached = read_source(&query_root.join("query_state/read_only_cached.rs"));

    assert!(plan.contains("let stable_location = world.internal_entity_location(entity)?;"));
    assert!(plan.contains("find_cached_archetype_plan(plans"));
    assert!(plan.contains("plan.write_component_locations(world, stable_location"));
    assert!(cached.contains("self.project_entity(world, entity, &mut component_locations)"));
    assert!(cached.contains("D::fetch_with_component_locations("));
}
