use super::*;

#[path = "maps/runtime_15_topics.rs"]
mod runtime_15_topics;
#[path = "maps/top_level_maps.rs"]
mod top_level_maps;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");
const TOP_LEVEL_MAP_GUARD: &str = concat!(
    "fn runtime_15_status_output_expected_slice_",
    "maps_are_child_owners"
);
const RUNTIME_15_TOPIC_MAP_GUARD: &str = concat!(
    "fn runtime_15_status_output_runtime_15_expected_slice_",
    "maps_are_child_owners"
);
const STRUCTURE_SUPPORT_MAP_GUARD: &str = concat!(
    "fn runtime_15_structure_support_expected_slice_",
    "maps_are_child_owners"
);

#[test]
fn runtime_15_status_output_expected_slice_guard_maps_are_child_owners() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
    );
    let top_level_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
    );
    let top_level_map_support_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/support_layout.rs",
    );
    let runtime_15_topics = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
    );
    let runtime_15_topic_expected_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps.rs",
    );
    let runtime_15_topic_review_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps.rs",
    );
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
        "status-output expected-slice maps parent mounts child owners",
        &parent,
        &[
            "mod runtime_15_topics;",
            "mod top_level_maps;",
            "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice topic parent mounts child owners",
        &runtime_15_topics,
        &[
            "#[path = \"runtime_15_topics/review_guard_maps.rs\"]",
            "mod review_guard_maps;",
            "#[path = \"runtime_15_topics/runtime_15_expected_slice_maps.rs\"]",
            "mod runtime_15_expected_slice_maps;",
        ],
    );
    for moved_guard in [TOP_LEVEL_MAP_GUARD, RUNTIME_15_TOPIC_MAP_GUARD] {
        assert!(
            !parent.contains(moved_guard),
            "status_output_expected_slices/maps.rs should mount child owners instead of defining {moved_guard}"
        );
    }
    for moved_anchor in [
        RUNTIME_15_TOPIC_MAP_GUARD,
        concat!("let status_runtime_15 = ", "read_runtime_src("),
        concat!(
            "Runtime 15 status expected-slice children ",
            "own topic literals"
        ),
    ] {
        assert!(
            !runtime_15_topics.contains(moved_anchor),
            "maps/runtime_15_topics.rs should mount child owners instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-output expected-slice map children preserve guards",
        &format!(
            "{top_level_maps}\n{top_level_map_support_layout}\n{runtime_15_topics}\n{runtime_15_topic_expected_maps}\n{runtime_15_topic_review_maps}"
        ),
        &[
            TOP_LEVEL_MAP_GUARD,
            RUNTIME_15_TOPIC_MAP_GUARD,
            STRUCTURE_SUPPORT_MAP_GUARD,
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
        ],
    );

    let test_count = parent.matches(TEST_ATTRIBUTE).count()
        + top_level_maps.matches(TEST_ATTRIBUTE).count()
        + top_level_map_support_layout.matches(TEST_ATTRIBUTE).count()
        + runtime_15_topics.matches(TEST_ATTRIBUTE).count()
        + runtime_15_topic_expected_maps
            .matches(TEST_ATTRIBUTE)
            .count()
        + runtime_15_topic_review_maps.matches(TEST_ATTRIBUTE).count();
    assert_eq!(
        test_count, 5,
        "status-output expected-slice guard parent plus children should preserve the map guards plus the parent layout guard and top-level support layout guard"
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
            top_level_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/support_layout.rs",
            top_level_map_support_layout.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
            runtime_15_topics.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps.rs",
            runtime_15_topic_expected_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/review_guard_maps.rs",
            runtime_15_topic_review_maps.as_str(),
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
                "Runtime 15 M3 status output expected-slice guard maps child-owner split",
                "runtime_15_status_output_expected_slice_guard_maps_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
                "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 runtime-15 expected-slice topic guard child-module split",
                "runtime_15_runtime_15_expected_slice_topic_guard_child_module_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/maps/runtime_15_topics/runtime_15_expected_slice_maps.rs",
                "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
                "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
    }
}
