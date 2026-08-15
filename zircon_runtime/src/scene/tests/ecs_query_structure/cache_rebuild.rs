use super::*;

#[test]
fn query_state_cache_compiles_one_binding_plan_per_matching_archetype() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    let cache = read_source(&query_root.join("query_state/cache.rs"));
    let plan = read_source(&query_root.join("query_state/archetype_plan.rs"));

    assert!(cache.contains("world.matching_query_archetypes(&self.access)"));
    assert!(cache.contains(".map(|archetype| self.compile_archetype_plan(world, archetype))"));
    assert!(cache.contains("Vec::with_capacity(self.access.reads().len())"));
    assert!(cache.contains("for component_id in self.access.reads().iter().copied()"));
    assert!(cache.contains("query_archetype_column_slot(archetype, component_id)"));
    assert!(cache.contains("QueryComponentBinding::SparseSet {"));
    assert!(plan.contains("QueryComponentBinding::Table"));
    assert!(plan.contains("column_slot"));
    assert!(
        plan.contains("rust_type_id"),
        "compiled bindings must retain the component Rust type selected during cache compilation"
    );
    let location =
        read_source(&manifest_dir().join("src/scene/ecs/storage/component_storage/location.rs"));
    assert!(
        location.contains("pub rust_type_id: Option<TypeId>"),
        "the per-row projection must carry the binding type token without a runtime registry probe"
    );
    assert!(!cache.contains("component_storage_locations_for_internal"));
    assert!(!cache.contains("visit_entity_locations_matching_archetypes"));
}

#[test]
fn query_state_membership_changes_refresh_local_plans_without_global_projection_rebuild() {
    let cache = read_source(&manifest_dir().join("src/scene/ecs/query/query_state/cache.rs"));

    assert!(cache.contains("if self.cached_archetype_generation == archetype_generation"));
    assert!(cache.contains("world.matching_query_archetypes_from("));
    assert!(cache.contains("if new_matches.is_empty()"));
    assert!(cache.contains("self.cached_archetype_plans.extend("));
    assert!(cache.contains("self.refresh_plan_memberships(world);"));
    assert!(cache.contains("for plan in &mut self.cached_archetype_plans"));
    assert!(cache.contains("plan.refresh_membership_generation(generation);"));
    assert!(cache.contains("world.query_archetype_entity_count(archetype)"));
    assert!(!cache.contains("query_cache_revision"));
    assert!(!cache.contains("cached_entities"));
    assert!(!cache.contains("cached_component_locations"));
}

#[test]
fn compiled_location_lookup_uses_bound_rust_type_without_registry_probes() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    for relative in ["query_data.rs", "query_filter.rs", "cached_query_iter.rs"] {
        let source = read_source(&query_root.join(relative));
        assert!(source.contains("TypeId::of::<T>()"));
        assert!(!source.contains("registered_component_id::<T>()"));
        assert!(!source.contains("binary_search_by_key(&component_id"));
    }

    let query_data = read_source(&query_root.join("query_data.rs"));
    let query_filter = read_source(&query_root.join("query_filter.rs"));
    assert!(
        !query_data.contains("Self::matches_data(world, entity)"),
        "cached query data must require an explicit location-aware match hook"
    );
    assert!(
        !query_data.contains("Self::fetch_with_ticks(world, entity, ticks)"),
        "cached query data must require an explicit location-aware fetch hook"
    );
    assert!(
        !query_filter.contains("Self::matches(world, entity, ticks)"),
        "cached query filters must require explicit location-aware matching"
    );
}
