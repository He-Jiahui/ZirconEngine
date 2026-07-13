use super::*;

#[path = "owner_budgets/nested_children.rs"]
mod nested_children;
#[path = "owner_budgets/route_children.rs"]
mod route_children;
#[path = "owner_budgets/status_mirrors.rs"]
mod status_mirrors;
#[path = "owner_budgets/surrounding_owners.rs"]
mod surrounding_owners;

const OWNER_BUDGET_PARENT_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets.rs";
const OWNER_BUDGET_ROUTE_CHILDREN_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/route_children.rs";
const OWNER_BUDGET_NESTED_CHILDREN_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/nested_children.rs";
const OWNER_BUDGET_SURROUNDING_OWNERS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/surrounding_owners.rs";
const OWNER_BUDGET_STATUS_MIRRORS_PATH: &str = "tests/runtime_absorption/structure_convention/test_file_budget/row_data/module_layout_child_summaries/owner_budgets/status_mirrors.rs";

const OWNER_BUDGET_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "route_children",
        OWNER_BUDGET_ROUTE_CHILDREN_PATH,
        "assert_module_layout_child_summary_route_owner_budgets_are_current",
    ),
    (
        "nested_children",
        OWNER_BUDGET_NESTED_CHILDREN_PATH,
        "assert_module_layout_child_summary_nested_budgets_are_current",
    ),
    (
        "surrounding_owners",
        OWNER_BUDGET_SURROUNDING_OWNERS_PATH,
        "assert_module_layout_child_summary_surrounding_owner_budgets_are_current",
    ),
    (
        "status_mirrors",
        OWNER_BUDGET_STATUS_MIRRORS_PATH,
        "runtime_15_module_layout_child_summary_owner_budget_guard_child_split_status_is_current",
    ),
];

#[test]
fn runtime_15_module_layout_child_summary_guard_owner_budgets_are_child_owned() {
    let parent = read_runtime_src(OWNER_BUDGET_PARENT_PATH);
    for (module_name, path, guard_name) in OWNER_BUDGET_CHILDREN {
        let module_mount = format!("#[path = \"owner_budgets/{module_name}.rs\"]");
        assert_contains_all(
            "module-layout child-summary owner-budget guard mounts child",
            &parent,
            &[module_mount.as_str(), *path, *guard_name],
        );
        let child = read_runtime_src(path);
        assert_contains_all(path, &child, &[*guard_name]);
    }

    route_children::assert_module_layout_child_summary_route_owner_budgets_are_current();
    nested_children::assert_module_layout_child_summary_nested_budgets_are_current();
    surrounding_owners::assert_module_layout_child_summary_surrounding_owner_budgets_are_current();
}

fn owner_budget_child_source_blob() -> String {
    OWNER_BUDGET_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n")
}
