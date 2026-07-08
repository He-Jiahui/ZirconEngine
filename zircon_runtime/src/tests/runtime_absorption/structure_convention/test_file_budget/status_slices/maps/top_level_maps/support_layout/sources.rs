use super::*;

const SOURCES_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/top_level_maps/sources.rs";

#[test]
fn runtime_15_top_level_expected_slice_sources_are_child_owned() {
    let sources = read_runtime_src(SOURCES_OWNER);

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
}
