use super::*;

#[test]
fn runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let query_state_dir = manifest_root.join("src/scene/ecs/query/query_state");
    let retired_helpers = query_state_dir.join("helpers.rs");
    let query_state_mod = read_text(
        &query_state_dir.join("mod.rs"),
        "scene ECS query-state module entry should be readable",
    );
    let many_item_array = read_text(
        &query_state_dir.join("many_item_array.rs"),
        "scene ECS query-state many-item array owner should be readable",
    );
    let query_state_callers = [
        read_text(
            &query_state_dir.join("cached_direct.rs"),
            "scene ECS cached direct query-state owner should be readable",
        ),
        read_text(
            &query_state_dir.join("mutable.rs"),
            "scene ECS mutable query-state owner should be readable",
        ),
        read_text(
            &query_state_dir.join("read_only.rs"),
            "scene ECS read-only query-state owner should be readable",
        ),
        read_text(
            &query_state_dir.join("read_only_cached.rs"),
            "scene ECS cached read-only query-state owner should be readable",
        ),
    ];
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/core_scene_asset_dynamic.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/scene_asset_runtime.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/scene_asset_runtime.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_helpers.exists(),
        "scene ECS query-state should not keep banned-name module file {:?}",
        retired_helpers
    );
    assert_contains_all(
        "scene ECS query-state module entry",
        &query_state_mod,
        &["mod many_item_array;"],
    );
    assert!(
        !query_state_mod.contains("mod helpers;"),
        "scene/ecs/query/query_state/mod.rs should not preserve the banned helpers module name"
    );
    assert_contains_all(
        "scene ECS query-state many-item array owner",
        &many_item_array,
        &[
            "pub(super) fn collect_many_query_items",
            "MaybeUninit<Item>",
            "assume_init_drop",
            "*const [Item; N]",
        ],
    );
    for caller in &query_state_callers {
        assert_contains_all(
            "scene ECS query-state caller",
            caller,
            &["super::many_item_array::collect_many_query_items"],
        );
        assert!(
            !caller.contains("super::helpers"),
            "query-state callers should consume many_item_array instead of helpers"
        );
    }

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
                "Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover",
                "runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred",
                "scene/ecs/query/query_state/many_item_array.rs",
                "runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
            ],
        );
    }
}
