use super::*;

#[test]
fn query_state_stays_folder_backed_by_query_owner() {
    let query_root = manifest_dir().join("src/scene/ecs/query");
    assert!(!query_root.join("query_state.rs").exists());
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
    let expected_modules = EXPECTED_QUERY_STATE_MODULES
        .iter()
        .map(|module| (*module).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_modules, expected_modules);

    let root = read_source(&owner_root.join("mod.rs"));
    assert!(
        root.lines().filter(|line| !line.trim().is_empty()).count()
            <= QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET
    );
    for module in EXPECTED_QUERY_STATE_MODULES {
        let path = owner_root.join(format!("{module}.rs"));
        assert!(
            path.exists(),
            "missing {}",
            relative_to_manifest(&path).display()
        );
        assert!(read_source(&path).lines().count() <= QUERY_STATE_OWNER_LINE_BUDGET);
    }
}

#[test]
fn query_state_cache_retains_only_compiled_archetype_plans_and_scalar_counts() {
    let owner_root = manifest_dir().join("src/scene/ecs/query/query_state");
    let root = read_source(&owner_root.join("mod.rs"));
    let plan = read_source(&owner_root.join("archetype_plan.rs"));

    assert!(root.contains("cached_archetype_plans: Vec<CachedArchetypePlan>"));
    assert!(root.contains("cached_archetype_generation: u64"));
    assert!(root.contains("cached_entity_count: usize"));
    for forbidden in [
        "cached_entities:",
        "cached_entity_indices:",
        "cached_locations:",
        "cached_component_locations:",
        "cached_component_location_offsets:",
        "cached_revision:",
    ] {
        assert!(
            !root.contains(forbidden),
            "QueryState retained `{forbidden}`"
        );
    }
    assert!(plan.contains("archetype_id: ArchetypeId"));
    assert!(plan.contains("membership_generation: u64"));
    assert!(plan.contains("bindings: Vec<QueryComponentBinding>"));
    assert!(plan.contains("column_slot: usize"));
    assert!(plan.contains("pub(crate) fn write_component_locations("));
}
