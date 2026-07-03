use super::*;

#[test]
fn runtime_15_scene_ecs_query_cached_queries_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ecs_query_dir = manifest_root.join("src/scene/tests/ecs_query");
    let retired_cache_helpers = ecs_query_dir.join("cache_helpers.rs");
    let ecs_query_parent = read_text(
        &manifest_root.join("src/scene/tests/ecs_query.rs"),
        "scene ECS query test module parent should be readable",
    );
    let cached_queries = read_text(
        &ecs_query_dir.join("cached_queries.rs"),
        "scene ECS query cached queries owner should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let scene_ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_cache_helpers.exists(),
        "scene ECS query tests should not keep banned-name module file {:?}",
        retired_cache_helpers
    );
    assert_contains_all(
        "scene ECS query test parent",
        &ecs_query_parent,
        &["mod cached_queries;"],
    );
    assert!(
        !ecs_query_parent.contains("mod cache_helpers;"),
        "scene/tests/ecs_query.rs should not preserve the banned cache_helpers module name"
    );
    assert_contains_all(
        "scene ECS query cached queries owner",
        &cached_queries,
        &[
            "fn query_state_cached_iteration_rebuilds_only_for_structural_changes",
            "fn query_state_count_and_empty_helpers_can_use_cached_candidates",
            "fn query_state_cached_direct_iteration_reads_storage_locations",
            "fn query_state_cached_direct_iteration_reads_sparse_locations",
            "fn query_state_cached_archetypes_do_not_require_optional_reads",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", scene_ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene ECS query cached queries module naming hard cutover",
                "runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred",
                "scene/tests/ecs_query/cached_queries.rs",
                "runtime_15_scene_ecs_query_cached_queries_uses_owner_name",
            ],
        );
    }
}
