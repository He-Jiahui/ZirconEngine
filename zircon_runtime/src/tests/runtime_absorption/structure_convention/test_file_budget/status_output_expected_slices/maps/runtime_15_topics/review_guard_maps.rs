use super::*;

const STATUS_PARENT: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs";
const DATE_PARENT: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs";
const STATUS_REVIEW_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
const STATUS_NAMING_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs";
const STATUS_SUPPORT_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
const DATE_REVIEW_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
const DATE_NAMING_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs";
const DATE_SUPPORT_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

#[test]
fn runtime_15_structure_support_expected_slice_maps_are_child_owners() {
    let status_parent = read_runtime_src(STATUS_PARENT);
    let date_parent = read_runtime_src(DATE_PARENT);
    let status_review_child = read_runtime_src(STATUS_REVIEW_CHILD);
    let status_naming_child = read_runtime_src(STATUS_NAMING_CHILD);
    let status_support_child = read_runtime_src(STATUS_SUPPORT_CHILD);
    let date_review_child = read_runtime_src(DATE_REVIEW_CHILD);
    let date_naming_child = read_runtime_src(DATE_NAMING_CHILD);
    let date_support_child = read_runtime_src(DATE_SUPPORT_CHILD);
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "M3 structure-support status expected-slice parent mounts map children",
        &status_parent,
        &[
            "#[path = \"m3_structure_support/review_guard_maps.rs\"]",
            "mod review_guard_maps;",
            "#[path = \"m3_structure_support/naming_guard_maps.rs\"]",
            "mod naming_guard_maps;",
            "#[path = \"m3_structure_support/status_support_maps.rs\"]",
            "mod status_support_maps;",
            "review_guard_maps::expected_status_for_slice(slice)",
            "naming_guard_maps::expected_status_for_slice(slice)",
            "status_support_maps::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "M3 structure-support date expected-slice parent mounts map children",
        &date_parent,
        &[
            "#[path = \"m3_structure_support/review_guard_maps.rs\"]",
            "mod review_guard_maps;",
            "#[path = \"m3_structure_support/naming_guard_maps.rs\"]",
            "mod naming_guard_maps;",
            "#[path = \"m3_structure_support/status_support_maps.rs\"]",
            "mod status_support_maps;",
            "review_guard_maps::expected_date_for_slice(slice)",
            "naming_guard_maps::expected_date_for_slice(slice)",
            "status_support_maps::expected_date_for_slice(slice)",
        ],
    );
    for moved_literal in [
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split",
        "Runtime 15 M3 graphics GPU-model guard child-owner split",
        "Runtime 15 M3 status output expected-slice guard child-owner split",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "M3 structure-support status parent should delegate {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "M3 structure-support date parent should delegate {moved_literal}"
        );
    }

    assert_contains_all(
        "review expected-slice children own review guard literals",
        &format!("{status_review_child}\n{date_review_child}"),
        &[
            "Runtime 15 M3 P0 robustness review guard child-owner split",
            "runtime_15_native_plugin_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 D12 runtime helper export macro review sync",
            "Some(\"2026-06-30\")",
        ],
    );
    assert_contains_all(
        "naming expected-slice children own naming guard literals",
        &format!("{status_naming_child}\n{date_naming_child}"),
        &[
            "Runtime 15 M3 graphics GPU-model guard child-owner split",
            "runtime_15_banned_names_global_module_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 plugin static manifest naming guard child-owner split",
            "Some(\"2026-06-30\")",
        ],
    );
    assert_contains_all(
        "status-support expected-slice children own status-support literals",
        &format!("{status_support_child}\n{date_support_child}"),
        &[
            "Runtime 15 M3 status output expected-slice guard child-owner split",
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 structure-support expected-slice map child-owner split",
            "runtime_15_structure_support_expected_slice_map_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-30\")",
        ],
    );

    for (path, source) in [
        (STATUS_PARENT, status_parent.as_str()),
        (DATE_PARENT, date_parent.as_str()),
        (STATUS_REVIEW_CHILD, status_review_child.as_str()),
        (STATUS_NAMING_CHILD, status_naming_child.as_str()),
        (STATUS_SUPPORT_CHILD, status_support_child.as_str()),
        (DATE_REVIEW_CHILD, date_review_child.as_str()),
        (DATE_NAMING_CHILD, date_naming_child.as_str()),
        (DATE_SUPPORT_CHILD, date_support_child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused expected-slice budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("status-output M3 row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 structure-support expected-slice map child-owner split",
                "runtime_15_structure_support_expected_slice_map_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
                "runtime_15_structure_support_expected_slice_maps_are_child_owners",
            ],
        );
    }
}
