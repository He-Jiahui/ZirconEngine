use super::*;

const STATUS: &str = "runtime_15_asset_dynamic_dynamic_api_vampire_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 asset-dynamic dynamic-API vampire guard child-owner split";
const GUARD: &str = "runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner";

#[test]
fn runtime_15_asset_dynamic_dynamic_api_vampire_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs",
    );

    assert_contains_all(
        "Runtime 15 asset-dynamic naming parent mounts dynamic-API vampire child owner",
        &parent,
        &[
            "#[path = \"asset_dynamic/asset_watch.rs\"]",
            "mod asset_watch;",
            "#[path = \"asset_dynamic/dynamic_api_vampire.rs\"]",
            "mod dynamic_api_vampire;",
            "#[path = \"asset_dynamic/scene_ecs_queries.rs\"]",
            "mod scene_ecs_queries;",
            "#[path = \"asset_dynamic/texture_containers.rs\"]",
            "mod texture_containers;",
        ],
    );
    for moved_test in [
        "fn runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name",
        "fn runtime_15_scene_ecs_query_cached_queries_uses_owner_name",
        "fn runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name",
        "fn runtime_15_dds_upload_policy_uses_classic_container_names",
        "fn runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
        "fn runtime_15_asset_change_construction_uses_owner_name",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/asset_dynamic.rs should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 asset-dynamic dynamic-API vampire child owns runtime support naming guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name",
            "dynamic_api/session/tests/vampire_runtime_support.rs",
            "runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-19-dynamic-api-filter-plan-anchor-current-owner.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );

    assert_contains_all_exact(
        "Runtime 15 dynamic-API filter current child owner",
        &current_anchor_owner,
        &[
            SLICE,
            STATUS,
            GUARD,
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs",
        ],
    );
    for (label, source) in [
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record asset-dynamic dynamic-API vampire child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
