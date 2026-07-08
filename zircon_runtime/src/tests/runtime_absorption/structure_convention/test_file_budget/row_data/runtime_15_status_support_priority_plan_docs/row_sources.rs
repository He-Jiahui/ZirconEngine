use super::*;

#[test]
fn runtime_15_priority_plan_docs_row_data_owner_is_child_backed() {
    let parent = read_runtime_src(PRIORITY_ROW_PARENT);
    let child_sources: Vec<(&str, String)> = PRIORITY_ROW_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect();

    assert_contains_all(
        "priority-plan-doc row-data parent mounts child row owners",
        &parent,
        &[
            "#[path = \"priority_plan_docs/integrity_guards.rs\"]",
            "#[path = \"priority_plan_docs/owner_guards.rs\"]",
            "#[path = \"priority_plan_docs/status_followups.rs\"]",
            "#[path = \"priority_plan_docs/row_data_owner.rs\"]",
            "integrity_guards::EXPECTED_STATUS_OUTPUT_SLICES",
            "owner_guards::EXPECTED_STATUS_OUTPUT_SLICES",
            "status_followups::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !parent.contains("pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &["),
        "priority_plan_docs.rs should route child row owners instead of owning row tuples directly"
    );

    for (module_name, child_path, representative_row) in PRIORITY_ROW_CHILDREN {
        assert!(
            parent.contains(&format!("mod {module_name};")),
            "priority_plan_docs.rs should mount `{module_name}`"
        );
        let child = child_sources
            .iter()
            .find(|(path, _)| *path == *child_path)
            .map(|(_, source)| source.as_str())
            .expect("child source should be loaded");
        assert_contains_all(
            child_path,
            child,
            &["type Slice = super::Slice;", representative_row],
        );
    }
}

#[test]
fn runtime_15_priority_plan_docs_owner_guard_rows_are_child_owned() {
    let owner_parent_path = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs";
    let owner_parent = read_runtime_src(owner_parent_path);
    let row_data_owner = read_runtime_src(PRIORITY_ROW_DATA_OWNER_PATH);

    assert_contains_all(
        "priority-plan-doc owner guard row-data parent mounts child row owners",
        &owner_parent,
        &[
            "#[path = \"owner_guards/inventory_rows.rs\"]",
            "#[path = \"owner_guards/layout_rows.rs\"]",
            "layout_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "inventory_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "OWNER_GUARD_ROW_ANCHOR_MIRROR",
        ],
    );
    assert!(
        !owner_parent.contains("pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &["),
        "owner_guards.rs should route owner-guard row children instead of owning row tuples directly"
    );

    for (module_name, child_path, representative_row) in PRIORITY_OWNER_GUARD_ROW_CHILDREN {
        assert!(
            owner_parent.contains(&format!("mod {module_name};")),
            "owner_guards.rs should mount `{module_name}`"
        );
        let child = read_runtime_src(child_path);
        assert_contains_all(
            child_path,
            &child,
            &["type Slice = super::Slice;", representative_row],
        );
        assert!(
            !child.contains("OWNER_GUARD_ROW_ANCHOR_MIRROR"),
            "{child_path} should own rows directly instead of relying on the parent anchor mirror"
        );
    }

    assert_contains_all(
        "priority-plan-doc row-data owner records owner-guard child split",
        &row_data_owner,
        &[
            OWNER_GUARD_CHILD_STATUS_NAME,
            OWNER_GUARD_CHILD_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs",
            OWNER_GUARD_CHILD_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
}
