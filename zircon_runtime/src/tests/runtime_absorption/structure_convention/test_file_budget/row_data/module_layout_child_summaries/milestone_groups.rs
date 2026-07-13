use super::*;

#[path = "milestone_groups/m3_child_groups.rs"]
mod m3_child_groups;
#[path = "milestone_groups/runtime_row_data.rs"]
mod runtime_row_data;
#[path = "milestone_groups/status_doc_groups.rs"]
mod status_doc_groups;

const MILESTONE_GROUPS_ROUTE_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups.rs";
const RUNTIME_ROW_DATA_GROUPS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/runtime_row_data.rs";
const M3_CHILD_GROUPS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/m3_child_groups.rs";
const STATUS_DOC_GROUPS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/status_doc_groups.rs";

const MILESTONE_GROUP_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 module-layout child-summary milestone-groups child split";
const MILESTONE_GROUP_SPLIT_STATUS_ID: &str = "runtime_15_module_layout_child_summary_milestone_groups_child_split_static_passed_cargo_deferred";

const MILESTONE_GROUP_CHILDREN: &[(&str, &str, &str, &[&str])] = &[
    (
        "runtime_row_data",
        RUNTIME_ROW_DATA_GROUPS_PATH,
        "runtime_15_module_layout_child_summary_runtime_row_data_groups_are_child_owned",
        &[
            "Runtime 15 M4 row-data child owns M4 split guard",
            "Runtime 15 M2 row-data child owns M2 split guard",
            "Runtime 15 M3 row-data child owns M3 split guard",
        ],
    ),
    (
        "m3_child_groups",
        M3_CHILD_GROUPS_PATH,
        "runtime_15_module_layout_child_summary_m3_child_groups_are_child_owned",
        &[
            "Runtime 15 M3 child-groups guard owns M3 child split guard",
            "Runtime 15 M3 child-group moved-row child owns moved-row assertions",
            "Runtime 15 M3 child-group status-doc child owns status/doc anchors",
            "Runtime 15 M3 child-group status-row-doc child owns row status/doc anchors",
        ],
    ),
    (
        "status_doc_groups",
        STATUS_DOC_GROUPS_PATH,
        "runtime_15_module_layout_child_summary_status_doc_groups_are_child_owned",
        &[
            "Runtime 15 module-layout status-doc child owns status/doc anchors",
            "Runtime 15 module-layout child-summary status-doc child owns status/doc anchors",
        ],
    ),
];

#[test]
fn runtime_15_module_layout_child_summary_milestone_groups_are_child_owner() {
    let child_summary_parent = read_runtime_src(CHILD_SUMMARY_PARENT_PATH);
    let route_source = read_runtime_src(MILESTONE_GROUPS_ROUTE_PATH);

    for (_, _, _, labels) in MILESTONE_GROUP_CHILDREN {
        for delegated_summary in *labels {
            assert!(
                !child_summary_parent.contains(delegated_summary),
                "module_layout_child_summaries.rs should delegate {delegated_summary}"
            );
        }
    }

    for (module_name, path, guard_name, labels) in MILESTONE_GROUP_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "module-layout child-summary milestone-groups route mounts child",
            &route_source,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*guard_name]);
        assert_contains_all(path, &child_source, labels);
    }
    assert!(
        !route_source.contains(concat!("let runtime_15_m4", "_row_data_parent =")),
        "milestone_groups.rs should delegate milestone row-data reads to child files"
    );
}

#[test]
fn runtime_15_module_layout_child_summary_milestone_group_children_are_status_recorded() {
    let production_guard_support =
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH);
    let expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        MILESTONE_GROUP_SPLIT_STATUS_NAME,
        MILESTONE_GROUP_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/runtime_row_data.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/m3_child_groups.rs",
        "structure_convention/test_file_budget/row_data/module_layout_child_summaries/milestone_groups/status_doc_groups.rs",
        "runtime_15_module_layout_child_summary_milestone_groups_are_child_owner",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production support row data records milestone-groups child split",
        &production_guard_support,
        &status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support map records milestone-groups child split",
        &expected_status_map,
        &[
            MILESTONE_GROUP_SPLIT_STATUS_NAME,
            MILESTONE_GROUP_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 date map records milestone-groups child split",
        &expected_date_map,
        &[MILESTONE_GROUP_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
