use super::super::super::super::*;
use super::*;

pub(super) struct TypedErrorChildOwnershipSources {
    pub(super) parent: String,
    pub(super) child: String,
    pub(super) child_ownership_child: String,
    pub(super) structure_assertions_child: String,
    pub(super) convergence_mounts_child: String,
    pub(super) delegation_child: String,
    pub(super) child_ownership_structure_child: String,
    pub(super) status_mirrors_child: String,
    pub(super) moved_guard_absence_child: String,
    pub(super) moved_guard_absence_child_tree: String,
    pub(super) native_plugin_loader_child: String,
    pub(super) structure_guard_typed_error_child: String,
    pub(super) typed_error_sources: String,
}

pub(super) fn typed_error_child_ownership_sources() -> TypedErrorChildOwnershipSources {
    let moved_guard_absence_child =
        read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);
    let moved_guard_absence_preserved_guards_child =
        read_runtime_src(MOVED_GUARD_ABSENCE_PRESERVED_GUARDS_CHILD);
    let moved_guard_absence_parent_backflow_child =
        read_runtime_src(MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD);
    let moved_guard_absence_path_anchors_child =
        read_runtime_src(MOVED_GUARD_ABSENCE_PATH_ANCHORS_CHILD);
    let moved_guard_absence_budgets_child = read_runtime_src(MOVED_GUARD_ABSENCE_BUDGETS_CHILD);
    let moved_guard_absence_status_mirrors_child =
        read_runtime_src(MOVED_GUARD_ABSENCE_STATUS_MIRRORS_CHILD);
    let moved_guard_absence_child_tree = [
        moved_guard_absence_child.as_str(),
        moved_guard_absence_preserved_guards_child.as_str(),
        moved_guard_absence_parent_backflow_child.as_str(),
        moved_guard_absence_path_anchors_child.as_str(),
        moved_guard_absence_budgets_child.as_str(),
        moved_guard_absence_status_mirrors_child.as_str(),
    ]
    .join("\n");

    TypedErrorChildOwnershipSources {
        parent: read_runtime_src(STRUCTURE_GUARD_PARENT),
        child: read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD),
        child_ownership_child: read_runtime_src(TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD),
        structure_assertions_child: read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD),
        convergence_mounts_child: read_runtime_src(TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD),
        delegation_child: read_runtime_src(TYPED_ERROR_STRUCTURE_DELEGATION_CHILD),
        child_ownership_structure_child: read_runtime_src(
            TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
        ),
        status_mirrors_child: read_runtime_src(TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD),
        moved_guard_absence_child,
        moved_guard_absence_child_tree,
        native_plugin_loader_child: read_runtime_src(TYPED_ERROR_NATIVE_STRUCTURE_CHILD),
        structure_guard_typed_error_child: read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD),
        typed_error_sources: typed_error_children_source(),
    }
}

pub(super) fn typed_error_structure_assertions_child_tree(
    sources: &TypedErrorChildOwnershipSources,
) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        sources.structure_assertions_child,
        sources.convergence_mounts_child,
        sources.delegation_child,
        sources.child_ownership_structure_child,
        sources.status_mirrors_child,
        sources.moved_guard_absence_child,
        sources.native_plugin_loader_child
    )
}

pub(super) fn typed_error_child_ownership_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_CHILD_OWNERSHIP_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_child_ownership_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in typed_error_child_ownership_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn typed_error_child_ownership_status_row_source() -> String {
    format!(
        "{}\n{}",
        read_runtime_src(TYPED_ERROR_CHILD_OWNERSHIP_STATUS_ROW_PARENT),
        read_runtime_src(TYPED_ERROR_CHILD_OWNERSHIP_STATUS_ROW_CHILD),
    )
}
