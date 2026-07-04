use super::super::super::super::*;

#[path = "convergence_mounts/asset_parents.rs"]
mod asset_parents;
#[path = "convergence_mounts/budgets.rs"]
mod budgets;
#[path = "convergence_mounts/root_child_rows.rs"]
mod root_child_rows;
#[path = "convergence_mounts/root_inventory.rs"]
mod root_inventory;
#[path = "convergence_mounts/root_paths.rs"]
mod root_paths;
#[path = "convergence_mounts/root_sources.rs"]
mod root_sources;
#[path = "convergence_mounts/root_statuses.rs"]
mod root_statuses;
#[path = "convergence_mounts/runtime_parents.rs"]
mod runtime_parents;
#[path = "convergence_mounts/status_mirrors.rs"]
mod status_mirrors;
#[path = "convergence_mounts/top_level.rs"]
mod top_level;

pub(super) use root_child_rows::*;
pub(super) use root_paths::*;
pub(super) use root_sources::*;
pub(super) use root_statuses::*;

pub(super) fn assert_typed_error_convergence_parents_are_folder_backed() {
    let sources = typed_error_convergence_mount_sources();
    top_level::assert_typed_error_convergence_top_level_parent_is_folder_backed(&sources);
    asset_parents::assert_typed_error_asset_parents_are_folder_backed(&sources);
    runtime_parents::assert_typed_error_runtime_parents_are_folder_backed(&sources);
    budgets::assert_typed_error_convergence_mount_budgets_are_focused(&sources);
}

#[test]
fn runtime_15_typed_error_convergence_mounts_guard_is_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD);
    let child_inventory = read_runtime_src(TYPED_ERROR_CONVERGENCE_MOUNTS_ROOT_CHILD_ROWS_CHILD);
    let child_tree = typed_error_convergence_mount_child_source_blob();
    let sources = typed_error_convergence_mount_sources();

    for (module_name, child_path, anchor) in TYPED_ERROR_CONVERGENCE_MOUNT_CHILDREN {
        let path_attr = format!("#[path = \"convergence_mounts/{module_name}.rs\"]");
        assert! {
            parent.contains(&path_attr),
            "typed-error convergence mounts parent should mount {module_name}"
        };
        assert! {
            child_inventory.contains(child_path),
            "typed-error convergence mounts child inventory should list {child_path}"
        };
        assert! {
            child_tree.contains(anchor),
            "typed-error convergence mounts child {child_path} should own anchor {anchor}"
        };
    }
    assert_typed_error_convergence_parents_are_folder_backed();
    budgets::assert_typed_error_convergence_mount_budgets_are_focused(&sources);
}
