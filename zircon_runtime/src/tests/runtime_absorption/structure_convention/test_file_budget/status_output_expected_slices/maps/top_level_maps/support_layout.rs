use super::*;

const ASSERTIONS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs";
const ASSERTION_LINE_BUDGETS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/line_budgets.rs";
const ASSERTION_PRE_RUNTIME_15_MAPS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs";
const ASSERTION_RUNTIME_15_MAPS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/runtime_15_maps.rs";
const ASSERTION_STATUS_AND_DOCS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/status_and_docs.rs";
const SOURCES_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs";

#[test]
fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
    );
    let support_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/support_layout.rs",
    );
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
    let assertion_line_budgets = read_runtime_src(ASSERTION_LINE_BUDGETS_OWNER);
    let assertion_pre_runtime_15_maps = read_runtime_src(ASSERTION_PRE_RUNTIME_15_MAPS_OWNER);
    let assertion_runtime_15_maps = read_runtime_src(ASSERTION_RUNTIME_15_MAPS_OWNER);
    let assertion_status_and_docs = read_runtime_src(ASSERTION_STATUS_AND_DOCS_OWNER);
    let sources = read_runtime_src(SOURCES_OWNER);
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
        "top-level expected-slice map parent mounts support owners",
        &parent,
        &[
            "#[path = \"top_level_maps/assertions.rs\"]",
            "mod assertions;",
            "#[path = \"top_level_maps/sources.rs\"]",
            "mod sources;",
            "#[path = \"top_level_maps/support_layout.rs\"]",
            "mod support_layout;",
            "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
        ],
    );
    for moved_anchor in [
        concat!("pub(super) struct ", "TopLevelMapSources"),
        concat!(
            "pub(super) fn assert_expected_slice_maps_",
            "are_child_owners"
        ),
        concat!("let status_parent = ", "read_runtime_src("),
        concat!(
            "Runtime 15 status expected-slice child ",
            "delegates topic owners"
        ),
        "fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
        "Runtime 15 M3 top-level expected-slice support-layout guard body child split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "maps/top_level_maps.rs should mount support child owners instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "top-level support-layout child owns moved guard body",
        &support_layout,
        &[
            "fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
            "Runtime 15 M3 top-level expected-slice support-layout guard body child split",
            "runtime_15_top_level_expected_slice_support_layout_guard_body_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/support_layout.rs",
        ],
    );
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
    assert_contains_all(
        "top-level map sources child owns source reads",
        &sources,
        &[
            concat!("pub(super) struct ", "TopLevelMapSources"),
            "pub(super) fn read_top_level_map_sources",
            "expected_slices/status/runtime_15/foundation.rs",
            "expected_slices/date/pre_runtime_15/runtime_11_14.rs",
            "structure_convention/test_file_budget/status_output_expected_slices.rs",
        ],
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/support_layout.rs",
            support_layout.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
            assertions.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/line_budgets.rs",
            assertion_line_budgets.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs",
            assertion_pre_runtime_15_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/runtime_15_maps.rs",
            assertion_runtime_15_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/status_and_docs.rs",
            assertion_status_and_docs.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs",
            sources.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused guard budget; got {line_count} lines"
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
                "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
                "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs",
                "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 top-level expected-slice assertion helper child split",
                "runtime_15_top_level_expected_slice_assertion_helper_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/runtime_15_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/pre_runtime_15_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/line_budgets.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions/status_and_docs.rs",
                "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 top-level expected-slice support-layout guard body child split",
                "runtime_15_top_level_expected_slice_support_layout_guard_body_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/support_layout.rs",
                "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
    }
}
