use super::*;

fn snapshot(
    workspace: &AssetWorkspaceState,
) -> crate::ui::workbench::snapshot::AssetWorkspaceSnapshot {
    workspace.build_snapshot(AssetSurfaceMode::Explorer)
}

#[test]
fn asset_tree_disclosure_collapses_descendants_and_keeps_a_visible_selection() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog_generation());
    workspace.select_folder("res://physics/materials");
    workspace.select_asset(Some("55555555-5555-5555-5555-555555555555".into()));
    let before = snapshot(&workspace);
    assert!(workspace.set_folder_expanded("res://physics", false, before.catalog_revision));
    let collapsed = snapshot(&workspace);
    assert_eq!(
        collapsed.selected_folder_id.as_deref(),
        Some("res://physics")
    );
    assert!(collapsed.selected_asset_uuid.is_none());
    assert!(!collapsed
        .folder_tree
        .iter()
        .any(|folder| folder.folder_id == "res://physics/materials"));
    assert_ne!(before.catalog_revision, collapsed.catalog_revision);
    assert!(workspace.set_folder_expanded("res://physics", true, collapsed.catalog_revision));
    let expanded = snapshot(&workspace);
    assert!(expanded
        .folder_tree
        .iter()
        .any(|folder| folder.folder_id == "res://physics/materials"));
    assert_eq!(expanded.selected_folder_id, collapsed.selected_folder_id);
}

#[test]
fn asset_tree_disclosure_expansion_does_not_select_an_unrelated_branch() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog_generation());
    workspace.select_folder("res://textures");
    workspace.select_asset(Some("33333333-3333-3333-3333-333333333333".into()));
    let before = snapshot(&workspace);
    assert!(workspace.set_folder_expanded("res://physics", false, before.catalog_revision));
    let collapsed = snapshot(&workspace);
    assert_eq!(before.selected_folder_id, collapsed.selected_folder_id);
    assert_eq!(before.selected_asset_uuid, collapsed.selected_asset_uuid);
    assert!(workspace.set_folder_expanded("res://physics", true, collapsed.catalog_revision));
    assert_eq!(
        snapshot(&workspace).selected_asset_uuid,
        before.selected_asset_uuid
    );
}

#[test]
fn asset_tree_disclosure_rejects_leaf_missing_and_stale_publications() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog_generation());
    let before = snapshot(&workspace);
    // This leaf contains an asset, which must not create an expand affordance.
    let leaf = before
        .folder_tree
        .iter()
        .find(|folder| folder.folder_id == "res://materials")
        .unwrap();
    assert!(!leaf.has_children && !leaf.expanded);
    assert!(!workspace.set_folder_expanded("res://materials", false, before.catalog_revision));
    assert!(!workspace.set_folder_expanded("res://missing", true, before.catalog_revision));
    assert!(workspace.set_folder_expanded("res://physics", false, before.catalog_revision));
    assert!(!workspace.set_folder_expanded("res://physics", true, before.catalog_revision));
    // Even equal source revision numbers do not make two catalog publications identical.
    let collapsed = snapshot(&workspace);
    workspace.sync_catalog(sample_catalog_generation());
    assert!(!workspace.set_folder_expanded("res://physics", true, collapsed.catalog_revision));
}

#[test]
fn asset_tree_disclosure_navigation_reveals_real_ancestors() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog_generation());
    let first = snapshot(&workspace);
    assert!(workspace.set_folder_expanded("res://", false, first.catalog_revision));
    assert!(workspace.select_folder("res://physics/materials"));
    let revealed = snapshot(&workspace);
    assert!(revealed
        .folder_tree
        .iter()
        .any(|folder| folder.selected && folder.folder_id == "res://physics/materials"));
    assert!(workspace.set_folder_expanded("res://physics", false, revealed.catalog_revision));
    assert!(workspace.navigate_to_asset("55555555-5555-5555-5555-555555555555"));
    assert!(snapshot(&workspace)
        .folder_tree
        .iter()
        .any(|folder| folder.selected && folder.folder_id == "res://physics/materials"));
    let visible = snapshot(&workspace);
    assert!(workspace.set_folder_expanded("res://physics", false, visible.catalog_revision));
    assert!(workspace
        .sync_content_browser_assets(&["55555555-5555-5555-5555-555555555555".to_string()]));
    let synced = snapshot(&workspace);
    assert_eq!(
        synced.selected_asset_uuid.as_deref(),
        Some("55555555-5555-5555-5555-555555555555")
    );
    assert!(synced
        .folder_tree
        .iter()
        .any(|folder| folder.selected && folder.folder_id == "res://physics/materials"));
}

#[test]
fn asset_tree_disclosure_refresh_prunes_removed_branches() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog_generation());
    let initial = snapshot(&workspace);
    assert!(workspace.set_folder_expanded("res://physics", false, initial.catalog_revision));
    let mut removed = sample_catalog();
    removed
        .folders
        .retain(|folder| !folder.folder_id.starts_with("res://physics"));
    workspace.sync_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(removed, 1),
    ));
    workspace.sync_catalog(sample_catalog_generation());
    assert!(snapshot(&workspace)
        .folder_tree
        .iter()
        .any(|folder| folder.folder_id == "res://physics/materials"));
    let current = snapshot(&workspace);
    assert!(workspace.set_folder_expanded("res://physics", false, current.catalog_revision));
    let mut other_project = sample_catalog();
    other_project.project_root = "E:/OtherProject".into();
    workspace.sync_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(other_project, 0),
    ));
    assert!(snapshot(&workspace)
        .folder_tree
        .iter()
        .any(|folder| folder.folder_id == "res://physics/materials"));
}
