use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_maps_children_preserve_guard_owners() {
    let top_level_maps = read_runtime_src(TOP_LEVEL_MAPS_PATH);
    let top_level_map_support_layout = read_runtime_src(TOP_LEVEL_SUPPORT_LAYOUT_PATH);
    let runtime_15_topics = read_runtime_src(RUNTIME_15_TOPICS_PATH);
    let runtime_15_topic_expected_maps = read_runtime_src(RUNTIME_15_TOPIC_EXPECTED_MAPS_PATH);
    let runtime_15_topic_review_maps = read_runtime_src(RUNTIME_15_TOPIC_REVIEW_MAPS_PATH);
    let runtime_15_topic_parent_routes = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/split/body/mounts/parent_routes.rs",
    );
    let review_topic_parent_routes = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/body/mounts/parent_routes.rs",
    );
    let structure_support_split_layout = read_runtime_src(STRUCTURE_SUPPORT_SPLIT_LAYOUT_PATH);
    let child_sources = GUARD_BODY_CHILD_PATHS
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "status-output expected-slice map children preserve moved guards",
        &format!(
            "{top_level_maps}\n{top_level_map_support_layout}\n{runtime_15_topics}\n{runtime_15_topic_expected_maps}\n{runtime_15_topic_review_maps}\n{runtime_15_topic_parent_routes}\n{review_topic_parent_routes}\n{structure_support_split_layout}\n{child_sources}"
        ),
        &[
            TOP_LEVEL_MAP_GUARD,
            RUNTIME_15_TOPIC_MAP_GUARD,
            "#[path = \"rt15_slices/child_owners.rs\"]",
            "#[path = \"review/structure_support_expected_slice.rs\"]",
            "runtime_15_status_output_runtime_15_expected_slice_maps_guard_is_folder_backed",
            "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
            "runtime_15_structure_support_expected_slice_guard_is_folder_backed",
            "runtime_15_structure_support_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_status_output_expected_slice_top_level_map_support_child_owners_are_folder_backed",
            "runtime_15_status_output_expected_slice_guard_maps_are_child_owners",
            MAPS_GUARD_BODY_GUARD,
        ],
    );
}
