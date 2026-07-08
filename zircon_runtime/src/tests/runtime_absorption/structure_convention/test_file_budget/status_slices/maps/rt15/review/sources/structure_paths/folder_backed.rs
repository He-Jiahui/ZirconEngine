use super::*;

#[test]
fn runtime_15_review_guard_source_structure_paths_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{}",
        STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_ROUTE_PATH
    ));
    let children =
        read_review_structure_path_sources(STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN);

    assert_contains_all(
        "review guard source structure paths parent mounts children",
        &parent,
        &[
            "#[path = \"structure_paths/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"structure_paths/foundation_maps.rs\"]",
            "mod foundation_maps;",
            "#[path = \"structure_paths/review_route.rs\"]",
            "mod review_route;",
            "#[path = \"structure_paths/root_routes.rs\"]",
            "mod root_routes;",
            "#[path = \"structure_paths/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"structure_paths/route_metadata.rs\"]",
            "mod route_metadata;",
            "#[path = \"structure_paths/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"structure_paths/structure_support.rs\"]",
            "mod structure_support;",
            "#[path = \"structure_paths/typed_status_support.rs\"]",
            "mod typed_status_support;",
        ],
    );

    for moved_anchor in [
        "STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN",
        "STRUCTURE_REVIEW_FOUNDATION_MAPS_CHILDREN",
        "STRUCTURE_REVIEW_STRUCTURE_SUPPORT_LITERAL_CHILDREN",
        "STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_BODY_CHILDREN",
        "STRUCTURE_REVIEW_ROUTE_GUARD_BODY_CHILDREN",
        "STRUCTURE_REVIEW_ROUTE_METADATA_STATUS_MIRROR_CHILDREN",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review guard source structure paths parent should delegate {moved_anchor}"
        );
    }

    assert_contains_all(
        "review guard source structure paths children",
        &children,
        &[
            STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_GUARD,
            "STRUCTURE_REVIEW_GUARD_SOURCE_CHILDREN",
            "STRUCTURE_REVIEW_FOUNDATION_MAPS_CHILDREN",
            "STRUCTURE_REVIEW_STRUCTURE_SUPPORT_LITERAL_CHILDREN",
            "STRUCTURE_REVIEW_STATUS_SUPPORT_GUARD_BODY_CHILDREN",
            "STRUCTURE_REVIEW_ROUTE_GUARD_BODY_CHILDREN",
            "STRUCTURE_REVIEW_ROUTE_METADATA_STATUS_MIRROR_CHILDREN",
        ],
    );

    for (path, limit) in [
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[0], 95usize),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[1], 30),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[2], 40),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[3], 15),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[4], 60),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[5], 35),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[6], 105),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[7], 40),
        (STRUCTURE_REVIEW_SOURCE_STRUCTURE_PATHS_CHILDREN[8], 35),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review guard source structure paths budget {limit}; got {line_count} lines"
        );
    }
}
