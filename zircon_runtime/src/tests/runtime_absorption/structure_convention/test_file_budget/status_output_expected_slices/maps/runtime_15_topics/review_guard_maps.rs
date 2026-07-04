use super::*;

#[path = "review_guard_maps/status_support_expected_slice.rs"]
mod status_support_expected_slice;
#[path = "review_guard_maps/structure_support_expected_slice.rs"]
mod structure_support_expected_slice;
#[path = "review_guard_maps/typed_error_expected_slice.rs"]
mod typed_error_expected_slice;

const STATUS_PARENT: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs";
const DATE_PARENT: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs";
const STATUS_REVIEW_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
const STATUS_REVIEW_TYPED_ERROR_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps.rs";
const STATUS_NAMING_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs";
const STATUS_SUPPORT_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
const STATUS_SUPPORT_ROW_DATA_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps.rs";
const STATUS_SUPPORT_PLAN_DOC_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps.rs";
const DATE_REVIEW_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
const DATE_REVIEW_TYPED_ERROR_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps.rs";
const DATE_NAMING_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs";
const DATE_SUPPORT_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";
const DATE_SUPPORT_ROW_DATA_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps.rs";
const DATE_SUPPORT_PLAN_DOC_CHILD: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps.rs";
const STRUCTURE_REVIEW_GUARD_PARENT: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps.rs";
const STRUCTURE_REVIEW_STRUCTURE_SUPPORT_GUARD_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/structure_support_expected_slice.rs";
const STRUCTURE_REVIEW_TYPED_ERROR_GUARD_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/typed_error_expected_slice.rs";
const STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/status_support_expected_slice.rs";

#[test]
fn runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned() {
    let structure_parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_PARENT);
    let structure_support_guard_child =
        read_runtime_src(STRUCTURE_REVIEW_STRUCTURE_SUPPORT_GUARD_CHILD);
    let typed_error_guard_child = read_runtime_src(STRUCTURE_REVIEW_TYPED_ERROR_GUARD_CHILD);
    let status_support_guard_child = read_runtime_src(STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_CHILD);
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
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
        "review_guard_maps.rs mounts expected-slice guard children",
        &structure_parent,
        &[
            "#[path = \"review_guard_maps/structure_support_expected_slice.rs\"]",
            "mod structure_support_expected_slice;",
            "#[path = \"review_guard_maps/status_support_expected_slice.rs\"]",
            "mod status_support_expected_slice;",
            "#[path = \"review_guard_maps/typed_error_expected_slice.rs\"]",
            "mod typed_error_expected_slice;",
        ],
    );
    for moved_anchor in [
        concat!(
            "fn ",
            "runtime_15_structure_support_expected_slice_maps_are_child_owners"
        ),
        concat!(
            "M3 structure-support",
            " status expected-slice parent mounts map children"
        ),
        concat!(
            "Runtime 15 M3 structure-support",
            " expected-slice map child-owner split"
        ),
    ] {
        assert!(
            !structure_parent.contains(moved_anchor),
            "review_guard_maps.rs should mount structure_support_expected_slice instead of keeping {moved_anchor}"
        );
    }

    for (path, source) in [
        (STRUCTURE_REVIEW_GUARD_PARENT, structure_parent.as_str()),
        (
            STRUCTURE_REVIEW_STRUCTURE_SUPPORT_GUARD_CHILD,
            structure_support_guard_child.as_str(),
        ),
        (
            STRUCTURE_REVIEW_TYPED_ERROR_GUARD_CHILD,
            typed_error_guard_child.as_str(),
        ),
        (
            STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_CHILD,
            status_support_guard_child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 structure guard budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "expected-slice guard children own moved tests",
        &format!("{structure_support_guard_child}\n{typed_error_guard_child}\n{status_support_guard_child}"),
        &[
            "runtime_15_structure_support_expected_slice_maps_are_child_owners",
            "runtime_15_review_guard_expected_slice_typed_error_maps_are_child_owned",
            "runtime_15_status_support_expected_slice_maps_are_child_owned",
            concat!(
                "Runtime 15 M3 structure-support",
                " expected-slice map child-owner split"
            ),
            "Runtime 15 M3 review guard typed-error expected-slice map child split",
            "Runtime 15 M3 status-support expected-slice map child split",
        ],
    );

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
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
                "Runtime 15 M3 review-guard expected-slice structure guard child-module split",
                "runtime_15_review_guard_expected_slice_structure_guard_child_module_split_static_passed_cargo_deferred",
                "Runtime 15 M3 structure-support expected-slice guard body child split",
                "runtime_15_structure_support_expected_slice_guard_body_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/structure_support_expected_slice.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/typed_error_expected_slice.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps/status_support_expected_slice.rs",
                "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
    }
}
