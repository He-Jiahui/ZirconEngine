use super::*;

#[test]
fn runtime_15_review_guard_root_source_route_children_are_child_owned() {
    let parent = include_str!("../route_children.rs");
    let budgets = read_route_child(SOURCE_ROUTE_CHILDREN_CHILDREN[0]);
    let route_inventory = read_route_child(SOURCE_ROUTE_CHILDREN_CHILDREN[2]);
    let source_reads = read_route_child(SOURCE_ROUTE_CHILDREN_CHILDREN[3]);
    let status_metadata = read_route_child(SOURCE_ROUTE_CHILDREN_CHILDREN[4]);
    let structure_rows = read_route_child(SOURCE_ROUTE_CHILDREN_CHILDREN[6]);

    assert_contains_all(
        "review guard source route-children delegates child owners",
        parent,
        &[
            "#[path = \"route_children/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_children/ownership.rs\"]",
            "mod ownership;",
            "#[path = \"route_children/route_inventory.rs\"]",
            "mod route_inventory;",
            "#[path = \"route_children/source_reads.rs\"]",
            "mod source_reads;",
            "#[path = \"route_children/status_metadata.rs\"]",
            "mod status_metadata;",
            "#[path = \"route_children/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"route_children/structure_rows.rs\"]",
            "mod structure_rows;",
            "pub(in super::super) use route_inventory::*;",
            "pub(in super::super) use source_reads::*;",
            "pub(in super::super) use status_metadata::*;",
            "pub(in super::super) use structure_rows::*;",
        ],
    );

    for moved_anchor in [
        "pub(in super::super) const STRUCTURE_REVIEW_ROUTE_CHILDREN",
        "pub(in super::super) fn read_review_root_sources",
        "pub(in super::super) fn read_review_guard_structure_rows",
        "pub(in super::super) const STRUCTURE_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN",
        "pub(in super::super) fn read_structure_support_expected_slice_rows",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "sources/route_children.rs should delegate {moved_anchor}"
        );
    }

    assert_contains_all(
        "route-children status metadata owns status constants",
        &status_metadata,
        &[
            "SOURCE_ROUTE_CHILDREN_SLICE",
            "SOURCE_ROUTE_CHILDREN_STATUS",
            "SOURCE_ROUTE_CHILDREN_FRAMEWORKS_STATUS",
            "SOURCE_ROUTE_CHILDREN_GUARD",
            "SOURCE_ROUTE_CHILDREN_ROUTE_PATH",
            "SOURCE_ROUTE_CHILDREN_CHILDREN",
        ],
    );
    assert_contains_all(
        "route-children budgets own child line budgets",
        &budgets,
        &[
            "runtime_15_review_guard_root_source_route_children_children_stay_budgeted",
            "SOURCE_ROUTE_CHILDREN_CHILDREN",
            "route-children child budget",
        ],
    );
    assert_contains_all(
        "route inventory owns root child list",
        &route_inventory,
        &[
            "pub(in super::super::super) const STRUCTURE_REVIEW_ROUTE_CHILDREN",
            "STRUCTURE_REVIEW_GUARD_SOURCES",
            "STRUCTURE_REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN",
        ],
    );
    assert_contains_all(
        "source reads own root source helper",
        &source_reads,
        &[
            "pub(in super::super::super) fn read_review_root_sources",
            "read_runtime_src(path)",
        ],
    );
    assert_contains_all(
        "structure rows own row aggregation helpers",
        &structure_rows,
        &[
            "pub(in super::super::super) const STRUCTURE_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN",
            "pub(in super::super::super) fn read_review_guard_structure_rows",
            "pub(in super::super::super) fn read_structure_support_expected_slice_rows",
            "STRUCTURE_SUPPORT_ROW_DATA_OWNER_ROW_CHILDREN",
        ],
    );
}

fn read_route_child(path: &str) -> String {
    read_runtime_src(&format!("tests/runtime_absorption/{path}"))
}
