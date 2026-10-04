use super::*;

fn folder(
    folder_id: &str,
    parent_folder_id: Option<&str>,
    display_name: &str,
    asset_count: usize,
) -> AssetFolderSnapshot {
    AssetFolderSnapshot {
        folder_id: folder_id.to_string(),
        parent_folder_id: parent_folder_id.map(str::to_string),
        display_name: display_name.to_string(),
        recursive_asset_count: asset_count,
        ..AssetFolderSnapshot::default()
    }
}

#[test]
fn selected_folder_breadcrumb_keeps_the_project_tree_and_asset_count() {
    let snapshot = AssetWorkspaceSnapshot {
        selected_folder_id: Some("res://models/props".to_string()),
        folder_tree: vec![
            folder("res://", None, "Content", 12),
            folder("res://models", Some("res://"), "Models", 8),
            folder("res://models/props", Some("res://models"), "Props", 5),
        ],
        ..AssetWorkspaceSnapshot::default()
    };

    assert_eq!(
        selected_folder_breadcrumb(&snapshot).as_deref(),
        Some("Content / Models / Props / 5 assets")
    );
}

#[test]
fn selected_folder_breadcrumb_does_not_loop_on_a_cyclic_catalog() {
    let snapshot = AssetWorkspaceSnapshot {
        selected_folder_id: Some("res://loop-a".to_string()),
        folder_tree: vec![
            folder("res://loop-a", Some("res://loop-b"), "A", 1),
            folder("res://loop-b", Some("res://loop-a"), "B", 2),
        ],
        ..AssetWorkspaceSnapshot::default()
    };

    let breadcrumb = selected_folder_breadcrumb(&snapshot).expect("selected folder path");
    assert!(breadcrumb.ends_with(" / 1 assets"));
    assert!(breadcrumb.len() < 64);
}

#[test]
fn selected_folder_breadcrumb_preserves_selection_and_parent_source_priority() {
    let snapshot = AssetWorkspaceSnapshot {
        selected_folder_id: Some("res://models/props".to_string()),
        folder_tree: vec![
            folder("res://", None, "Content", 12),
            folder("res://models", Some("res://"), "Tree Models", 8),
            folder("res://models/props", Some("res://models"), "Tree Props", 5),
        ],
        visible_folders: vec![
            folder("res://models", Some("res://"), "Visible Models", 8),
            folder(
                "res://models/props",
                Some("res://models"),
                "Visible Props",
                7,
            ),
        ],
        ..AssetWorkspaceSnapshot::default()
    };

    assert_eq!(
        selected_folder_breadcrumb(&snapshot).as_deref(),
        Some("Content / Tree Models / Visible Props / 7 assets")
    );
}
