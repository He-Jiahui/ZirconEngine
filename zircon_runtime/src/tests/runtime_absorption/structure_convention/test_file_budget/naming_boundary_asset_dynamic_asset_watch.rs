use super::*;

const STATUS: &str =
    "runtime_15_asset_dynamic_asset_watch_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 asset-dynamic asset-watch guard child-owner split";
const GUARD: &str = "runtime_15_asset_dynamic_asset_watch_guards_are_child_owner";

#[test]
fn runtime_15_asset_dynamic_asset_watch_guards_are_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/asset_watch.rs",
    );

    assert_contains_all(
        "Runtime 15 asset-dynamic naming parent mounts asset-watch child owner",
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
        "fn runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
        "fn runtime_15_asset_change_construction_uses_owner_name",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/asset_dynamic.rs should mount asset_watch child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 asset-dynamic asset-watch child owns asset watcher naming guards",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
            "fn runtime_15_asset_change_construction_uses_owner_name",
            "asset/watch/shutdown_on_drop.rs",
            "asset/watch/asset_change_construction.rs",
            "runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
            "runtime_15_asset_change_construction_uses_owner_name",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/asset_watch.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
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
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/asset_watch.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record asset-dynamic asset-watch child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
