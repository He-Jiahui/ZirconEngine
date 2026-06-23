use super::*;

#[test]
fn query_many_cached_iter_uses_borrowed_cache_index_membership() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let many_text = read_source(&query_root.join("query_many_iter.rs"));
    let read_only_cached_text =
        read_source(&query_root.join("query_state").join("read_only_cached.rs"));
    let helpers_text = read_source(&query_root.join("query_state").join("helpers.rs"));

    assert!(
        many_text.contains("pub struct QueryManyCachedIter<'world, 'state"),
        "query_many_iter.rs must expose a cached-many iterator that borrows QueryState cache membership"
    );
    assert!(
        many_text.contains("cached_entity_indices: &'state [(EntityId, usize)]"),
        "QueryManyCachedIter must borrow sorted QueryState cache indices instead of owning a filtered entity Vec"
    );
    assert!(
        many_text.contains("cached_locations: &'state [StableEntityLocation]")
            && many_text.contains("cached_component_locations: &'state [ComponentStorageLocation]")
            && many_text.contains("cached_component_location_offsets: &'state [usize]"),
        "QueryManyCachedIter must borrow QueryState's cached component-location path"
    );
    assert!(
        many_text
            .contains("let Some(index) = cached_query_entity_index(cached_entity_indices, entity)")
            || (many_text.contains("cached_query_entity_index(")
                && many_text.contains("cached_entity_indices")
                && many_text.contains("let Some(index)")),
        "QueryManyCachedIter must use cached_query_entity_index for binary cache membership"
    );
    assert!(
        read_only_cached_text.contains("QueryManyCachedIter::new(")
            && read_only_cached_text.contains("&self.cached_entity_indices")
            && read_only_cached_text.contains("&self.cached_locations")
            && read_only_cached_text.contains("&self.cached_component_locations")
            && read_only_cached_text.contains("&self.cached_component_location_offsets"),
        "query_state/read_only_cached.rs must construct QueryManyCachedIter with flat QueryState cached locations"
    );
    assert!(
        !read_only_cached_text.contains("let entities = cached_many_entities"),
        "query_state/read_only_cached.rs must not pre-collect cached many entities before iterator construction"
    );
    assert!(
        !helpers_text.contains("fn cached_many_entities"),
        "query_state/helpers.rs must not keep the old cached_many_entities allocation helper"
    );

    let cached_many_iterator = many_text
        .split("impl<'world, 'state, D, F, I> Iterator for QueryManyCachedIter")
        .nth(1)
        .and_then(|text| text.split("fn world_entity_matches").next())
        .expect("read QueryManyCachedIter iterator impl");
    assert!(
        cached_many_iterator.contains("F::matches_component_locations("),
        "QueryManyCachedIter must keep dynamic filter checks over cached component locations"
    );
    assert!(
        cached_many_iterator.contains("D::fetch_with_component_locations("),
        "QueryManyCachedIter must fetch through cached component locations"
    );
    assert!(
        !cached_many_iterator.contains("world_entity_matches::<D, F>"),
        "QueryManyCachedIter must not route cached membership through uncached entity validation"
    );
    assert!(
        !cached_many_iterator.contains("D::matches_data"),
        "QueryManyCachedIter must trust QueryState cache membership for QueryData shape"
    );
    assert!(
        !cached_many_iterator.contains("D::fetch_with_ticks(world, entity, ticks)"),
        "QueryManyCachedIter must not fetch through the uncached world lookup path"
    );
    assert!(
        !cached_many_iterator.contains("world.contains_entity"),
        "QueryManyCachedIter must not re-check entity existence after cache membership succeeds"
    );

    let contains_cached = read_only_cached_text
        .split("pub(crate) fn contains_cached_with_ticks")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn get_cached_with_ticks").next())
        .expect("read cached contains implementation");
    let get_cached_after_update = read_only_cached_text
        .split("fn get_cached_after_update_with_ticks")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn get_many_cached_with_ticks")
                .next()
        })
        .expect("read cached get-after-update implementation");
    assert!(
        contains_cached.contains("self.cached_entity_location(entity)")
            && (contains_cached.contains("F::matches_component_locations(")
                || contains_cached.contains("F::matches_component_locations_with_stats(")),
        "cached contains must use the resolved cache slot and component-location filter path"
    );
    assert!(
        get_cached_after_update.contains("self.cached_entity_location(entity)")
            && (get_cached_after_update.contains("F::matches_component_locations(")
                || get_cached_after_update.contains("F::matches_component_locations_with_stats("))
            && get_cached_after_update.contains("D::fetch_with_component_locations("),
        "cached get paths must reuse cached component locations for filter and fetch"
    );
    assert!(
        !contains_cached.contains("F::matches(world, entity, ticks)")
            && !get_cached_after_update.contains("F::matches(world, entity, ticks)")
            && !get_cached_after_update.contains("D::fetch_with_ticks(world, entity, ticks)"),
        "cached contains/get paths must not return to uncached filter/fetch lookups"
    );
    assert!(
        read_only_cached_text
            .split("pub(crate) fn get_cached_with_ticks")
            .nth(1)
            .and_then(|text| text.split("fn get_cached_after_update_with_ticks").next())
            .is_some_and(|text| text.contains("world.contains_entity(entity)")),
        "get_cached_with_ticks must keep NotSpawned detection before cache membership errors"
    );
}

#[test]
fn cached_query_iter_trusts_query_state_data_membership() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let iter_text = read_source(&query_root.join("query_iter.rs"));
    let read_only_cached_text =
        read_source(&query_root.join("query_state").join("read_only_cached.rs"));

    let cached_branch = iter_text
        .split(
            "if let (Some(locations), Some(component_locations), Some(component_location_offsets))",
        )
        .nth(1)
        .and_then(|text| text.split("continue;").next())
        .expect("read cached QueryIter branch");
    assert!(
        cached_branch.contains("F::matches_component_locations("),
        "cached QueryIter must keep dynamic filter checks over cached component locations"
    );
    assert!(
        cached_branch.contains("D::fetch_with_component_locations("),
        "cached QueryIter must fetch through cached component locations"
    );
    assert!(
        !cached_branch.contains("D::matches_component_locations"),
        "cached QueryIter must trust QueryState cache membership for QueryData shape"
    );
    assert!(
        iter_text.contains("F::matches(self.world, entity, self.ticks)")
            && iter_text.contains("D::matches_data(self.world, entity)"),
        "uncached QueryIter must keep full world entity validation"
    );
    assert!(
        read_only_cached_text.contains("self.update_cache(world);")
            && read_only_cached_text.contains("QueryIter::new_cached_locations(")
            && read_only_cached_text.contains("&self.cached_entities")
            && read_only_cached_text.contains("&self.cached_component_locations")
            && read_only_cached_text.contains("&self.cached_component_location_offsets"),
        "QueryState::iter_cached_with_ticks must refresh and pass cached membership to QueryIter"
    );
}

#[test]
fn query_many_cached_direct_iter_uses_requested_entity_stream_without_index_vec() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let cached_iter_text = read_source(&query_root.join("cached_query_iter.rs"));
    let cached_direct_text = read_source(&query_root.join("query_state").join("cached_direct.rs"));
    let system_query_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("system")
            .join("query.rs"),
    );

    assert!(
        cached_iter_text.contains("pub struct CachedQueryManyIter<'world, 'state, D, F = (), I ="),
        "CachedQueryManyIter must carry the caller-provided entity iterator type"
    );
    assert!(
        cached_iter_text.contains("cached_entity_indices: &'state [(EntityId, usize)]"),
        "CachedQueryManyIter must borrow QueryState's sorted cache index"
    );
    assert!(
        cached_iter_text.contains("component_locations: &'state [ComponentStorageLocation]")
            && cached_iter_text.contains("component_location_offsets: &'state [usize]"),
        "CachedQueryManyIter must borrow QueryState's flat cached component-location buffer and offsets"
    );
    assert!(
        cached_iter_text.contains("requested_entities: I"),
        "CachedQueryManyIter must keep the requested entity stream instead of an allocated index Vec"
    );
    assert!(
        cached_iter_text.contains("for entity_item in self.requested_entities.by_ref()"),
        "CachedQueryManyIter::next must consume the requested entity stream directly"
    );
    assert!(
        cached_iter_text.contains("cached_query_entity_index(self.cached_entity_indices, entity)"),
        "CachedQueryManyIter must derive cache slots from binary membership lookup"
    );
    assert!(
        !cached_iter_text.contains("indices: std::vec::IntoIter<usize>"),
        "CachedQueryManyIter must not own pre-collected cache indices"
    );
    assert!(
        !cached_iter_text.contains("fn cached_query_many_indices"),
        "cached_query_iter.rs must not keep the old index-collection helper"
    );
    assert!(
        !cached_iter_text.contains("fn matches_cached_data")
            && !cached_iter_text.contains("matches_cached_data("),
        "CachedQueryData must not expose a redundant structural match hook after QueryState cache rebuilds"
    );
    assert!(
        cached_direct_text.contains("&self.cached_entity_indices,")
            && cached_direct_text.contains("&self.cached_component_location_offsets")
            && !cached_direct_text.contains("let indices = cached_query_many_indices"),
        "cached_direct.rs must pass borrowed cache indices, flat component-location offsets, and the request stream to CachedQueryManyIter"
    );
    assert!(
        !cached_direct_text.contains("matches_cached_data"),
        "cached_direct.rs must trust QueryState cache membership for CachedQueryData shape"
    );
    assert!(
        system_query_text.contains("CachedQueryManyIter<'_, '_, D, F, EntityList::IntoIter>"),
        "system Query cached-direct many API must preserve the caller iterator type"
    );
}
