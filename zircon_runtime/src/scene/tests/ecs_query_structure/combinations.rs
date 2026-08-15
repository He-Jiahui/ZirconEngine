use super::*;

#[test]
fn cached_combinations_keep_call_local_candidates_and_reuse_archetype_plans() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    let read = read_source(&query_root.join("query_combinations_iter.rs"));
    let mutable = read_source(&query_root.join("query_combinations_mut_iter.rs"));
    let read_state = read_source(&query_root.join("query_state/read_only_cached.rs"));
    let mutable_state = read_source(&query_root.join("query_state/mutable.rs"));

    assert!(read.contains("pub(crate) fn new_from_cached_plans("));
    assert!(read.contains("plans: &'state [CachedArchetypePlan]"));
    assert!(read.contains("stable_locations: Vec<StableEntityLocation>"));
    assert!(read.contains("plan.write_component_locations("));
    assert!(read.contains("D::fetch_with_component_locations("));
    assert!(mutable.contains("pub(crate) fn new_from_cached_plans("));
    assert!(mutable.contains("plans: &'state [CachedArchetypePlan]"));
    assert!(mutable.contains("D::fetch_mut_with_component_locations("));
    assert!(read_state.contains("QueryCombinationIter::new_from_cached_plans("));
    assert!(mutable_state.contains("QueryCombinationMutIter::new_from_cached_plans("));

    for source in [&read, &mutable, &read_state, &mutable_state] {
        assert!(!source.contains("cached_component_locations"));
        assert!(!source.contains("cached_component_location_offsets"));
    }
}
