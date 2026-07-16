use super::*;

#[test]
fn runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ecs_systems_dir = manifest_root.join("src/scene/tests/ecs_systems");
    let retired_query_helpers = ecs_systems_dir.join("query_helpers.rs");
    let ecs_systems_parent = read_text(
        &manifest_root.join("src/scene/tests/ecs_systems.rs"),
        "scene ECS systems parent should be readable",
    );
    let many_single_queries = read_text(
        &ecs_systems_dir.join("many_single_queries.rs"),
        "scene ECS systems many/single query owner should be readable",
    );
    let test_budget_guard = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs",
        ),
        "scene ECS systems test-budget guard should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

    assert!(
        !retired_query_helpers.exists(),
        "scene ECS systems should not keep banned-name module file {:?}",
        retired_query_helpers
    );
    assert_contains_all(
        "scene ECS systems parent",
        &ecs_systems_parent,
        &["mod many_single_queries;"],
    );
    assert!(
        !ecs_systems_parent.contains("mod query_helpers;"),
        "scene/tests/ecs_systems.rs should not preserve the banned query_helpers module name"
    );
    assert_contains_all(
        "scene ECS systems many/single query owner",
        &many_single_queries,
        &[
            "fn system_query_get_many_helpers_preserve_order_duplicates_and_run_window_filters",
            "fn system_query_iter_many_preserves_order_duplicates_and_run_window_filters",
            "fn system_query_single_helpers_report_zero_one_many_matches",
        ],
    );
    assert_contains_all(
        "scene ECS systems test-budget guard",
        &test_budget_guard,
        &[
            "scene/tests/ecs_systems/many_single_queries.rs",
            "mod many_single_queries;",
        ],
    );
    assert!(
        !test_budget_guard.contains("scene/tests/ecs_systems/query_helpers.rs"),
        "scene ECS systems test-budget guard should not keep retired query_helpers path"
    );

    let docs = [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("scene ECS doc", ecs_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ];
    for (label, source) in docs {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 scene ECS systems many/single queries module naming hard cutover",
                "runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "scene/tests/ecs_systems/many_single_queries.rs",
                "runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name",
            ],
        );
    }
}
