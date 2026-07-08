use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_source_constants_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{}",
        REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_ROUTE_PATH
    ));
    let children = read_runtime_absorption_sources(REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN);

    assert_contains_all(
        "review-route route metadata source constants parent mounts children",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/child_sources.rs\"]",
            "mod child_sources;",
            "#[path = \"route_meta/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"route_meta/guard_routes.rs\"]",
            "mod guard_routes;",
            "#[path = \"route_meta/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"route_meta/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"route_meta/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"route_meta/structure_routes.rs\"]",
            "mod structure_routes;",
        ],
    );

    for moved_anchor in [
        "REVIEW_ROUTE_CHILD_SOURCES_SLICE",
        "REVIEW_ROUTE_METADATA_BUDGETS_SLICE",
        "REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_SLICE",
        "REVIEW_ROUTE_METADATA_STATUS_MIRRORS_SLICE",
        "STRUCTURE_REVIEW_CHILD_ROUTE_PARENT",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review-route route metadata source constants parent should delegate {moved_anchor}"
        );
    }

    assert_contains_all(
        "review-route route metadata source constants children",
        &children,
        &[
            REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_GUARD,
            "REVIEW_ROUTE_CHILD_SOURCES_SLICE",
            "REVIEW_ROUTE_METADATA_BUDGETS_SLICE",
            "REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_FOLDER_BACKED_SLICE",
            "REVIEW_ROUTE_METADATA_STATUS_MIRRORS_SLICE",
            "STRUCTURE_REVIEW_CHILD_ROUTE_PARENT",
        ],
    );

    for (path, limit) in [
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[0], 35usize),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[1], 70),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[2], 90),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[3], 30),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[4], 55),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[5], 95),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[6], 30),
        (REVIEW_ROUTE_METADATA_SOURCE_CONSTANTS_CHILDREN[7], 30),
    ] {
        let line_count = read_runtime_src(&format!("tests/runtime_absorption/{path}"))
            .lines()
            .count();
        assert!(
            line_count < limit,
            "{path} should stay below the review-route route metadata source constants budget {limit}; got {line_count} lines"
        );
    }
}
