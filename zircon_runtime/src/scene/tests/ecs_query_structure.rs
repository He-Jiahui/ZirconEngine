use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_QUERY_STATE_MODULES: &[&str] = &[
    "cached_direct",
    "helpers",
    "mod",
    "mutable",
    "read_only",
    "read_only_cached",
    "stats",
    "system_param",
];
const QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET: usize = 180;
const QUERY_STATE_OWNER_LINE_BUDGET: usize = 450;

#[test]
fn query_state_stays_folder_backed_by_query_owner() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let legacy_file = query_root.join("query_state.rs");
    assert!(
        !legacy_file.exists(),
        "QueryState must stay folder-backed; do not recreate {}",
        relative_to_manifest(&legacy_file).display()
    );

    let owner_root = query_root.join("query_state");
    let actual_modules: BTreeSet<_> = std::fs::read_dir(&owner_root)
        .expect("read query_state owner directory")
        .map(|entry| {
            entry
                .expect("read query_state owner entry")
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".rs")
                .to_owned()
        })
        .collect();
    let expected_modules: BTreeSet<_> = EXPECTED_QUERY_STATE_MODULES
        .iter()
        .map(|module| (*module).to_owned())
        .collect();
    assert_eq!(
        actual_modules, expected_modules,
        "QueryState owner modules changed; update the architecture review before adding/removing query-state owners"
    );

    let query_mod = std::fs::read_to_string(query_root.join("mod.rs")).expect("read query mod");
    assert!(
        query_mod.contains("mod query_state;"),
        "query/mod.rs must keep QueryState behind the query_state owner module"
    );

    let root_path = owner_root.join("mod.rs");
    let root_text = std::fs::read_to_string(&root_path).expect("read query_state root");
    let root_non_empty_lines = root_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    assert!(
        root_non_empty_lines <= QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET,
        "query_state/mod.rs must stay a small state/cache owner; found {root_non_empty_lines} non-empty lines"
    );
    for forbidden in [
        "D: CachedQueryData",
        "D: QueryData,",
        "D: QueryMutData",
        "impl<D, F> SystemParam",
    ] {
        assert!(
            !root_text.contains(forbidden),
            "query_state/mod.rs must not own `{forbidden}` impl families"
        );
    }

    for module in EXPECTED_QUERY_STATE_MODULES {
        let module_path = owner_root.join(format!("{module}.rs"));
        assert!(
            module_path.exists(),
            "missing QueryState owner module {}",
            relative_to_manifest(&module_path).display()
        );
        let module_lines = std::fs::read_to_string(&module_path)
            .expect("read QueryState owner module")
            .lines()
            .count();
        assert!(
            module_lines <= QUERY_STATE_OWNER_LINE_BUDGET,
            "{} must split again before it becomes another ECS query hot spot; found {module_lines} lines",
            relative_to_manifest(&module_path).display()
        );
    }

    let read_only_text = read_source(&owner_root.join("read_only.rs"));
    let read_only_cached_text = read_source(&owner_root.join("read_only_cached.rs"));
    let cached_direct_text = read_source(&owner_root.join("cached_direct.rs"));
    for forbidden in [
        "iter_cached",
        "single_cached",
        "get_cached",
        "iter_many_cached",
        "iter_combinations_cached",
        "contains_cached",
    ] {
        assert!(
            !read_only_text.contains(forbidden),
            "query_state/read_only.rs must stay uncached-only; `{forbidden}` belongs in read_only_cached.rs"
        );
        assert!(
            read_only_cached_text.contains(forbidden),
            "query_state/read_only_cached.rs must own `{forbidden}`"
        );
    }

    assert!(
        root_text.contains("pub(crate) fn cached_entity_location("),
        "query_state/mod.rs must own shared cache-slot resolution for cached query owners"
    );
    assert!(
        root_text.contains("cache_hits: u64")
            && root_text.contains("cache_misses: u64")
            && root_text.contains("last_candidate_entity_count: usize")
            && root_text.contains("last_matched_entity_count: usize"),
        "QueryState must keep Runtime 07 cache telemetry counters beside the cache owner"
    );
    let stats_text = read_source(&owner_root.join("stats.rs"));
    assert!(
        stats_text.contains("pub struct QueryStateCacheStats")
            && stats_text.contains("pub fn cache_stats(&self) -> QueryStateCacheStats")
            && stats_text.contains("pub fn record_diagnostics(")
            && stats_text.contains("ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC")
            && stats_text.contains("\"ecs.query.archetype_cache_hits\"")
            && stats_text.contains("cache_hits: self.cache_hits")
            && stats_text.contains("candidate_entity_count: self.last_candidate_entity_count"),
        "query_state/stats.rs must expose the Runtime 07 cache telemetry snapshot without moving cache ownership"
    );
    assert!(
        root_text.contains("cached_component_locations: Vec<ComponentStorageLocation>")
            && root_text.contains("cached_component_location_offsets: Vec<usize>")
            && root_text.contains("Vec::with_capacity(self.access.reads().len())")
            && root_text.contains(".extend(component_locations.iter().copied())"),
        "QueryState cache rebuilds must store cached component locations as one flat buffer plus per-entity offsets"
    );
    assert!(
        !root_text.contains("Vec<Vec<ComponentStorageLocation>>"),
        "QueryState must not preserve one cached component-location Vec allocation per matched entity"
    );
    for (path, text) in [
        (
            "query_state/read_only_cached.rs",
            read_only_cached_text.as_str(),
        ),
        ("query_state/cached_direct.rs", cached_direct_text.as_str()),
    ] {
        assert!(
            text.contains("self.cached_entity_location(entity)"),
            "{path} must use QueryState's shared cache-slot resolver"
        );
    }
}

#[test]
fn cached_component_location_paths_fail_closed_on_cache_vector_drift() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");

    for relative_path in [
        "cached_query_iter.rs",
        "query_iter.rs",
        "query_many_iter.rs",
        "query_state/mod.rs",
    ] {
        let text = read_source(&query_root.join(relative_path));
        assert!(
            !text.contains(".map_or(&[][..], Vec::as_slice)"),
            "{relative_path} must fail closed when a cached component-location slot is missing instead of treating cache drift as an empty component slice"
        );
    }

    let cached_iter_text = read_source(&query_root.join("cached_query_iter.rs"));
    assert!(
        cached_iter_text.contains("pub(crate) fn cached_query_component_locations")
            && cached_iter_text.contains("component_location_offsets: &[usize]")
            && cached_iter_text.contains("component_locations.get(start..end)"),
        "cached_query_iter.rs must own the flat component-location slice resolver"
    );
}

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
            && contains_cached.contains("F::matches_component_locations("),
        "cached contains must use the resolved cache slot and component-location filter path"
    );
    assert!(
        get_cached_after_update.contains("self.cached_entity_location(entity)")
            && get_cached_after_update.contains("F::matches_component_locations(")
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
            .and_then(|text| text.split("fn cached_entity_location").next())
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

#[test]
fn query_state_cache_rebuild_uses_access_reads_without_per_rebuild_merge() {
    let query_root = manifest_dir()
        .join("src")
        .join("scene")
        .join("ecs")
        .join("query");
    let state_text = read_source(&query_root.join("query_state").join("mod.rs"));
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
        state_text.contains(
            "component_storage_locations_for_internal("
        ),
        "QueryState cache rebuilds must reuse access.reads() for cached component storage locations"
    );
    assert!(
        state_text.contains("self.access.reads(),")
            && state_text.contains("&mut component_locations"),
        "QueryState cache rebuilds must fill a reusable component-location scratch Vec from access.reads()"
    );
    assert!(
        state_text.contains("let matched_archetypes = world.matching_query_archetypes(&self.access);")
            && state_text.contains(
                "let candidate_count = world.matching_query_archetype_entity_count(&matched_archetypes);",
            )
            && state_text.contains(
                "world.visit_entity_locations_matching_archetypes(&matched_archetypes, |location| {",
            )
            && !state_text.contains("entity_locations_matching_query_archetypes")
            && !state_text.contains("candidate_locations"),
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
        state_text.contains("let candidate_count = world.matching_query_archetype_entity_count(&matched_archetypes);")
            && state_text.contains("let component_count = self.access.reads().len();")
            && state_text.contains("self.cached_entities.reserve(candidate_count);")
            && state_text.contains("self.cached_entity_indices.reserve(candidate_count);")
            && state_text.contains("self.cached_locations.reserve(candidate_count);")
            && state_text.contains(
                "self.cached_component_location_offsets\n            .reserve(candidate_count);",
            )
            && state_text.contains(
                "self.cached_component_locations\n            .reserve(candidate_count.saturating_mul(component_count));",
            ),
        "QueryState cache rebuilds must reserve candidate-sized entity/location caches before repopulating them"
    );
    assert!(
        !state_text.contains("cached_component_ids"),
        "QueryState::update_cache must not rebuild a temporary cached_component_ids Vec per cache revision"
    );
    assert!(
        state_text.contains("cached_component_locations: Vec<ComponentStorageLocation>")
            && state_text.contains("cached_component_location_offsets: Vec<usize>")
            && !state_text.contains("Vec<Vec<ComponentStorageLocation>>"),
        "QueryState must keep component-location cache storage flat instead of retaining one Vec per entity"
    );
    assert!(
        !state_text.contains("self.access.writes().iter()"),
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

#[test]
fn archetype_index_matching_reuses_sorted_component_index_without_per_query_resort() {
    let signature_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("archetype_signature.rs"),
    );
    let archetype_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("archetype_index.rs"),
    );
    let matching_archetypes = archetype_text
        .split("pub fn matching_archetypes(")
        .nth(1)
        .and_then(|text| text.split("fn add_entity_to").next())
        .expect("read matching_archetypes implementation");
    let signature_indexer = archetype_text
        .split("fn index_signature_components(")
        .nth(1)
        .and_then(|text| text.split("impl Default for ArchetypeIndex").next())
        .expect("read index_signature_components implementation");

    assert!(
        signature_indexer.contains("insert_archetype_id(ids, id);"),
        "ArchetypeIndex must centralize sorted per-component index insertion"
    );
    assert!(
        archetype_text
            .contains("fn insert_archetype_id(ids: &mut Vec<ArchetypeId>, id: ArchetypeId)")
            && archetype_text.contains("if let Err(index) = ids.binary_search(&id)")
            && archetype_text.contains("ids.insert(index, id);"),
        "per-component archetype index lists must stay sorted and unique as they are built"
    );
    assert!(
        !signature_indexer.contains("ids.contains(&id)")
            && !signature_indexer.contains("ids.sort_unstable();"),
        "signature indexing must not use linear contains plus full-list sort after each insertion"
    );
    assert!(
        !matching_archetypes.contains("candidates.sort_unstable();")
            && !matching_archetypes.contains("candidates.dedup();"),
        "matching_archetypes must rely on sorted unique index ownership instead of resorting each query"
    );
    assert!(
        matching_archetypes.contains("all_archetype_ids(&self.records)")
            && !matching_archetypes
                .contains("self.records.iter().map(ArchetypeRecord::id).collect()"),
        "matching_archetypes must size the all-archetype fallback instead of relying on collect growth"
    );
    assert!(
        matching_archetypes.contains("candidates.retain(|id|"),
        "matching_archetypes should filter the already-sorted candidate list in place"
    );
    assert!(
        archetype_text
            .contains("fn all_archetype_ids(records: &[ArchetypeRecord]) -> Vec<ArchetypeId>")
            && archetype_text.contains("let mut ids = Vec::with_capacity(records.len());")
            && archetype_text.contains("for record in records")
            && archetype_text.contains("ids.push(record.id());"),
        "all-archetype query fallback must use exact-capacity Vec construction"
    );
    assert!(
        signature_text.contains("fn normalize_components(mut components: Vec<ComponentId>)")
            && signature_text.contains("if components.len() > 1")
            && signature_text.contains("components.sort_unstable();")
            && signature_text.contains("components.dedup();"),
        "ArchetypeSignature normalization must skip sort/dedup for trivial component lists while preserving normalized multi-component signatures"
    );
}

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
            && owned_read_constructor.contains("let candidate_count =")
            && owned_read_constructor
                .contains("read_only_combination_candidate_count::<D, F>(world, entities, ticks)")
            && owned_read_constructor.contains("if candidate_count < K")
            && owned_read_constructor
                .contains("let mut matched_entities = Vec::with_capacity(candidate_count);")
            && owned_read_constructor.contains("for entity in entities.iter().copied()")
            && owned_read_constructor
                .contains("read_only_combination_candidate_matches::<D, F>(world, entity, ticks)")
            && owned_read_constructor.contains("matched_entities.push(entity)")
            && !owned_read_constructor.contains(".collect::<Vec<_>>()")
            && !owned_read_constructor.contains("filter(|entity|"),
        "uncached read-only combinations must skip impossible group sizes, then count candidates first and push into an exact-capacity Vec"
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

#[test]
fn query_access_conflicts_with_uses_allocation_free_boolean_path() {
    let query_access_text = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("query")
            .join("query_access.rs"),
    );

    assert!(
        query_access_text.contains("pub fn conflicts_with(&self, other: &Self) -> bool"),
        "QueryAccess must keep a boolean conflict-check entry point"
    );
    assert!(
        query_access_text.contains("!self.has_disjoint_filter(other)")
            && query_access_text
                .contains("sorted_component_slices_intersect(&self.writes, &other.reads)")
            && query_access_text
                .contains("sorted_component_slices_intersect(&self.reads, &other.writes)"),
        "QueryAccess::conflicts_with must check sorted read/write intersections directly"
    );
    assert!(
        !query_access_text
            .contains("sorted_component_slices_intersect(&self.writes, &other.writes)"),
        "QueryAccess::conflicts_with must not repeat write/write checks already covered by write-implies-read"
    );
    assert!(
        !query_access_text.contains("!self.conflicting_components_with(other).is_empty()"),
        "QueryAccess::conflicts_with must not allocate the detailed conflict Vec for boolean checks"
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", relative_to_manifest(path).display()))
}

fn relative_to_manifest(path: &Path) -> PathBuf {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_path_buf()
}
