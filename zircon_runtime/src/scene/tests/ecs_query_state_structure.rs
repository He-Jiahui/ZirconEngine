fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .expect("read source section")
}

#[test]
fn cached_point_queries_project_entity_through_the_matching_archetype_plan() {
    let cached_direct = include_str!("../ecs/query/query_state/cached_direct.rs");
    let read_only_cached = include_str!("../ecs/query/query_state/read_only_cached.rs");
    let plan = include_str!("../ecs/query/query_state/archetype_plan.rs");

    assert!(plan.contains("let stable_location = world.internal_entity_location(entity)?;"));
    assert!(plan.contains("find_cached_archetype_plan(plans"));
    assert!(plan.contains("plan.write_component_locations(world, stable_location"));
    for source in [cached_direct, read_only_cached] {
        assert!(source.contains("self.project_entity(world, entity, &mut component_locations)"));
        assert!(!source.contains("cached_entity_index"));
        assert!(!source.contains("cached_component_location_offsets"));
    }
    assert!(cached_direct.contains("D::fetch_cached("));
    assert!(read_only_cached.contains("D::fetch_with_component_locations("));
}

#[test]
fn cached_count_and_empty_helpers_iterate_compiled_plans_without_projection_buffers() {
    let cached_direct = include_str!("../ecs/query/query_state/cached_direct.rs");
    let read_only_cached = include_str!("../ecs/query/query_state/read_only_cached.rs");

    let read_count = source_between(
        read_only_cached,
        "pub(crate) fn count_cached_with_ticks",
        "pub(crate) fn contains_cached_with_ticks",
    );
    let read_empty = source_between(
        read_only_cached,
        "pub(crate) fn is_empty_cached_with_ticks",
        "pub(crate) fn count_cached_with_ticks",
    );
    assert!(read_count.contains("self.iter_cached_with_ticks(world, ticks).count()"));
    assert!(read_empty.contains("self.iter_cached_with_ticks(world, ticks).next().is_none()"));

    let direct_count = source_between(
        cached_direct,
        "pub(crate) fn count_cached_direct_with_ticks",
        "pub(crate) fn contains_cached_direct_with_ticks",
    );
    let direct_empty = source_between(
        cached_direct,
        "pub(crate) fn is_empty_cached_direct_with_ticks",
        "pub(crate) fn count_cached_direct_with_ticks",
    );
    assert!(direct_count.contains("self.iter_cached_direct_with_ticks(world, ticks).count()"));
    assert!(direct_empty.contains("self.iter_cached_direct_with_ticks(world, ticks)"));
    assert!(direct_empty.contains(".next()"));
    for source in [read_count, read_empty, direct_count, direct_empty] {
        assert!(!source.contains("cached_entities"));
        assert!(!source.contains("cached_locations"));
    }
}

#[test]
fn query_state_many_item_collection_uses_direct_initialized_array_read() {
    let helpers = include_str!("../ecs/query/query_state/many_item_array.rs");
    let collect_many = helpers
        .split("pub(super) fn collect_many_query_items")
        .nth(1)
        .expect("read collect_many_query_items body");

    assert!(collect_many.contains("let mut values: [MaybeUninit<Item>; N]"));
    assert!(collect_many.contains("slot.write(item);"));
    assert!(collect_many.contains("value.assume_init_drop();"));
    assert!(collect_many.contains("as *const [Item; N]"));
    assert!(collect_many.contains(".read()"));
}
