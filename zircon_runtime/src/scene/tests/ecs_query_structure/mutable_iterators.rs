use super::*;

#[test]
fn query_many_mut_iterators_use_borrowed_cache_index_membership() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let many_text = read_source(&query_root.join("query_many_mut_iter.rs"));
    let many_unique_text = read_source(&query_root.join("query_many_unique_mut_iter.rs"));
    let mutable_text = read_source(&query_root.join("query_state").join("mutable.rs"));

    for (path, text, struct_name) in [
        (
            "query_many_mut_iter.rs",
            many_text.as_str(),
            "QueryManyMutIter",
        ),
        (
            "query_many_unique_mut_iter.rs",
            many_unique_text.as_str(),
            "QueryManyUniqueMutIter",
        ),
    ] {
        assert!(
            text.contains(&format!("pub struct {struct_name}<'world, 'state")),
            "{path} must keep a state lifetime for borrowed QueryState cache membership"
        );
        assert!(
            text.contains("cached_entity_indices: &'state [(EntityId, usize)]"),
            "{path} must borrow sorted QueryState cache indices instead of owning a cloned entity Vec"
        );
        assert!(
            text.contains("cached_component_locations: &'state [ComponentStorageLocation]")
                && text.contains("cached_component_location_offsets: &'state [usize]"),
            "{path} must borrow QueryState's flat component-location cache for dynamic filters"
        );
        assert!(
            text.contains("cached_query_entity_index(self.cached_entity_indices, entity)"),
            "{path} must use cached_query_entity_index for binary cache membership"
        );
        assert!(
            text.contains("cached_query_component_locations(")
                && text.contains("F::matches_component_locations("),
            "{path} must evaluate dynamic filters through cached component locations"
        );
        assert!(
            !text.contains("cached_entities: Vec<EntityId>"),
            "{path} must not own cloned QueryState cached entities"
        );
        assert!(
            !text.contains("cached_entities.contains"),
            "{path} must not use linear Vec::contains on cached entities"
        );
        assert!(
            !text.contains("D::matches_data(world, entity)"),
            "{path} must trust QueryState cache membership for QueryData shape"
        );
        assert!(
            !text.contains("F::matches(world, entity, self.ticks)"),
            "{path} must not return to world-level filter lookups after cache membership succeeds"
        );
    }

    assert!(
        !mutable_text.contains("self.cached_entities.clone(), entities, ticks"),
        "query_state/mutable.rs must not clone cached_entities when creating many-mut iterators"
    );
    assert!(
        mutable_text.contains("&self.cached_entity_indices")
            && mutable_text.contains("&self.cached_component_locations")
            && mutable_text.contains("&self.cached_component_location_offsets"),
        "query_state/mutable.rs must pass borrowed sorted cache indices plus flat component-location cache to many-mut iterators"
    );
    assert!(
        mutable_text
            .contains("for (index, entity) in self.cached_entities.iter().copied().enumerate()"),
        "query_state/mutable.rs for_each_mut must iterate the QueryState cache directly"
    );
    assert!(
        !mutable_text.contains("let entities = self.cached_entities.clone();"),
        "query_state/mutable.rs for_each_mut must not clone all cached entities before callbacks"
    );
    assert!(
        !mutable_text.contains("D::matches_data(world, entity)"),
        "query_state/mutable.rs cached mutable paths must trust QueryState cache membership for QueryData shape"
    );
    assert!(
        mutable_text.contains("cached_query_component_locations(")
            && mutable_text.contains("F::matches_component_locations(")
            && mutable_text.contains("self.cached_entity_location(entity)"),
        "query_state/mutable.rs cached mutable paths must reuse cached component-location slices for dynamic filters"
    );
    assert!(
        !mutable_text.contains("F::matches(world, entity, ticks)"),
        "query_state/mutable.rs cached mutable paths must not return to world-level filter lookups"
    );
}

#[test]
fn query_mut_iter_uses_borrowed_cached_entities_without_recollecting() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let query_mut_text = read_source(&query_root.join("query_mut_iter.rs"));
    let system_query_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("system")
            .join("query.rs"),
    );

    assert!(
        query_mut_text.contains("pub struct QueryMutIter<'world, 'state"),
        "QueryMutIter must borrow QueryState cache membership with an explicit state lifetime"
    );
    assert!(
        query_mut_text.contains("entities: &'state [EntityId]"),
        "QueryMutIter must borrow cached entity ids instead of owning a rebuilt Vec"
    );
    assert!(
        query_mut_text.contains("component_locations: &'state [ComponentStorageLocation]")
            && query_mut_text.contains("component_location_offsets: &'state [usize]"),
        "QueryMutIter must borrow QueryState's flat component-location cache for dynamic filters"
    );
    assert!(
        query_mut_text.contains("QueryMutIter::new(")
            && query_mut_text.contains("self.cached_entities()")
            && query_mut_text.contains("self.cached_component_locations()")
            && query_mut_text.contains("self.cached_component_location_offsets()"),
        "QueryState::iter_mut_with_ticks must pass the cached entity slice plus flat component-location cache"
    );
    assert!(
        query_mut_text.contains("self.cached_entities()"),
        "QueryMutIter construction must not bypass the QueryState cache accessor"
    );
    assert!(
        !query_mut_text.contains(".collect::<Vec<_>>()"),
        "QueryMutIter construction must not recollect cached locations into a Vec"
    );
    assert!(
        !query_mut_text.contains("D::matches_data(world, entity)"),
        "QueryMutIter must trust QueryState cache membership for QueryData shape"
    );
    assert!(
        query_mut_text.contains("cached_query_component_locations(")
            && query_mut_text.contains("F::matches_component_locations("),
        "QueryMutIter must evaluate dynamic filters through cached component locations"
    );
    assert!(
        !query_mut_text.contains("F::matches(world, entity, self.ticks)"),
        "QueryMutIter must not return to world-level filter lookups after cache membership succeeds"
    );
    assert!(
        !query_mut_text.contains("std::vec::IntoIter<EntityId>"),
        "QueryMutIter must not own a Vec iterator over cloned entity ids"
    );
    assert!(
        system_query_text.contains("QueryMutIter<'_, '_, D, F>"),
        "system Query::iter_mut must preserve the borrowed QueryState cache lifetime"
    );
}
