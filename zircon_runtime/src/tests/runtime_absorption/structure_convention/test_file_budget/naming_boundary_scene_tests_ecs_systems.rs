use super::*;

const STATUS: &str =
    "runtime_15_scene_tests_ecs_systems_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 scene-tests ECS systems guard child-owner split";
const GUARD: &str = "runtime_15_scene_tests_ecs_systems_guard_is_child_owner";

#[test]
fn runtime_15_scene_tests_ecs_systems_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests/ecs_systems.rs",
    );

    assert_contains_all(
        "Runtime 15 scene-tests naming parent mounts ECS systems child",
        &parent,
        &[
            "#[path = \"scene_tests/ecs_systems.rs\"]",
            "mod ecs_systems;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name"),
        "runtime_15_m2/scene_tests.rs should mount the ECS systems child instead of defining the naming guard"
    );

    assert_contains_all(
        "Runtime 15 scene-tests ECS systems child owns guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name",
            "scene/tests/ecs_systems/many_single_queries.rs",
            "runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_timeout_no_result",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests/ecs_systems.rs",
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
