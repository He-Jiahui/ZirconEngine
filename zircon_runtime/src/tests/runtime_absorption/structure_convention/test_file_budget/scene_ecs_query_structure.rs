use super::*;

#[test]
fn runtime_15_scene_ecs_query_structure_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/ecs_query_structure.rs");
    let archetype_access = read_runtime_src("scene/tests/ecs_query_structure/archetype_access.rs");
    let cache_rebuild = read_runtime_src("scene/tests/ecs_query_structure/cache_rebuild.rs");
    let cached_iterators = read_runtime_src("scene/tests/ecs_query_structure/cached_iterators.rs");
    let combinations = read_runtime_src("scene/tests/ecs_query_structure/combinations.rs");
    let mutable_iterators =
        read_runtime_src("scene/tests/ecs_query_structure/mutable_iterators.rs");
    let query_state_layout =
        read_runtime_src("scene/tests/ecs_query_structure/query_state_layout.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "scene ECS query structure parent test module mounts",
        &parent,
        &[
            "mod archetype_access;",
            "mod cache_rebuild;",
            "mod cached_iterators;",
            "mod combinations;",
            "mod mutable_iterators;",
            "mod query_state_layout;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/ecs_query_structure.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn query_state_stays_folder_backed_by_query_owner",
        "fn query_many_mut_iterators_use_borrowed_cache_index_membership",
        "fn query_many_cached_iter_uses_borrowed_cache_index_membership",
        "fn query_state_cache_rebuild_uses_access_reads_without_per_rebuild_merge",
        "fn cached_combinations_trust_query_state_data_membership",
        "fn query_access_conflicts_with_uses_allocation_free_boolean_path",
    ] {
        assert!(
            !parent.contains(moved_test),
            "scene/tests/ecs_query_structure.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "query-state layout child owns query-state module and cache-vector contracts",
        &query_state_layout,
        &[
            "use super::*;",
            "fn query_state_stays_folder_backed_by_query_owner",
            "fn cached_component_location_paths_fail_closed_on_cache_vector_drift",
        ],
    );
    assert_contains_all(
        "mutable iterator child owns borrowed mutable query iterator contracts",
        &mutable_iterators,
        &[
            "use super::*;",
            "fn query_many_mut_iterators_use_borrowed_cache_index_membership",
            "fn query_mut_iter_uses_borrowed_cached_entities_without_recollecting",
        ],
    );
    assert_contains_all(
        "cached iterator child owns cached read iterator contracts",
        &cached_iterators,
        &[
            "use super::*;",
            "fn query_many_cached_iter_uses_borrowed_cache_index_membership",
            "fn cached_query_iter_trusts_query_state_data_membership",
            "fn query_many_cached_direct_iter_uses_requested_entity_stream_without_index_vec",
        ],
    );
    assert_contains_all(
        "cache rebuild child owns cache reserve and hot-path contracts",
        &cache_rebuild,
        &[
            "use super::*;",
            "fn query_state_cache_rebuild_uses_access_reads_without_per_rebuild_merge",
        ],
    );
    assert_contains_all(
        "archetype/access child owns sorted index and boolean conflict contracts",
        &archetype_access,
        &[
            "use super::*;",
            "fn archetype_index_matching_reuses_sorted_component_index_without_per_query_resort",
            "fn query_access_conflicts_with_uses_allocation_free_boolean_path",
        ],
    );
    assert_contains_all(
        "combinations child owns cached combination iterator contracts",
        &combinations,
        &[
            "use super::*;",
            "fn cached_combinations_trust_query_state_data_membership",
        ],
    );

    let migrated_test_count = [
        query_state_layout.as_str(),
        mutable_iterators.as_str(),
        cached_iterators.as_str(),
        cache_rebuild.as_str(),
        archetype_access.as_str(),
        combinations.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 11,
        "scene ECS query structure child owners should preserve all 11 tests moved out of the parent"
    );

    for (path, source) in [
        ("scene/tests/ecs_query_structure.rs", parent.as_str()),
        (
            "scene/tests/ecs_query_structure/archetype_access.rs",
            archetype_access.as_str(),
        ),
        (
            "scene/tests/ecs_query_structure/cache_rebuild.rs",
            cache_rebuild.as_str(),
        ),
        (
            "scene/tests/ecs_query_structure/cached_iterators.rs",
            cached_iterators.as_str(),
        ),
        (
            "scene/tests/ecs_query_structure/combinations.rs",
            combinations.as_str(),
        ),
        (
            "scene/tests/ecs_query_structure/mutable_iterators.rs",
            mutable_iterators.as_str(),
        ),
        (
            "scene/tests/ecs_query_structure/query_state_layout.rs",
            query_state_layout.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("ECS doc", ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 scene ECS query structure test folder split",
                "runtime_15_scene_ecs_query_structure_tests_folder_split_static_passed_cargo_deferred",
                "scene/tests/ecs_query_structure.rs",
                "scene/tests/ecs_query_structure/cache_rebuild.rs",
                "scene/tests/ecs_query_structure/cached_iterators.rs",
                "runtime_15_scene_ecs_query_structure_tests_are_folder_backed",
            ],
        );
    }
}
