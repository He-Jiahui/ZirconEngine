fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .expect("read source section")
}

#[test]
fn query_state_item_fetch_errors_use_direct_branches() {
    let cached_direct = include_str!("../ecs/query/query_state/cached_direct.rs");
    let read_only_cached = include_str!("../ecs/query/query_state/read_only_cached.rs");
    let read_only = include_str!("../ecs/query/query_state/read_only.rs");
    let mutable = include_str!("../ecs/query/query_state/mutable.rs");
    let cached_direct_get = source_between(
        cached_direct,
        "pub(crate) fn get_cached_direct_with_ticks",
        "pub(crate) fn get_many_cached_direct_with_ticks",
    );
    let cached_direct_get_after_update = source_between(
        cached_direct,
        "fn get_cached_direct_after_update_with_ticks",
        "\n    }\n}",
    );

    assert!(
        cached_direct_get.contains("let Some(item)")
            && cached_direct_get.contains("D::fetch_cached(")
            && cached_direct_get
                .contains("return Err(QueryEntityError::QueryDoesNotMatch(entity));")
            && cached_direct_get.contains("Ok(item)")
            && !cached_direct_get.contains(".ok_or(")
            && cached_direct_get_after_update.contains("let Some(item)")
            && cached_direct_get_after_update.contains("D::fetch_cached(")
            && cached_direct_get_after_update
                .contains("return Err(QueryEntityError::QueryDoesNotMatch(entity));")
            && cached_direct_get_after_update.contains("Ok(item)")
            && !cached_direct_get_after_update.contains(".ok_or("),
        "cached direct query fetches must project missing data through direct branches"
    );
    assert!(
        read_only_cached.contains("let Some(item)")
            && read_only_cached.contains("D::fetch_with_component_locations(")
            && read_only_cached
                .contains("return Err(QueryEntityError::QueryDoesNotMatch(entity));")
            && read_only_cached.contains("Ok(item)")
            && !read_only_cached.contains(".ok_or("),
        "read-only cached query fetches must project missing data through a direct branch"
    );
    assert!(
        read_only.contains("let Some(item) = D::fetch_with_ticks(world, entity, ticks) else")
            && read_only.contains("return Err(QueryEntityError::QueryDoesNotMatch(entity));")
            && read_only.contains("Ok(item)")
            && !read_only.contains(".ok_or("),
        "read-only query fetches must project missing data through a direct branch"
    );
    assert!(
        mutable.contains("let Some(item) = D::fetch_mut_with_ticks(world, entity, ticks) else")
            && mutable
                .contains("let Some(item) = D::fetch_mut_with_ticks(unsafe { &mut *world }, entity, ticks) else")
            && mutable.contains("let Some(entity) = matched else")
            && mutable.contains("return Err(QuerySingleError::NoEntities);")
            && mutable.contains("return Err(QueryEntityError::QueryDoesNotMatch(entity));")
            && !mutable.contains(".ok_or("),
        "mutable query fetches must project missing data through direct branches"
    );
}

#[test]
fn query_state_count_helpers_scan_directly() {
    let cached_direct = include_str!("../ecs/query/query_state/cached_direct.rs");
    let read_only_cached = include_str!("../ecs/query/query_state/read_only_cached.rs");
    let read_only = include_str!("../ecs/query/query_state/read_only.rs");
    let system_query = include_str!("../ecs/system/query.rs");

    let read_only_count = read_only
        .split("pub(crate) fn count_with_ticks")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn get_with_ticks").next())
        .expect("read read-only count_with_ticks body");
    assert!(
        read_only.contains(
            "self.count_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))"
        ) && read_only_count.contains("let mut count = 0_usize;")
            && read_only_count
                .contains("for entity in world.entity_ids_for_query().iter().copied()")
            && read_only_count.contains("D::fetch_with_ticks(world, entity, ticks).is_some()")
            && read_only_count.contains("count += 1;")
            && !read_only.contains("self.iter(world).count()"),
        "read-only query counts must scan matching entities directly"
    );

    let read_only_cached_count = read_only_cached
        .split("pub(crate) fn count_cached_with_ticks")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn contains_cached_with_ticks")
                .next()
        })
        .expect("read cached read-only count body");
    assert!(
        read_only_cached_count.contains("self.update_cache(world);")
            && read_only_cached_count.contains("let mut index = 0_usize;")
            && read_only_cached_count.contains("while index < self.cached_entities.len()")
            && read_only_cached_count.contains("cached_query_component_locations(")
            && read_only_cached_count.contains("D::fetch_with_component_locations(")
            && read_only_cached_count.contains(".is_some()")
            && read_only_cached_count.contains("count += 1;")
            && !read_only_cached_count
                .contains("self.iter_cached_with_ticks(world, ticks).count()"),
        "cached read-only query counts must refresh cache and scan cached rows directly"
    );

    let cached_direct_count = cached_direct
        .split("pub(crate) fn count_cached_direct_with_ticks")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn contains_cached_direct_with_ticks")
                .next()
        })
        .expect("read cached-direct count body");
    assert!(
        cached_direct_count.contains("self.update_cache(world);")
            && cached_direct_count.contains("while index < self.cached_entities.len()")
            && cached_direct_count
                .contains("F::matches_cached(world, entity, component_locations, ticks)")
            && cached_direct_count.contains(
                "D::fetch_cached(world, entity, stable_location, component_locations, ticks)"
            )
            && cached_direct_count.contains("count += 1;")
            && !cached_direct_count
                .contains("self.iter_cached_direct_with_ticks(world, ticks).count()"),
        "cached-direct query counts must scan cached rows directly"
    );

    let system_count = system_query
        .split("pub fn count(&self) -> usize")
        .nth(1)
        .and_then(|text| text.split("pub fn contains(&self").next())
        .expect("read system Query::count body");
    assert!(
        system_count.contains("let state = unsafe { &mut *self.state };")
            && system_count.contains("state.count_cached_with_ticks(world, self.ticks)")
            && !system_count.contains("self.iter().count()"),
        "system read-only Query::count must preserve cache refresh while avoiding iterator count"
    );
}

#[test]
fn query_state_empty_helpers_scan_directly() {
    let cached_direct = include_str!("../ecs/query/query_state/cached_direct.rs");
    let read_only_cached = include_str!("../ecs/query/query_state/read_only_cached.rs");
    let read_only = include_str!("../ecs/query/query_state/read_only.rs");
    let system_query = include_str!("../ecs/system/query.rs");

    let read_only_empty = read_only
        .split("pub(crate) fn is_empty_with_ticks")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn count_with_ticks").next())
        .expect("read read-only is_empty_with_ticks body");
    assert!(
        read_only.contains(
            "self.is_empty_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))"
        ) && read_only_empty.contains("for entity in world.entity_ids_for_query().iter().copied()")
            && read_only_empty.contains("D::fetch_with_ticks(world, entity, ticks).is_some()")
            && read_only_empty.contains("return false;")
            && read_only_empty.contains("true")
            && !read_only.contains("self.iter(world).next().is_none()"),
        "read-only query empty checks must scan matching entities directly"
    );

    let read_only_cached_empty = read_only_cached
        .split("pub(crate) fn is_empty_cached_with_ticks")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn count_cached_with_ticks").next())
        .expect("read cached read-only empty body");
    assert!(
        read_only_cached_empty.contains("self.update_cache(world);")
            && read_only_cached_empty.contains("let mut index = 0_usize;")
            && read_only_cached_empty.contains("while index < self.cached_entities.len()")
            && read_only_cached_empty.contains("cached_query_component_locations(")
            && read_only_cached_empty.contains("D::fetch_with_component_locations(")
            && read_only_cached_empty.contains("return false;")
            && read_only_cached_empty.contains("true")
            && !read_only_cached_empty
                .contains("self.iter_cached_with_ticks(world, ticks).next().is_none()"),
        "cached read-only query empty checks must refresh cache and scan cached rows directly"
    );

    let cached_direct_empty = cached_direct
        .split("pub(crate) fn is_empty_cached_direct_with_ticks")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn count_cached_direct_with_ticks")
                .next()
        })
        .expect("read cached-direct empty body");
    assert!(
        cached_direct_empty.contains("self.update_cache(world);")
            && cached_direct_empty.contains("while index < self.cached_entities.len()")
            && cached_direct_empty
                .contains("F::matches_cached(world, entity, component_locations, ticks)")
            && cached_direct_empty.contains(
                "D::fetch_cached(world, entity, stable_location, component_locations, ticks)"
            )
            && cached_direct_empty.contains("return false;")
            && cached_direct_empty.contains("true")
            && !cached_direct_empty.contains("iter_cached_direct_with_ticks(world, ticks)")
            && !cached_direct_empty.contains(".next().is_none()"),
        "cached-direct query empty checks must scan cached rows directly"
    );

    let system_empty = system_query
        .split("pub fn is_empty(&self) -> bool")
        .nth(1)
        .and_then(|text| text.split("pub fn count(&self) -> usize").next())
        .expect("read system Query::is_empty body");
    assert!(
        system_empty.contains("let state = unsafe { &mut *self.state };")
            && system_empty.contains("state.is_empty_cached_with_ticks(world, self.ticks)")
            && !system_empty.contains("self.iter().next().is_none()"),
        "system read-only Query::is_empty must preserve cache refresh while avoiding iterator first-item probing"
    );
}

#[test]
fn query_state_many_item_collection_uses_direct_initialized_array_read() {
    let helpers = include_str!("../ecs/query/query_state/helpers.rs");
    let collect_many = helpers
        .split("pub(super) fn collect_many_query_items")
        .nth(1)
        .expect("read collect_many_query_items body");

    assert!(
        collect_many.contains("let mut values: [MaybeUninit<Item>; N]")
            && collect_many.contains("slot.write(item);")
            && collect_many.contains("value.assume_init_drop();")
            && collect_many.contains("let initialized = unsafe")
            && collect_many.contains("as *const [Item; N]")
            && collect_many.contains(".read()")
            && collect_many.contains("Ok(initialized)")
            && !collect_many.contains("values.map(|value|"),
        "many-item query collection must read the initialized array directly instead of using array map"
    );
}
