use super::*;

#[test]
fn runtime_15_code_review_structure_guard_root_and_children_row_data_is_child_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_ROOT_AND_CHILDREN_PATH);
    let child_blob = structure_guard_root_and_children_source_blob();
    let row_data_owner = read_runtime_src(STRUCTURE_GUARD_ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "structure-guard root-and-children parent mounts child row owners",
        &parent,
        &[
            "#[path = \"root_and_children/code_review_findings.rs\"]",
            "#[path = \"root_and_children/p0_robustness.rs\"]",
            "#[path = \"root_and_children/plugin_importer_dx.rs\"]",
            "#[path = \"root_and_children/p0_native_fixture.rs\"]",
            "#[path = \"root_and_children/f8_child_owner.rs\"]",
            "#[path = \"root_and_children/late_api_cleanup.rs\"]",
            "code_review_findings::STRUCTURE_GUARD_CHILD_OWNER_SPLIT",
            "code_review_findings::STRUCTURE_GUARD_CHILDREN_FOLDER_BACKED_SPLIT",
            "p0_robustness::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT",
            "plugin_importer_dx::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT",
            "p0_native_fixture::LEAF_OWNER_GUARD_FOLDER_BACKED_SPLIT",
            "f8_child_owner::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT",
            "late_api_cleanup::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT",
        ],
    );
    assert!(
        !parent.contains(
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_host_callbacks.rs"
        ),
        "root_and_children.rs should route child owners instead of owning long row evidence lists"
    );
    assert_contains_all(
        "structure-guard root-and-children child owners keep representative row evidence",
        &child_blob,
        &[
            "runtime_15_code_review_findings_structure_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_code_review_findings_structure_guard_children_folder_backed_static_passed_cargo_deferred",
            "runtime_15_p0_robustness_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_dx_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_f8_child_owner_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_late_api_cleanup_structure_guard_folder_backed_static_passed_cargo_deferred",
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-guard row-data owner records root-and-children child split status",
        &row_data_owner,
        &[
            ROOT_AND_CHILDREN_ROW_DATA_STATUS_NAME,
            ROOT_AND_CHILDREN_ROW_DATA_STATUS_ID,
            ROOT_AND_CHILDREN_ROW_DATA_GUARD_NAME,
        ],
    );
}
