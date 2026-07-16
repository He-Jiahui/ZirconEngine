const SLICE: &str = "Runtime 15 M3 generated-code guard folder-backed split";
const STATUS: &str = "runtime_15_generated_code_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_generated_code_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_generated_code_guard_is_folder_backed";

const PARENT_PATH: &str = "generated_code_guard.rs";
const CHILD_PATHS: &[&str] = &[
    "generated_code_guard/behavior.rs",
    "generated_code_guard/delegation.rs",
    "generated_code_guard/markers.rs",
    "generated_code_guard/scope.rs",
    "generated_code_guard/support.rs",
    "generated_code_guard/split_layout.rs",
];

#[test]
fn runtime_15_generated_code_guard_is_folder_backed() {
    let parent = include_str!("../generated_code_guard.rs");
    let children = [
        include_str!("behavior.rs"),
        include_str!("delegation.rs"),
        include_str!("markers.rs"),
        include_str!("scope.rs"),
        include_str!("support.rs"),
        include_str!("split_layout.rs"),
    ];

    assert_contains_all(
        "generated-code guard parent routes child owners",
        parent,
        &[
            "mod behavior;",
            "mod delegation;",
            "mod markers;",
            "mod scope;",
            "mod split_layout;",
            "mod support;",
        ],
    );

    for moved_anchor in [
        "GENERATED_MARKER_PREFIX",
        "GeneratedBehaviorDecision",
        "export_template_scan_scope_stays_folder_backed",
        "export_entry_templates_delegate_to_app_export_bootstrap_facade",
        "collect_rust_source_files",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "generated-code guard parent should not retain moved owner `{moved_anchor}`"
        );
        assert!(
            children.iter().any(|source| source.contains(moved_anchor)),
            "generated-code guard children should own moved owner `{moved_anchor}`"
        );
    }

    for (path, source, max_lines) in [
        (PARENT_PATH, parent, 30usize),
        (CHILD_PATHS[0], children[0], 210),
        (CHILD_PATHS[1], children[1], 130),
        (CHILD_PATHS[2], children[2], 130),
        (CHILD_PATHS[3], children[3], 90),
        (CHILD_PATHS[4], children[4], 80),
        (CHILD_PATHS[5], children[5], 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{path} has {line_count} lines, expected <= {max_lines}"
        );
    }

    let row_data = include_str!(
        "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/structure_guard_rows.rs"
    );
    assert_contains_all(
        "module-convention row data records generated-code guard split",
        row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_PATHS[0],
            CHILD_PATHS[5],
            GUARD,
        ],
    );

    let status_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs"
    );
    assert_contains_all("structure route status map", status_map, &[SLICE, STATUS]);

    let date_map = include_str!(
        "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs"
    );
    assert_contains_all("structure route date map", date_map, &[SLICE, "2026-07-05"]);

    for (label, source) in [
        (
            "Runtime 15 subplan",
            crate::tests::runtime_absorption::current_source_fixture::RUNTIME_ARCHITECTURE_IMPLEMENTATION_OUTPUT,
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
        ),
        (
            "engine code review findings",
            include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        ),
        (
            "module convention doc",
            include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
    ] {
        assert_contains_all(label, source, &[SLICE, STATUS, GUARD, CHILD_PATHS[5]]);
    }

    let frameworks = include_str!(
        "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
    );
    assert_contains_all(
        "frameworks plan records generated-code guard split",
        frameworks,
        &[SLICE, STATUS, FRAMEWORKS_STATUS, GUARD],
    );
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .copied()
        .filter(|needle| !source.contains(needle))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing expected anchors:\n{}",
        missing.join("\n")
    );
}
