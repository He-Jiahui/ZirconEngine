use super::*;

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

    let cache_text = read_source(&owner_root.join("cache.rs"));
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
        cache_text.contains("pub(crate) fn cached_entity_location("),
        "query_state/cache.rs must own shared cache-slot resolution for cached query owners"
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
            && stats_text.contains("candidate_entity_count: self.last_candidate_entity_count")
            && stats_text.contains("pub(crate) fn record_change_detection_stats("),
        "query_state/stats.rs must expose Runtime 07 cache and change-detection telemetry without moving cache ownership"
    );
    assert!(
        root_text.contains("cached_component_locations: Vec<ComponentStorageLocation>")
            && root_text.contains("cached_component_location_offsets: Vec<usize>")
            && cache_text.contains("Vec::with_capacity(component_count)")
            && cache_text.contains(".extend(component_locations.iter().copied())"),
        "QueryState cache owner must store cached component locations as one flat buffer plus per-entity offsets"
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
