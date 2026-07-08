use super::*;

pub(super) fn typed_error_structure_assertions_child_tree() -> String {
    [
        read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER),
        typed_error_convergence_mount_nested_child_tree(),
        read_runtime_src(TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER),
        typed_error_native_plugin_loader_nested_child_tree(),
    ]
    .join("\n")
}

pub(super) fn typed_error_native_structure_child_tree() -> String {
    [
        read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER),
        typed_error_native_plugin_loader_nested_child_tree(),
        typed_error_native_plugin_loader_route_child_tree(),
    ]
    .join("\n")
}

pub(super) fn typed_error_moved_guard_absence_child_tree() -> String {
    [
        read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/child_ownership.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/preserved_guards.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/parent_backflow.rs"),
        typed_error_moved_guard_absence_parent_backflow_child_tree(),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/path_anchors.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/budgets.rs"),
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/status_mirrors.rs"),
    ]
    .join("\n")
}

pub(super) fn typed_error_source_inventory_child_tree() -> String {
    [
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER),
        typed_error_source_inventory_nested_child_tree(),
    ]
    .join("\n")
}

fn typed_error_convergence_mount_nested_child_tree() -> String {
    [
        "top_level",
        "asset_parents",
        "runtime_parents",
        "budgets",
        "status_mirrors",
        "root_paths",
        "root_statuses",
        "root_child_rows",
        "root_sources",
        "root_inventory",
    ]
    .into_iter()
    .map(|child| {
        read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts/{child}.rs"
        ))
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn typed_error_native_plugin_loader_nested_child_tree() -> String {
    [
        "budgets",
        "child_inventory",
        "delegation",
        "metadata",
        "routes",
        "source_helper_ownership",
        "source_helper_status",
        "sources",
        "status_mirrors",
    ]
    .into_iter()
    .map(|child| {
        read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/{child}.rs"
        ))
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn typed_error_moved_guard_absence_parent_backflow_child_tree() -> String {
    [
        "parent_paths",
        "guard_names",
        "guard_body",
        "metadata",
        "child_inventory",
        "sources",
        "child_ownership",
        "status_current",
    ]
    .into_iter()
    .map(|child| {
        read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence/parent_backflow/{child}.rs"
        ))
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn typed_error_source_inventory_nested_child_tree() -> String {
    [
        "budgets",
        "child_inventory",
        "child_sources",
        "delegation",
        "metadata",
        "paths",
        "reads",
        "source_helper_ownership",
        "source_helper_status",
        "status_mirrors",
    ]
    .into_iter()
    .map(|child| {
        read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/{child}.rs"
        ))
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn typed_error_native_plugin_loader_route_child_tree() -> String {
    [
        "abi_surfaces",
        "child_inventory",
        "child_ownership",
        "lifecycle_paths",
        "live_host",
        "manifest_sources",
        "metadata",
        "plugin_descriptor",
        "replay_runtime",
        "source_helper_ownership",
        "source_helper_status",
        "sources",
        "status_current",
        "top_level",
    ]
    .into_iter()
    .map(|child| {
        read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader/routes/{child}.rs"
        ))
    })
    .collect::<Vec<_>>()
    .join("\n")
}
