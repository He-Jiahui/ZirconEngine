use super::*;

const ASSERTIONS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions.rs";
const ASSERTION_LINE_BUDGETS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/line_budgets.rs";
const ASSERTION_PRE_RUNTIME_15_MAPS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs";
const ASSERTION_RUNTIME_15_MAPS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/runtime_15_maps.rs";
const ASSERTION_STATUS_AND_DOCS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/assertions/status_and_docs.rs";

#[test]
fn runtime_15_top_level_expected_slice_assertion_helpers_are_child_owned() {
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
    let assertion_line_budgets = read_runtime_src(ASSERTION_LINE_BUDGETS_OWNER);
    let assertion_pre_runtime_15_maps = read_runtime_src(ASSERTION_PRE_RUNTIME_15_MAPS_OWNER);
    let assertion_runtime_15_maps = read_runtime_src(ASSERTION_RUNTIME_15_MAPS_OWNER);
    let assertion_status_and_docs = read_runtime_src(ASSERTION_STATUS_AND_DOCS_OWNER);

    assert_contains_all(
        "top-level map assertions child mounts focused assertion helpers",
        &assertions,
        &[
            "#[path = \"assertions/line_budgets.rs\"]",
            "mod line_budgets;",
            "#[path = \"assertions/pre_runtime_15_maps.rs\"]",
            "mod pre_runtime_15_maps;",
            "#[path = \"assertions/runtime_15_maps.rs\"]",
            "mod runtime_15_maps;",
            "#[path = \"assertions/status_and_docs.rs\"]",
            "mod status_and_docs;",
            concat!(
                "pub(super) fn assert_expected_slice_maps_",
                "are_child_owners"
            ),
            "runtime_15_maps::assert_runtime_15_maps(sources)",
            "pre_runtime_15_maps::assert_pre_runtime_15_maps(sources)",
            "line_budgets::assert_line_budgets(sources)",
            "status_and_docs::assert_status_and_docs(sources)",
        ],
    );
    for moved_anchor in [
        "fn assert_runtime_15_maps(",
        "fn assert_pre_runtime_15_maps(",
        "fn assert_line_budgets(",
        "fn assert_status_and_docs(",
        concat!(
            "Runtime 15 status expected-slice topic owners ",
            "preserve representative literals"
        ),
        "pre-Runtime-15 date expected-slice children own legacy date literals",
        "status-output Runtime 15 row data",
    ] {
        assert!(
            !assertions.contains(moved_anchor),
            "top_level_maps/assertions.rs should mount focused assertion helpers instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 assertion helper child owns Runtime 15 map checks",
        &assertion_runtime_15_maps,
        &[
            concat!(
                "Runtime 15 status expected-slice topic owners ",
                "preserve representative literals"
            ),
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "runtime_15_status_output_expected_slice_maps_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 assertion helper child owns legacy map checks",
        &assertion_pre_runtime_15_maps,
        &[
            "pre-Runtime-15 date expected-slice children own legacy date literals",
            "Runtime 14 animation runtime-status focused recheck timeout",
            "Runtime 12 input boundary grouped manager import guard repair",
        ],
    );
    assert_contains_all(
        "line-budget assertion helper child owns budget checks",
        &assertion_line_budgets,
        &[
            "pub(super) fn assert_line_budgets(sources: &TopLevelMapSources)",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
            "line_count < 800",
        ],
    );
    assert_contains_all(
        "status-and-docs assertion helper child owns mirror checks",
        &assertion_status_and_docs,
        &[
            "pub(super) fn assert_status_and_docs(sources: &TopLevelMapSources)",
            "status-output Runtime 15 row data",
            "Runtime 15 M3 status output expected-slice maps split",
            "runtime_15_status_output_expected_slice_maps_are_child_owners",
        ],
    );
}
