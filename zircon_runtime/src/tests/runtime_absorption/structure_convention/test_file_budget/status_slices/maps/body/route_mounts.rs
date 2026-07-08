use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_maps_are_child_owners() {
    let parent = read_runtime_src(MAPS_PARENT_PATH);
    let guard_body = read_runtime_src(GUARD_BODY_PARENT_PATH);
    let runtime_15_topics = read_runtime_src(RUNTIME_15_TOPICS_PATH);

    assert_contains_all(
        "status-output expected-slice maps parent mounts child owners",
        &parent,
        &[
            "#[path = \"maps/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"maps/runtime_15_topics.rs\"]",
            "mod runtime_15_topics;",
            "#[path = \"maps/top_level_maps.rs\"]",
            "mod top_level_maps;",
        ],
    );
    assert_contains_all(
        "status-output expected-slice maps guard body mounts focused children",
        &guard_body,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/paths.rs\"]",
            "mod paths;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice topic parent mounts child owners",
        &runtime_15_topics,
        &[
            "#[path = \"rt15/review_guard_maps.rs\"]",
            "mod review_guard_maps;",
            "#[path = \"rt15/runtime_15_expected_slice_maps.rs\"]",
            "mod runtime_15_expected_slice_maps;",
        ],
    );
    for moved_guard in [TOP_LEVEL_MAP_GUARD, RUNTIME_15_TOPIC_MAP_GUARD] {
        assert!(
            !parent.contains(moved_guard),
            "status_slices/maps.rs should mount child owners instead of defining {moved_guard}"
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
}
