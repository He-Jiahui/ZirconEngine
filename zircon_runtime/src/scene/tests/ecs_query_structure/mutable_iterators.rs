use super::*;

#[test]
fn mutable_query_iterators_borrow_plans_and_keep_only_call_local_scratch() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    for (path, struct_name) in [
        ("query_mut_iter.rs", "QueryMutIter"),
        ("query_many_mut_iter.rs", "QueryManyMutIter"),
        ("query_many_unique_mut_iter.rs", "QueryManyUniqueMutIter"),
    ] {
        let source = read_source(&query_root.join(path));
        assert!(source.contains(&format!("pub struct {struct_name}<'world, 'state")));
        assert!(source.contains("plans: &'state [CachedArchetypePlan]"));
        assert!(source.contains("component_locations: Vec<ComponentStorageLocation>"));
        assert!(source.contains("D::fetch_mut_with_component_locations("));
        assert!(!source.contains("cached_entity_indices"));
        assert!(!source.contains("cached_component_location_offsets"));
    }
}

#[test]
fn full_mutable_iteration_snapshots_only_stable_locations_for_alias_safety() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    let iter = read_source(&query_root.join("query_mut_iter.rs"));

    assert!(iter.contains("candidates: Vec<StableEntityLocation>"));
    assert!(iter.contains("world.stable_query_location_iter("));
    assert!(iter.contains("find_cached_archetype_plan(self.plans"));
    assert!(iter.contains("plan.write_component_locations("));
    assert!(iter.contains("QueryMutIter::new(world, &self.cached_archetype_plans, ticks)"));
    assert!(!iter.contains("cached_entities.clone()"));
}

#[test]
fn mutable_point_and_many_queries_project_each_requested_entity_from_plans() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    let many = read_source(&query_root.join("query_many_mut_iter.rs"));
    let unique = read_source(&query_root.join("query_many_unique_mut_iter.rs"));
    let mutable = read_source(&query_root.join("query_state/mutable.rs"));

    for source in [&many, &unique] {
        assert!(source.contains("project_entity_from_plans("));
        assert!(source.contains("F::matches_component_locations("));
        assert!(!source.contains("D::matches_data(world, entity)"));
    }
    assert!(mutable.contains("project_entity_from_plans("));
    assert!(mutable.contains("&self.cached_archetype_plans"));
}
