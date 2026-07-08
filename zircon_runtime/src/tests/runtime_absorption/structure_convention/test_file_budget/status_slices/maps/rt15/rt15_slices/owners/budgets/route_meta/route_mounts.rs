use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_is_child_owned(
) {
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{BUDGETS_ROUTE_PATH}"));
    let sources = read_runtime_src(&format!("tests/runtime_absorption/{BUDGETS_SOURCES_PATH}"));
    let guard_body = read_runtime_src(&format!(
        "tests/runtime_absorption/{BUDGETS_GUARD_BODY_PATH}"
    ));
    let route_metadata = read_runtime_src(&format!(
        "tests/runtime_absorption/{BUDGETS_ROUTE_METADATA_PATH}"
    ));
    let route_metadata_children = read_budget_route_metadata_children();

    assert_contains_all(
        "Runtime 15 expected-slice budget route parent",
        &parent,
        &[
            "#[path = \"budgets/sources.rs\"]",
            "mod sources;",
            "#[path = \"budgets/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"budgets/route_metadata.rs\"]",
            "mod route_metadata;",
            "use sources::*;",
        ],
    );
    for moved_anchor in [
        "EXPECTED_SLICE_BUDGET_SOURCE_PATHS",
        "#[test]",
        "runtime_15_expected_slice_child_owner_sources_stay_budgeted",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "owners/budgets.rs should delegate moved budget route metadata {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice budget route children",
        &format!("{sources}\n{guard_body}\n{route_metadata}\n{route_metadata_children}"),
        &[
            "EXPECTED_SLICE_BUDGET_SOURCE_PATHS",
            "runtime_15_expected_slice_child_owner_sources_stay_budgeted",
            BUDGET_GUARD,
        ],
    );
}
