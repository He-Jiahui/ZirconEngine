use super::*;

const STATUS: &str =
    "runtime_15_banned_names_scene_dynamic_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 banned-name scene-dynamic guard child-owner split";
const GUARD: &str = "runtime_15_banned_names_scene_dynamic_guard_is_child_owner";

#[test]
fn runtime_15_banned_names_scene_dynamic_guard_is_child_owner() {
    let parent =
        read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs",
    );

    assert_contains_all(
        "Runtime 15 banned-names parent mounts scene-dynamic child owner",
        &parent,
        &[
            "#[path = \"banned_names/scene_dynamic.rs\"]",
            "mod scene_dynamic;",
            "#[path = \"banned_names/global_modules.rs\"]",
            "mod global_modules;",
            "#[path = \"banned_names/graphics_construction.rs\"]",
            "mod graphics_construction;",
        ],
    );
    assert!(
        !parent.contains("fn runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name"),
        "runtime_15_m2/banned_names.rs should mount scene_dynamic child instead of defining the scene dynamic document guard"
    );
    assert!(
        !parent.contains("fn runtime_15_graphics_construction_new_owners_use_construct_names"),
        "runtime_15_m2/banned_names.rs should mount graphics_construction child instead of defining the graphics construction guard"
    );
    assert!(
        !parent.contains("fn runtime_15_no_banned_name_modules"),
        "runtime_15_m2/banned_names.rs should mount global_modules child instead of defining the global banned-name guard"
    );

    assert_contains_all(
        "Runtime 15 banned-names scene-dynamic child owns value migration hard-cut guard",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_scene_dynamic_document_uses_value_migration_owner",
            "v1_project_document.rs",
            "migration_dir",
            "migrate_project_world",
            "from_value::<World>",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs",
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
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                GUARD,
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/graphics_construction.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record banned-names scene-dynamic child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-06-30"],
    );
}
