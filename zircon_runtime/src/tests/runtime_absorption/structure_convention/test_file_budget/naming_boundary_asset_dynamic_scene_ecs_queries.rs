use super::*;

const STATUS: &str =
    "runtime_15_asset_dynamic_scene_ecs_query_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 asset-dynamic scene-ECS query guard child-owner split";
const GUARD: &str = "runtime_15_asset_dynamic_scene_ecs_query_guard_is_child_owner";

#[test]
fn runtime_15_asset_dynamic_scene_ecs_query_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/scene_ecs_queries.rs",
    );

    assert_contains_all(
        "Runtime 15 asset-dynamic naming parent mounts scene-ECS query child owner",
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
        "Runtime 15 asset-dynamic scene-ECS child owns cached query naming guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_scene_ecs_query_cached_queries_uses_owner_name",
            "scene/tests/ecs_query/cached_queries.rs",
            "runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/scene_ecs_queries.rs",
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
}
