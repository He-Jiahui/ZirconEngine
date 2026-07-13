use super::*;

const STATUS: &str =
    "runtime_15_ui_platform_input_guard_child_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 UI platform-input guard child-owner split";
const GUARD: &str = "runtime_15_ui_platform_input_guards_are_child_owner";

#[test]
fn runtime_15_ui_platform_input_guards_are_child_owner() {
    let parent = read_runtime_src("tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs");
    let child = read_runtime_src(
        "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui/platform_input.rs",
    );

    assert_contains_all(
        "Runtime 15 UI naming parent mounts platform-input child owner",
        &parent,
        &[
            "#[path = \"ui/platform_input.rs\"]",
            "mod platform_input;",
            "fn runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name",
            "fn runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context",
            "fn runtime_15_ui_template_schema_uses_source_fixture_names",
        ],
    );
    for moved_test in [
        "fn runtime_15_platform_input_uses_dom_keycode_names",
        "fn runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
    ] {
        assert!(
            !parent.contains(moved_test),
            "runtime_15_m2/ui.rs should mount the platform_input child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "Runtime 15 UI platform-input child owns platform input naming guards",
        &child,
        &[
            "use super::*;",
            "fn runtime_15_platform_input_uses_dom_keycode_names",
            "fn runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
            "dom_key_code",
            "runtime_input_baseline",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui/platform_input.rs",
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
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs",
                "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui/platform_input.rs",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record UI platform-input child owner",
        &format!("{status_map}\n{date_map}"),
        &[SLICE, STATUS, "2026-07-01"],
    );
}
