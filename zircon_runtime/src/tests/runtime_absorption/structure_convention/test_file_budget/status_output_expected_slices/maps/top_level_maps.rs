use super::*;

#[path = "top_level_maps/assertions.rs"]
mod assertions;
#[path = "top_level_maps/sources.rs"]
mod sources;

const ASSERTIONS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs";
const SOURCES_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/sources.rs";

#[test]
fn runtime_15_status_output_expected_slice_maps_are_child_owners() {
    let sources = sources::read_top_level_map_sources();

    assertions::assert_expected_slice_maps_are_child_owners(&sources);
}

#[test]
fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
    );
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
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
            "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
            "fn runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
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
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "maps/top_level_maps.rs should mount support child owners instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "top-level map assertions child owns expected-slice checks",
        &assertions,
        &[
            concat!(
                "pub(super) fn assert_expected_slice_maps_",
                "are_child_owners"
            ),
            concat!(
                "Runtime 15 status expected-slice topic owners ",
                "preserve representative literals"
            ),
            "pre-Runtime-15 date expected-slice children own legacy date literals",
            "status-output Runtime 15 row data",
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
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
            assertions.as_str(),
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
    }
}
